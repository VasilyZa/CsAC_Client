use aes::{Aes128, Aes192, Aes256};
use base64::Engine;
use cbc::cipher::{block_padding::NoPadding, BlockDecryptMut, KeyIvInit};
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::header::HeaderMap;
use reqwest::multipart::{Form, Part};
use reqwest::{Client, Method};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Mutex;

const DIRECT_API_BASES: &[&str] = &[
    "https://cschat.ccccocccc.cc/web/rpc",
    "https://cschat.ccccocccc.cc/web/api",
    "https://csac.ccccocccc.cc/web/rpc",
    "https://csac.ccccocccc.cc/web/api",
];

const ROUTER_API_BASES: &[&str] = &[
    "https://csac.ccccocccc.cc/web/x.php",
    "https://csac.ccccocccc.cc/web/rpc.php",
    "https://cschat.ccccocccc.cc/web/x.php",
    "https://cschat.ccccocccc.cc/web/rpc.php",
];

const LEGACY_API_BASE: &str = "https://cschat.ccccocccc.cc";
const CSAC_LEGACY_BASE: &str = "https://csac.ccccocccc.cc";
const PRIMARY_RPC_BASE: &str = "https://cschat.ccccocccc.cc/web/rpc";

static HTTP: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .cookie_store(true)
        .user_agent(
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 \
             (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36 CsAC-Desktop/0.1",
        )
        .build()
        .expect("failed to create HTTP client")
});

static VMHOST_COOKIES: Lazy<Mutex<HashMap<String, String>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

static ACTIVE_BASE: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

static CHALLENGE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?s)\ba\s*=\s*toNumbers\("([0-9a-fA-F]+)"\).*?\bb\s*=\s*toNumbers\("([0-9a-fA-F]+)"\).*?\bc\s*=\s*toNumbers\("([0-9a-fA-F]+)"\)"#,
    )
    .expect("valid vmhost challenge regex")
});

static TO_NUMBERS_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"toNumbers\("([0-9a-fA-F]+)"\)"#).expect("valid toNumbers regex"));

static GROUP_ITEM_START_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)<div\s+class=["'][^"']*\bgroup-item\b[^"']*["'][^>]*>"#)
        .expect("valid group item regex")
});

static GROUP_NAME_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?is)<div\s+class=["'][^"']*\bgroup-name\b[^"']*["'][^>]*>(.*?)</div>"#)
        .expect("valid group name regex")
});

static GROUP_INTRO_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?is)<div\s+class=["'][^"']*\bgroup-intro\b[^"']*["'][^>]*>(.*?)</div>"#)
        .expect("valid group intro regex")
});

static GROUP_ID_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?is)(?:群组\s*ID\s*[：:]\s*|group_view\.php\?rid=)(\d+)"#)
        .expect("valid group id regex")
});

static GROUP_VIEW_LINK_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?is)<a\b[^>]*href=["'][^"']*group_view\.php\?rid=(\d+)[^"']*["'][^>]*>(.*?)</a>"#,
    )
    .expect("valid group view link regex")
});

#[derive(Debug, Deserialize)]
struct ApiRequest {
    method: String,
    path: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct ApiResponse {
    status: u16,
    data: Value,
    endpoint: String,
}

#[derive(Debug, Deserialize)]
struct AvatarUploadRequest {
    filename: String,
    mime: String,
    data_base64: String,
}

#[derive(Debug, Deserialize)]
struct ChatFileUploadRequest {
    kind: String,
    target_id: i64,
    file_kind: String,
    filename: String,
    mime: String,
    data_base64: String,
    duration: Option<i64>,
}

#[tauri::command]
async fn api_request(req: ApiRequest) -> Result<ApiResponse, String> {
    if !req.path.starts_with('/') || req.path.contains("://") || req.path.contains("..") {
        return Err("invalid api path".into());
    }

    let method = req
        .method
        .parse::<Method>()
        .map_err(|_| "invalid method".to_string())?;

    let params = req.params.unwrap_or_else(|| json!({}));
    if req.path == "/message/mark_read.php" && params.get("friend_id").is_some() {
        return Ok(ApiResponse {
            status: 200,
            data: json!({ "success": true, "message": "已标记已读" }),
            endpoint: "local://mark-private-read".to_string(),
        });
    }

    let mut last_error = None;
    let mut attempts = Vec::new();
    let endpoints = candidate_endpoints(&req.path);

    for endpoint in endpoints {
        let base = endpoint.base.clone();
        let first = match send_http_request(method.clone(), &endpoint, &params).await {
            Ok(response) => response,
            Err(err) => {
                last_error = Some(err.to_string());
                continue;
            }
        };
        remember_response_cookies(&base, first.headers())?;
        let status = first.status().as_u16();
        let text = first.text().await.map_err(|err| err.to_string())?;

        if looks_like_aes_challenge(&text) {
            match solve_vmhost_cookie(&text) {
                Ok(cookie) => {
                    merge_cookie(&base, &cookie)?;
                    let retry = match send_http_request(method.clone(), &endpoint, &params).await {
                        Ok(response) => response,
                        Err(err) => {
                            last_error = Some(err.to_string());
                            continue;
                        }
                    };
                    remember_response_cookies(&base, retry.headers())?;
                    let retry_status = retry.status().as_u16();
                    let retry_text = retry.text().await.map_err(|err| err.to_string())?;
                    let retry_data =
                        parse_response_text(&retry_text, retry_status, &endpoint.url, &req.path);

                    attempts.push(endpoint_attempt(&endpoint.url, retry_status, &retry_data));
                    if should_try_next_endpoint(&req.path, retry_status, &retry_data) {
                        last_error = Some(message_from_value(&retry_data));
                        continue;
                    }

                    if !retry_data
                        .get("challenge")
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
                    {
                        if is_success_response(&retry_data) {
                            remember_active_base(&base)?;
                        }
                        return Ok(ApiResponse {
                            status: retry_status,
                            data: retry_data,
                            endpoint: endpoint.url,
                        });
                    }
                    last_error = Some(message_from_value(&retry_data));
                    continue;
                }
                Err(err) => {
                    last_error = Some(err);
                    continue;
                }
            }
        }

        let data = parse_response_text(&text, status, &endpoint.url, &req.path);

        attempts.push(endpoint_attempt(&endpoint.url, status, &data));
        if should_try_next_endpoint(&req.path, status, &data) {
            last_error = Some(message_from_value(&data));
            continue;
        }

        if is_success_response(&data) {
            remember_active_base(&base)?;
        }

        return Ok(ApiResponse {
            status,
            data,
            endpoint: endpoint.url,
        });
    }

    if is_auth_probe(&req.path) && has_unauthorized_attempt(&attempts) {
        return Ok(ApiResponse {
            status: 401,
            data: json!({
                "success": false,
                "message": "请先登录"
            }),
            endpoint: String::new(),
        });
    }

    Ok(ApiResponse {
        status: 0,
        data: json!({
            "success": false,
            "message": last_error.unwrap_or_else(|| "无法连接 CsAC 服务器".to_string()),
            "attempts": attempts
        }),
        endpoint: String::new(),
    })
}

#[tauri::command]
async fn upload_avatar(req: AvatarUploadRequest) -> Result<ApiResponse, String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(req.data_base64.trim())
        .map_err(|_| "头像数据解析失败".to_string())?;
    if bytes.is_empty() {
        return Err("头像文件为空".into());
    }
    if bytes.len() > 5 * 1024 * 1024 {
        return Err("头像不能超过 5MB".into());
    }

    let endpoints = candidate_endpoints("/user/update_profile.php");
    let mut last_error = None;
    let mut attempts = Vec::new();

    for endpoint in endpoints {
        let part = Part::bytes(bytes.clone())
            .file_name(req.filename.clone())
            .mime_str(if req.mime.trim().is_empty() {
                "application/octet-stream"
            } else {
                req.mime.trim()
            })
            .map_err(|err| err.to_string())?;
        let form = Form::new().text("action", "avatar").part("avatar", part);

        let origin = endpoint_origin(&endpoint.url);
        let mut builder = HTTP
            .post(&endpoint.url)
            .header("Accept", "application/json, text/plain, */*")
            .header("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8")
            .header("Cache-Control", "no-cache")
            .header("Pragma", "no-cache")
            .header("X-Requested-With", "XMLHttpRequest")
            .header("Origin", origin.clone())
            .header("Referer", format!("{origin}/user_home.php"))
            .multipart(form);

        if let Some(cookie) = cookie_header_for(&endpoint.base) {
            builder = builder.header("Cookie", cookie);
        }

        let response = match builder.send().await {
            Ok(response) => response,
            Err(err) => {
                last_error = Some(err.to_string());
                continue;
            }
        };
        remember_response_cookies(&endpoint.base, response.headers())?;
        let status = response.status().as_u16();
        let text = response.text().await.map_err(|err| err.to_string())?;
        let data = parse_response_text(&text, status, &endpoint.url, "/user/update_profile.php");
        attempts.push(endpoint_attempt(&endpoint.url, status, &data));

        if should_try_next_endpoint("/user/update_profile.php", status, &data) {
            last_error = Some(message_from_value(&data));
            continue;
        }

        if is_success_response(&data) {
            remember_active_base(&endpoint.base)?;
        }

        return Ok(ApiResponse {
            status,
            data,
            endpoint: endpoint.url,
        });
    }

    Ok(ApiResponse {
        status: 0,
        data: json!({
            "success": false,
            "message": last_error.unwrap_or_else(|| "头像上传失败".to_string()),
            "attempts": attempts
        }),
        endpoint: String::new(),
    })
}

#[tauri::command]
async fn upload_chat_file(req: ChatFileUploadRequest) -> Result<ApiResponse, String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(req.data_base64.trim())
        .map_err(|_| "文件数据解析失败".to_string())?;
    if bytes.is_empty() {
        return Err("文件为空".into());
    }

    let is_group = req.kind == "group";
    let (path, field, max_bytes, fallback_message) = match req.file_kind.as_str() {
        "image" => {
            if bytes.len() > 5 * 1024 * 1024 {
                return Err("图片不能超过 5MB".into());
            }
            let path = if is_group {
                "/message/send_group_msg.php"
            } else {
                "/message/send_private_msg.php"
            };
            (path, "img", 5 * 1024 * 1024, "图片发送失败")
        }
        "voice" => {
            if bytes.len() > 10 * 1024 * 1024 {
                return Err("语音不能超过 10MB".into());
            }
            (
                "/message/send_voice_msg.php",
                "voice",
                10 * 1024 * 1024,
                "语音发送失败",
            )
        }
        _ => return Err("未知文件类型".into()),
    };

    if req.target_id <= 0 {
        return Err("无效的聊天对象".into());
    }
    if bytes.len() > max_bytes {
        return Err(fallback_message.into());
    }

    let endpoints = candidate_endpoints(path);
    let mut last_error = None;
    let mut attempts = Vec::new();

    for endpoint in endpoints {
        let mut form = Form::new();
        if is_group {
            form = form.text("room_id", req.target_id.to_string());
        } else {
            form = form.text("friend_id", req.target_id.to_string());
        }
        if req.file_kind == "image" {
            form = form.text("content", "");
        } else {
            form = form.text(
                "duration",
                req.duration.unwrap_or_default().max(0).to_string(),
            );
        }
        if let Some(act) = &endpoint.act {
            form = form.text("act", act.clone());
        }

        let part = Part::bytes(bytes.clone())
            .file_name(req.filename.clone())
            .mime_str(if req.mime.trim().is_empty() {
                if req.file_kind == "voice" {
                    "audio/webm"
                } else {
                    "application/octet-stream"
                }
            } else {
                req.mime.trim()
            })
            .map_err(|err| err.to_string())?;
        form = form.part(field, part);

        let origin = endpoint_origin(&endpoint.url);
        let referer = endpoint_referer(
            &endpoint.url,
            &json!({
                "friend_id": if is_group { Value::Null } else { json!(req.target_id) },
                "room_id": if is_group { json!(req.target_id) } else { Value::Null },
            }),
        )
        .unwrap_or_else(|| format!("{origin}/"));
        let mut builder = HTTP
            .post(&endpoint.url)
            .header("Accept", "application/json, text/plain, */*")
            .header("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8")
            .header("Cache-Control", "no-cache")
            .header("Pragma", "no-cache")
            .header("X-Requested-With", "XMLHttpRequest")
            .header("Origin", origin)
            .header("Referer", referer)
            .multipart(form);

        if let Some(cookie) = cookie_header_for(&endpoint.base) {
            builder = builder.header("Cookie", cookie);
        }

        let response = match builder.send().await {
            Ok(response) => response,
            Err(err) => {
                last_error = Some(err.to_string());
                continue;
            }
        };
        remember_response_cookies(&endpoint.base, response.headers())?;
        let status = response.status().as_u16();
        let text = response.text().await.map_err(|err| err.to_string())?;
        let data = parse_response_text(&text, status, &endpoint.url, path);
        attempts.push(endpoint_attempt(&endpoint.url, status, &data));

        if should_try_next_endpoint(path, status, &data) {
            last_error = Some(message_from_value(&data));
            continue;
        }

        if is_success_response(&data) {
            remember_active_base(&endpoint.base)?;
        }

        return Ok(ApiResponse {
            status,
            data,
            endpoint: endpoint.url,
        });
    }

    Ok(ApiResponse {
        status: 0,
        data: json!({
            "success": false,
            "message": last_error.unwrap_or_else(|| fallback_message.to_string()),
            "attempts": attempts
        }),
        endpoint: String::new(),
    })
}

fn candidate_endpoints(path: &str) -> Vec<ApiEndpoint> {
    let active = ACTIVE_BASE.lock().ok().and_then(|base| base.clone());
    let mut endpoints: Vec<ApiEndpoint> = Vec::new();

    push_endpoint(
        &mut endpoints,
        ApiEndpoint::from_base(PRIMARY_RPC_BASE, path),
    );

    if let Some(active) = active {
        let endpoint = ApiEndpoint::from_base(&active, path);
        if active != LEGACY_API_BASE {
            push_endpoint(&mut endpoints, endpoint);
        }
    }

    for base in DIRECT_API_BASES.iter().chain(ROUTER_API_BASES.iter()) {
        push_endpoint(&mut endpoints, ApiEndpoint::from_base(base, path));
    }
    if let Some(paths) = legacy_paths(path) {
        for legacy_path in paths {
            push_endpoint(&mut endpoints, ApiEndpoint::legacy(legacy_path));
        }
    }
    if let Some(pairs) = legacy_endpoint_bases(path) {
        for (base, legacy_path) in pairs {
            push_endpoint(
                &mut endpoints,
                ApiEndpoint::legacy_with_base(base, legacy_path),
            );
        }
    }
    endpoints
}

fn push_endpoint(endpoints: &mut Vec<ApiEndpoint>, endpoint: ApiEndpoint) {
    if !endpoints
        .iter()
        .any(|item| item.url == endpoint.url && item.body_format == endpoint.body_format)
    {
        endpoints.push(endpoint);
    }
}

fn remember_active_base(base: &str) -> Result<(), String> {
    *ACTIVE_BASE
        .lock()
        .map_err(|_| "active base lock poisoned".to_string())? = Some(base.to_string());
    Ok(())
}

fn is_success_response(data: &Value) -> bool {
    if let Some(value) = data.get("success").and_then(value_to_bool) {
        return value;
    }
    if let Some(value) = data.get("ok").and_then(value_to_bool) {
        return value;
    }
    if let Some(code) = data.get("code").and_then(value_to_i64) {
        return code == 0 || code == 200;
    }
    false
}

struct ApiEndpoint {
    url: String,
    base: String,
    act: Option<String>,
    body_format: BodyFormat,
}

impl ApiEndpoint {
    fn legacy(path: &'static str) -> Self {
        Self {
            url: format!("{LEGACY_API_BASE}{path}"),
            base: LEGACY_API_BASE.to_string(),
            act: None,
            body_format: BodyFormat::Form,
        }
    }

    fn legacy_with_base(base: &'static str, path: &'static str) -> Self {
        Self {
            url: format!("{base}{path}"),
            base: base.to_string(),
            act: None,
            body_format: BodyFormat::Form,
        }
    }

    fn from_base(base: &str, path: &str) -> Self {
        if base.ends_with(".php") {
            Self {
                url: base.to_string(),
                base: base.to_string(),
                act: Some(path.trim_start_matches('/').to_string()),
                body_format: BodyFormat::Form,
            }
        } else {
            Self {
                url: format!("{base}{path}"),
                base: base.to_string(),
                act: None,
                body_format: BodyFormat::Form,
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BodyFormat {
    Form,
}

fn legacy_paths(path: &str) -> Option<&'static [&'static str]> {
    match path {
        "/friend/send_request.php" => Some(&["/send_friend_request.php", "/add_friend.php"]),
        "/group/get_public_list.php" => Some(&["/group_list.php"]),
        "/message/get_private_msg.php" => Some(&["/get_friend_msg.php"]),
        "/message/send_private_msg.php" => Some(&["/send_friend_msg.php"]),
        _ => None,
    }
}

fn legacy_endpoint_bases(path: &str) -> Option<&'static [(&'static str, &'static str)]> {
    match path {
        "/group/get_public_list.php" => Some(&[
            (LEGACY_API_BASE, "/group_list.php"),
            (CSAC_LEGACY_BASE, "/group_list.php"),
        ]),
        _ => None,
    }
}

fn endpoint_attempt(url: &str, status: u16, data: &Value) -> Value {
    json!({
        "endpoint": url,
        "status": status,
        "message": message_from_value(data)
    })
}

fn is_auth_probe(path: &str) -> bool {
    path == "/user/get_info.php"
}

fn has_unauthorized_attempt(attempts: &[Value]) -> bool {
    attempts
        .iter()
        .any(|attempt| attempt.get("status").and_then(Value::as_u64) == Some(401))
}

fn should_try_next_endpoint(path: &str, status: u16, data: &Value) -> bool {
    if status == 401 {
        return true;
    }

    if path == "/group/get_public_list.php"
        && !data
            .get("endpoint")
            .and_then(Value::as_str)
            .unwrap_or("")
            .ends_with("/group_list.php")
        && is_success_response(data)
        && public_group_count(data).unwrap_or(0) == 0
    {
        return true;
    }

    let parse_error = data
        .get("parse_error")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if !parse_error {
        return false;
    }

    matches!(status, 0 | 200 | 204 | 400 | 404 | 405 | 500..=599)
}

async fn send_http_request(
    method: Method,
    endpoint: &ApiEndpoint,
    params: &Value,
) -> reqwest::Result<reqwest::Response> {
    let origin = endpoint_origin(&endpoint.url);
    let referer = endpoint_referer(&endpoint.url, params).unwrap_or_else(|| format!("{origin}/"));
    let mut builder = HTTP
        .request(method.clone(), &endpoint.url)
        .header("Accept", "application/json, text/plain, */*")
        .header("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8")
        .header("Cache-Control", "no-cache")
        .header("Pragma", "no-cache")
        .header("X-Requested-With", "XMLHttpRequest")
        .header("Sec-Fetch-Dest", "empty")
        .header("Sec-Fetch-Mode", "cors")
        .header("Sec-Fetch-Site", "same-origin")
        .header("Referer", referer);

    if let Some(cookie) = cookie_header_for(&endpoint.base) {
        builder = builder.header("Cookie", cookie);
    }

    if method == Method::GET {
        if let Some(query) = params.as_object() {
            let act = endpoint.act.iter().map(|act| ("act", act.clone()));
            let pairs: Vec<(&str, String)> = query
                .iter()
                .filter_map(|(key, value)| value_to_param(value).map(|v| (key.as_str(), v)))
                .chain(act)
                .collect();
            builder = builder.query(&pairs);
        }
    } else if let Some(form) = params.as_object() {
        let act = endpoint.act.iter().map(|act| ("act", act.clone()));
        let pairs: Vec<(&str, String)> = form
            .iter()
            .filter_map(|(key, value)| value_to_param(value).map(|v| (key.as_str(), v)))
            .chain(act)
            .collect();
        builder = builder.header("Origin", origin);
        builder = builder.form(&pairs);
    }

    builder.send().await
}

fn cookie_header_for(base: &str) -> Option<String> {
    let scope = cookie_scope(base);
    let cookies = VMHOST_COOKIES.lock().ok()?;
    cookies
        .get(scope.as_str())
        .cloned()
        .or_else(|| cookies.get(LEGACY_API_BASE).cloned())
}

fn value_to_param(value: &Value) -> Option<String> {
    match value {
        Value::Null => None,
        Value::String(value) => Some(value.clone()),
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(if *value { "1" } else { "0" }.to_string()),
        other => Some(other.to_string()),
    }
}

fn value_to_i64(value: &Value) -> Option<i64> {
    match value {
        Value::Number(value) => value.as_i64(),
        Value::String(value) => value.trim().parse().ok(),
        Value::Bool(value) => Some(if *value { 1 } else { 0 }),
        _ => None,
    }
}

fn value_to_bool(value: &Value) -> Option<bool> {
    match value {
        Value::Bool(value) => Some(*value),
        Value::Number(value) => Some(value.as_i64().unwrap_or_default() != 0),
        Value::String(value) => match value.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" | "success" | "ok" => Some(true),
            "0" | "false" | "no" | "off" | "fail" | "failed" | "error" => Some(false),
            _ => None,
        },
        _ => None,
    }
}

fn public_group_count(data: &Value) -> Option<usize> {
    for path in [
        &["groups"][..],
        &["rooms"][..],
        &["list"][..],
        &["items"][..],
        &["data"][..],
        &["data", "groups"][..],
        &["data", "rooms"][..],
        &["data", "list"][..],
        &["data", "items"][..],
        &["result", "groups"][..],
        &["result", "rooms"][..],
        &["result", "items"][..],
    ] {
        if let Some(count) =
            value_at_path(data, path).and_then(|value| value.as_array().map(Vec::len))
        {
            return Some(count);
        }
    }
    None
}

fn value_at_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    Some(current)
}

fn remember_response_cookies(base: &str, headers: &HeaderMap) -> Result<(), String> {
    for value in headers.get_all(reqwest::header::SET_COOKIE) {
        let Ok(cookie) = value.to_str() else {
            continue;
        };
        let Some(pair) = cookie.split(';').next() else {
            continue;
        };
        merge_cookie(base, pair.trim())?;
    }
    Ok(())
}

fn merge_cookie(base: &str, cookie_pair: &str) -> Result<(), String> {
    let Some((name, value)) = cookie_pair.split_once('=') else {
        return Ok(());
    };
    let name = name.trim();
    if name.is_empty() || name.eq_ignore_ascii_case("path") {
        return Ok(());
    }

    let mut cookies = VMHOST_COOKIES
        .lock()
        .map_err(|_| "cookie cache lock poisoned".to_string())?;
    let entry = cookies.entry(cookie_scope(base)).or_default();
    let mut pairs: Vec<(String, String)> = entry
        .split(';')
        .filter_map(|part| {
            let (key, val) = part.trim().split_once('=')?;
            Some((key.trim().to_string(), val.trim().to_string()))
        })
        .filter(|(key, _)| key != name)
        .collect();
    pairs.push((name.to_string(), value.trim().to_string()));
    *entry = pairs
        .into_iter()
        .map(|(key, val)| format!("{key}={val}"))
        .collect::<Vec<_>>()
        .join("; ");
    Ok(())
}

fn cookie_scope(base: &str) -> String {
    base.split("/web/")
        .next()
        .unwrap_or(base)
        .trim_end_matches('/')
        .to_string()
}

fn endpoint_origin(url: &str) -> String {
    let without_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"));
    if let Some(rest) = without_scheme {
        let scheme = if url.starts_with("https://") {
            "https"
        } else {
            "http"
        };
        if let Some(host) = rest.split('/').next() {
            return format!("{scheme}://{host}");
        }
    }
    LEGACY_API_BASE.to_string()
}

fn endpoint_referer(url: &str, params: &Value) -> Option<String> {
    if url.ends_with("/add_friend.php") {
        let to_uid = params.get("to_uid").and_then(value_to_param)?;
        return Some(format!("{LEGACY_API_BASE}/user_home.php?uid={to_uid}"));
    }
    if url.ends_with("/send_friend_msg.php") || url.ends_with("/get_friend_msg.php") {
        let friend_id = params.get("friend_id").and_then(value_to_param)?;
        return Some(format!("{LEGACY_API_BASE}/sct.php?friend_id={friend_id}"));
    }
    None
}

fn parse_response_text(text: &str, status: u16, url: &str, request_path: &str) -> Value {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return json!({
            "success": false,
            "message": "服务器返回了空响应",
            "status": status,
            "endpoint": url,
            "parse_error": true
        });
    }

    if looks_like_aes_challenge(trimmed) {
        return json!({
            "success": false,
            "message": "服务器防护页仍在拦截 API 请求，已尝试计算 vmhost __test Cookie 但未通过。",
            "status": status,
            "endpoint": url,
            "parse_error": true,
            "challenge": true
        });
    }

    if request_path == "/group/get_public_list.php" && url.ends_with("/group_list.php") {
        if let Some(groups) = parse_group_list_page(trimmed) {
            return json!({
                "success": true,
                "message": "公开群组加载成功",
                "groups": groups,
                "endpoint": url
            });
        }
    }

    if looks_like_site_script(trimmed) {
        return json!({
            "success": false,
            "message": "服务器返回了站点页面脚本，不是 API 响应",
            "status": status,
            "endpoint": url,
            "parse_error": true
        });
    }

    if trimmed.eq_ignore_ascii_case("success") {
        return json!({
            "success": true,
            "message": "操作成功"
        });
    }

    if let Ok(data) = serde_json::from_str(trimmed) {
        return data;
    }

    if let Some(json_text) = extract_json(trimmed) {
        if let Ok(data) = serde_json::from_str(json_text) {
            return data;
        }
    }

    let preview = if trimmed.contains('<') && trimmed.contains('>') {
        html_to_text(trimmed)
    } else {
        trimmed.chars().take(180).collect()
    };
    json!({
        "success": false,
        "message": if preview.is_empty() {
            "服务器返回了非 JSON 响应"
        } else {
            preview.as_str()
        },
        "status": status,
        "endpoint": url,
        "parse_error": true
    })
}

fn looks_like_aes_challenge(text: &str) -> bool {
    (text.contains("toNumbers") && text.contains("toHex"))
        || text.contains("slowAES")
        || text.contains("__test")
        || text.contains("document.cookie")
}

fn looks_like_site_script(text: &str) -> bool {
    (text.contains("js_css") && text.contains("document.createElement"))
        || text.contains("#site_header .submenus")
}

fn extract_json(text: &str) -> Option<&str> {
    let start = text.find(['{', '['])?;
    let end = text.rfind(['}', ']'])?;
    if start <= end {
        Some(&text[start..=end])
    } else {
        None
    }
}

fn parse_group_list_page(html: &str) -> Option<Vec<Value>> {
    if looks_like_login_page(html) {
        return None;
    }
    if !html.contains("group-list")
        && !html.contains("group-item")
        && !html.contains("group_view.php?rid=")
    {
        return None;
    }

    let starts: Vec<usize> = GROUP_ITEM_START_RE
        .find_iter(html)
        .map(|matched| matched.start())
        .collect();
    if starts.is_empty() {
        let groups = parse_group_links(html);
        if !groups.is_empty() {
            return Some(groups);
        }
        return if html.contains("暂无公开群组") {
            Some(Vec::new())
        } else {
            None
        };
    }

    let mut groups = Vec::new();
    for (index, start) in starts.iter().enumerate() {
        let end = starts.get(index + 1).copied().unwrap_or(html.len());
        let block = &html[*start..end];
        let Some(room_id) = GROUP_ID_RE
            .captures(block)
            .and_then(|caps| caps.get(1))
            .and_then(|value| value.as_str().parse::<i64>().ok())
        else {
            continue;
        };
        let room_name = GROUP_NAME_RE
            .captures(block)
            .and_then(|caps| caps.get(1))
            .map(|value| clean_html_text(value.as_str()))
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| format!("群组 {room_id}"));
        let intro = GROUP_INTRO_RE
            .captures(block)
            .and_then(|caps| caps.get(1))
            .map(|value| clean_html_text(value.as_str()))
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "暂无简介".to_string());
        groups.push(json!({
            "id": room_id,
            "room_id": room_id,
            "room_name": room_name,
            "intro": intro,
            "join_type": 1
        }));
    }

    if groups.is_empty() {
        let linked = parse_group_links(html);
        if linked.is_empty() {
            None
        } else {
            Some(linked)
        }
    } else {
        Some(groups)
    }
}

fn parse_group_links(html: &str) -> Vec<Value> {
    let mut groups: Vec<Value> = Vec::new();
    for caps in GROUP_VIEW_LINK_RE.captures_iter(html) {
        let Some(room_id) = caps
            .get(1)
            .and_then(|value| value.as_str().parse::<i64>().ok())
        else {
            continue;
        };
        if groups
            .iter()
            .any(|group| group.get("room_id").and_then(Value::as_i64) == Some(room_id))
        {
            continue;
        }
        let link_text = caps
            .get(2)
            .map(|value| clean_html_text(value.as_str()))
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| format!("群组 {room_id}"));
        groups.push(json!({
            "id": room_id,
            "room_id": room_id,
            "room_name": link_text,
            "intro": "暂无简介",
            "join_type": 1
        }));
    }
    groups
}

fn looks_like_login_page(html: &str) -> bool {
    html.contains("name=\"username\"")
        && html.contains("name=\"pwd\"")
        && (html.contains("立即登录") || html.contains("<title>登录</title>"))
}

fn clean_html_text(text: &str) -> String {
    decode_html_entities(&html_to_text(text))
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn decode_html_entities(text: &str) -> String {
    text.replace("&nbsp;", " ")
        .replace("&#160;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

fn html_to_text(text: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for ch in text.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    let compact = out.split_whitespace().collect::<Vec<_>>().join(" ");
    compact.chars().take(180).collect()
}

fn message_from_value(data: &Value) -> String {
    data.get("message")
        .and_then(Value::as_str)
        .unwrap_or("请求失败")
        .to_string()
}

fn solve_vmhost_cookie(script: &str) -> Result<String, String> {
    let (key_hex, iv_hex, cipher_hex) = extract_vmhost_params(script)
        .ok_or_else(|| "识别到 vmhost 防护页，但未找到 a/b/c 参数".to_string())?;
    let key = hex_to_bytes(&key_hex)?;
    let iv = hex_to_bytes(&iv_hex)?;
    let cipher_text = hex_to_bytes(&cipher_hex)?;

    if iv.len() != 16 {
        return Err("vmhost 防护页 IV 长度异常".to_string());
    }
    if !cipher_text.len().is_multiple_of(16) {
        return Err("vmhost 防护页密文长度异常".to_string());
    }

    let mut plain = match key.len() {
        16 => cbc::Decryptor::<Aes128>::new_from_slices(&key, &iv)
            .map_err(|err| err.to_string())?
            .decrypt_padded_vec_mut::<NoPadding>(&cipher_text)
            .map_err(|err| err.to_string())?,
        24 => cbc::Decryptor::<Aes192>::new_from_slices(&key, &iv)
            .map_err(|err| err.to_string())?
            .decrypt_padded_vec_mut::<NoPadding>(&cipher_text)
            .map_err(|err| err.to_string())?,
        32 => cbc::Decryptor::<Aes256>::new_from_slices(&key, &iv)
            .map_err(|err| err.to_string())?
            .decrypt_padded_vec_mut::<NoPadding>(&cipher_text)
            .map_err(|err| err.to_string())?,
        len => return Err(format!("vmhost 防护页 AES key 长度异常：{len}")),
    };

    strip_pkcs7_padding(&mut plain);
    Ok(format!("__test={}", bytes_to_hex(&plain)))
}

fn extract_vmhost_params(script: &str) -> Option<(String, String, String)> {
    if let Some(caps) = CHALLENGE_RE.captures(script) {
        return Some((
            caps[1].to_string(),
            caps[2].to_string(),
            caps[3].to_string(),
        ));
    }

    let values: Vec<String> = TO_NUMBERS_RE
        .captures_iter(script)
        .filter_map(|caps| caps.get(1).map(|value| value.as_str().to_string()))
        .filter(|value| value.len() >= 32)
        .collect();

    if values.len() >= 3 {
        Some((values[0].clone(), values[1].clone(), values[2].clone()))
    } else {
        None
    }
}

fn strip_pkcs7_padding(bytes: &mut Vec<u8>) {
    let Some(&last) = bytes.last() else {
        return;
    };
    let pad = last as usize;
    if pad == 0 || pad > 16 || pad > bytes.len() {
        return;
    }
    if bytes[bytes.len() - pad..]
        .iter()
        .all(|byte| *byte as usize == pad)
    {
        bytes.truncate(bytes.len() - pad);
    }
}

fn hex_to_bytes(input: &str) -> Result<Vec<u8>, String> {
    if !input.len().is_multiple_of(2) {
        return Err("hex 长度异常".to_string());
    }
    (0..input.len())
        .step_by(2)
        .map(|idx| {
            u8::from_str_radix(&input[idx..idx + 2], 16).map_err(|_| "hex 内容异常".to_string())
        })
        .collect()
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            api_request,
            upload_avatar,
            upload_chat_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
