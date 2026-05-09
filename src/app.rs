use futures::join;
use leptos::ev::{Event, SubmitEvent};
use leptos::html::{Div, Input, Select, Textarea};
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{json, Value};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Clone, Debug)]
enum Page {
    Login,
    Register,
    Home,
    PublicGroups,
    Notices,
    Account,
    About,
    UserDetail(i64),
    Report(ReportTarget),
    GroupChat(i64, String),
    GroupManage(i64, String),
    PrivateChat(i64, String),
}

#[derive(Clone, Debug)]
enum ReportTarget {
    User {
        uid: i64,
        username: String,
        nickname: String,
    },
    Group {
        room_id: i64,
        room_name: String,
    },
}

#[derive(Clone, Debug, Default, Deserialize)]
struct User {
    #[serde(alias = "id")]
    uid: i64,
    #[allow(dead_code)]
    username: Option<String>,
    nickname: Option<String>,
    avatar: Option<String>,
    #[allow(dead_code)]
    online_status: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct Friend {
    #[serde(deserialize_with = "de_i64")]
    friend_id: i64,
    display_name: Option<String>,
    username: Option<String>,
    avatar: Option<String>,
    #[serde(default, deserialize_with = "de_opt_i64")]
    unread_count: Option<i64>,
    online_status: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct Group {
    #[serde(
        default,
        alias = "id",
        alias = "rid",
        alias = "group_id",
        alias = "roomId",
        deserialize_with = "de_i64"
    )]
    room_id: i64,
    #[serde(
        default,
        alias = "name",
        alias = "title",
        deserialize_with = "de_opt_string"
    )]
    room_name: Option<String>,
    #[serde(
        default,
        alias = "description",
        alias = "desc",
        deserialize_with = "de_opt_string"
    )]
    intro: Option<String>,
    #[serde(default, deserialize_with = "de_opt_i64")]
    unread_count: Option<i64>,
    #[serde(default, deserialize_with = "de_i64")]
    join_type: i64,
    #[serde(default, deserialize_with = "de_opt_string")]
    owner_name: Option<String>,
    #[serde(default, deserialize_with = "de_opt_i64")]
    member_count: Option<i64>,
    #[serde(default, deserialize_with = "de_bool")]
    is_in_group: bool,
    #[serde(default, deserialize_with = "de_bool")]
    has_apply: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct Message {
    #[serde(deserialize_with = "de_i64")]
    id: i64,
    #[serde(default, deserialize_with = "de_opt_i64")]
    uid: Option<i64>,
    #[serde(default, deserialize_with = "de_opt_i64")]
    from_uid: Option<i64>,
    nickname: Option<String>,
    avatar: Option<String>,
    content: Option<String>,
    #[serde(default, deserialize_with = "de_opt_i64")]
    msg_type: Option<i64>,
    image_url: Option<String>,
    voice_url: Option<String>,
    #[serde(default, deserialize_with = "de_opt_i64")]
    duration: Option<i64>,
    #[serde(default, deserialize_with = "de_opt_i64")]
    voice_duration: Option<i64>,
    add_time: Option<String>,
    #[serde(default, deserialize_with = "de_opt_i64")]
    created_at: Option<i64>,
    #[allow(dead_code)]
    can_recall: Option<bool>,
    #[serde(default, deserialize_with = "de_opt_i64")]
    is_recalled: Option<i64>,
    #[serde(default, deserialize_with = "de_opt_i64")]
    is_read: Option<i64>,
    is_mentioned: Option<bool>,
    reply_to_me: Option<bool>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct FriendRequest {
    #[serde(deserialize_with = "de_i64")]
    id: i64,
    #[allow(dead_code)]
    #[serde(default, deserialize_with = "de_opt_i64")]
    friend_id: Option<i64>,
    nickname: Option<String>,
    #[allow(dead_code)]
    username: Option<String>,
    avatar: Option<String>,
    content: Option<String>,
    #[serde(rename = "type")]
    #[serde(default, deserialize_with = "de_opt_i64")]
    kind: Option<i64>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct DeletedNotice {
    #[serde(deserialize_with = "de_i64")]
    friend_id: i64,
    nickname: Option<String>,
    #[allow(dead_code)]
    username: Option<String>,
    avatar: Option<String>,
    #[serde(default, deserialize_with = "de_opt_i64")]
    delete_by: Option<i64>,
    delete_time: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct Notice {
    #[serde(deserialize_with = "de_i64")]
    id: i64,
    title: Option<String>,
    content: Option<String>,
    add_time: Option<String>,
    #[serde(default, deserialize_with = "de_opt_i64")]
    is_read: Option<i64>,
}

#[derive(Clone, Debug, Default)]
struct HomeData {
    friends: Vec<Friend>,
    groups: Vec<Group>,
    requests: Vec<FriendRequest>,
    deleted: Vec<DeletedNotice>,
    total_unread: i64,
}

#[derive(Clone, Debug, Default)]
struct UserSearchResult {
    uid: i64,
    username: String,
    nickname: String,
    avatar: Option<String>,
    is_friend: bool,
    request_sent: bool,
    request_received: bool,
    can_add_friend: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct UserProfile {
    #[serde(default, alias = "id", deserialize_with = "de_i64")]
    uid: i64,
    #[serde(default, deserialize_with = "de_opt_string")]
    username: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string")]
    nickname: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string")]
    avatar: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string")]
    remark: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string")]
    online_status: Option<String>,
    #[serde(default, deserialize_with = "de_bool")]
    is_self: bool,
    #[serde(default, deserialize_with = "de_bool")]
    is_friend: bool,
    #[serde(default, alias = "friend_request_sent", deserialize_with = "de_bool")]
    request_sent: bool,
    #[serde(
        default,
        alias = "friend_request_received",
        deserialize_with = "de_bool"
    )]
    request_received: bool,
    #[serde(default, deserialize_with = "de_bool")]
    is_blocked: bool,
    #[serde(default, deserialize_with = "de_bool")]
    can_add_friend: bool,
}

#[derive(Clone, Debug, Default)]
struct UserDetailData {
    user: UserProfile,
    created_groups: Vec<Group>,
}

#[derive(Clone, Debug, Default)]
struct GroupSearchResult {
    room_id: i64,
    room_name: String,
    intro: String,
    notice: String,
    owner_name: String,
    join_type: i64,
    ask_question: String,
    is_in_group: bool,
    has_apply: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct GroupManageRoom {
    #[allow(dead_code)]
    #[serde(default, alias = "id", alias = "room_id", deserialize_with = "de_i64")]
    room_id: i64,
    #[serde(default)]
    room_name: String,
    #[serde(default)]
    intro: String,
    #[serde(default)]
    notice: String,
    #[serde(default)]
    invite_code: String,
    #[serde(default, deserialize_with = "de_i64")]
    join_type: i64,
    #[allow(dead_code)]
    #[serde(default, deserialize_with = "de_i64")]
    owner_uid: i64,
    #[serde(default)]
    owner_name: String,
    #[serde(default)]
    ask_question: String,
    #[serde(default)]
    fixed_code: String,
    #[serde(default, deserialize_with = "de_i64")]
    show_in_list: i64,
    #[serde(default, deserialize_with = "de_i64")]
    allow_invite: i64,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct GroupMember {
    #[serde(default, deserialize_with = "de_i64")]
    uid: i64,
    #[serde(default)]
    nickname: String,
    #[serde(default, deserialize_with = "de_opt_string")]
    avatar: Option<String>,
    #[serde(default)]
    is_owner: bool,
    #[serde(default)]
    is_admin: bool,
    #[serde(default)]
    is_muted: bool,
    #[serde(default, deserialize_with = "de_i64")]
    mute_until: i64,
    #[serde(default)]
    online_status: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct GroupApplication {
    #[serde(default, deserialize_with = "de_i64")]
    id: i64,
    #[serde(default, deserialize_with = "de_opt_i64")]
    uid: Option<i64>,
    #[serde(default)]
    nickname: String,
    #[serde(default)]
    answer_content: String,
    #[serde(default)]
    apply_time: String,
}

#[derive(Clone, Debug, Default)]
struct GroupManageData {
    room: GroupManageRoom,
    members: Vec<GroupMember>,
    applications: Vec<GroupApplication>,
    application_error: Option<String>,
    #[allow(dead_code)]
    is_in_group: bool,
    is_owner: bool,
    is_admin: bool,
    can_view_invite: bool,
}

fn de_i64<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    Ok(value_to_i64(&value).unwrap_or_default())
}

fn de_opt_i64<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    Ok(value_to_i64(&value))
}

fn de_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    Ok(value_to_bool(&value).unwrap_or(false))
}

fn de_opt_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    Ok(value_to_string(&value))
}

fn value_to_i64(value: &Value) -> Option<i64> {
    match value {
        Value::Number(value) => value.as_i64(),
        Value::String(value) => value.parse().ok(),
        Value::Bool(value) => Some(if *value { 1 } else { 0 }),
        _ => None,
    }
}

fn value_to_bool(value: &Value) -> Option<bool> {
    match value {
        Value::Bool(value) => Some(*value),
        Value::Number(value) => Some(value.as_i64().unwrap_or_default() != 0),
        Value::String(value) => {
            let value = value.trim().to_ascii_lowercase();
            match value.as_str() {
                "1" | "true" | "yes" | "on" | "success" | "ok" => Some(true),
                "0" | "false" | "no" | "off" | "fail" | "failed" | "error" => Some(false),
                _ => None,
            }
        }
        _ => None,
    }
}

fn value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::Null => None,
        Value::String(value) => Some(value.clone()),
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(if *value { "1" } else { "0" }.to_string()),
        _ => None,
    }
}

#[derive(Serialize)]
struct InvokeArgs {
    req: ApiRequest,
}

#[derive(Serialize)]
struct ApiRequest {
    method: String,
    path: String,
    params: Value,
}

#[derive(Deserialize)]
struct ApiResponse {
    status: u16,
    data: Value,
    #[allow(dead_code)]
    endpoint: String,
}

#[derive(Serialize)]
struct AvatarUploadArgs {
    req: AvatarUploadRequest,
}

#[derive(Serialize)]
struct AvatarUploadRequest {
    filename: String,
    mime: String,
    data_base64: String,
}

#[derive(Serialize)]
struct ChatFileUploadArgs {
    req: ChatFileUploadRequest,
}

#[derive(Clone, Debug)]
struct PendingFile {
    filename: String,
    mime: String,
    data_base64: String,
}

#[derive(Serialize)]
struct ChatFileUploadRequest {
    kind: String,
    target_id: i64,
    file_kind: String,
    filename: String,
    mime: String,
    data_base64: String,
    duration: Option<i64>,
}

#[component]
pub fn App() -> impl IntoView {
    let page = RwSignal::new(Page::Login);
    let me = RwSignal::new(None::<User>);
    let home = RwSignal::new(HomeData::default());
    let public_groups = RwSignal::new(Vec::<Group>::new());
    let notices = RwSignal::new(Vec::<Notice>::new());
    let messages = RwSignal::new(Vec::<Message>::new());
    let group_manage = RwSignal::new(None::<GroupManageData>);
    let found_user = RwSignal::new(None::<UserSearchResult>);
    let found_group = RwSignal::new(None::<GroupSearchResult>);
    let user_detail = RwSignal::new(None::<UserDetailData>);
    let status = RwSignal::new("正在连接 CsAC...".to_string());
    let loading = RwSignal::new(false);
    let dark_mode = RwSignal::new(load_dark_mode());

    let toggle_dark_mode = move |_| {
        dark_mode.update(|value| {
            *value = !*value;
            save_dark_mode(*value);
        });
    };

    let refresh_home = move |switch_to_home: bool| {
        loading.set(true);
        if switch_to_home && me.get_untracked().is_some() {
            page.set(Page::Home);
            status.set(String::new());
        }
        spawn_local(async move {
            let result = load_home_data().await;
            loading.set(false);
            match result {
                Ok((user, data)) => {
                    me.set(Some(user));
                    home.set(data);
                    if switch_to_home {
                        page.set(Page::Home);
                    }
                    status.set(String::new());
                }
                Err(err) => {
                    me.set(None);
                    page.set(Page::Login);
                    status.set(err);
                }
            }
        });
    };

    Effect::new(move |_| refresh_home(true));

    let open_public_groups = move || {
        page.set(Page::PublicGroups);
        if !public_groups.get_untracked().is_empty() {
            status.set(String::new());
        }
        loading.set(true);
        spawn_local(async move {
            match api_get("/group/get_public_list", json!({})).await {
                Ok(data) => {
                    let groups = public_group_list(&data);
                    public_groups.set(groups);
                    if public_groups.get_untracked().is_empty() {
                        status.set(public_group_empty_message(&data));
                    } else {
                        status.set(String::new());
                    }
                }
                Err(err) => status.set(err),
            }
            loading.set(false);
        });
    };

    let open_notices = move || {
        page.set(Page::Notices);
        if !notices.get_untracked().is_empty() {
            status.set(String::new());
        }
        loading.set(true);
        spawn_local(async move {
            match api_get("/user/get_notice_list", json!({})).await {
                Ok(data) => {
                    notices.set(list_from_field(&data, "notices"));
                    status.set(String::new());
                }
                Err(err) => status.set(err),
            }
            loading.set(false);
        });
    };

    let open_user_detail = move |uid: i64| {
        if uid <= 0 {
            status.set("无效的用户 ID".into());
            return;
        }
        user_detail.set(None);
        page.set(Page::UserDetail(uid));
        loading.set(true);
        spawn_local(async move {
            match load_user_detail(uid).await {
                Ok(data) => {
                    user_detail.set(Some(data));
                    status.set(String::new());
                }
                Err(err) => status.set(err),
            }
            loading.set(false);
        });
    };

    let logout = move |_| {
        spawn_local(async move {
            let _ = api_post("/auth/logout", json!({})).await;
            me.set(None);
            home.set(HomeData::default());
            page.set(Page::Login);
            status.set("已退出登录".to_string());
        });
    };

    view! {
        <div class="app-shell" class:dark=move || dark_mode.get()>
            <aside class="sidebar">
                <div class="brand">
                    <div class="brand-mark">
                        <img src="/icon/favicon.ico" alt="CsAC" />
                    </div>
                    <div>
                        <div class="brand-name">"CsAC"</div>
                        <div class="brand-sub">"桌面聊天客户端"</div>
                    </div>
                </div>

                <nav class="nav">
                    <button class:active=move || matches!(page.get(), Page::Home) on:click=move |_| refresh_home(true)>"主页"</button>
                    <button class:active=move || matches!(page.get(), Page::PublicGroups) on:click=move |_| open_public_groups()>"公开群组"</button>
                    <button class:active=move || matches!(page.get(), Page::Notices) on:click=move |_| open_notices()>
                        "通知"
                        <Show when=move || { home.get().total_unread > 0 }>
                            <span class="pill">{move || home.get().total_unread}</span>
                        </Show>
                    </button>
                    <button class:active=move || matches!(page.get(), Page::Account) on:click=move |_| {
                        page.set(Page::Account);
                        status.set(String::new());
                    }>"账户"</button>
                    <button class:active=move || matches!(page.get(), Page::About) on:click=move |_| {
                        page.set(Page::About);
                        status.set(String::new());
                    }>"关于"</button>
                </nav>

                <div class="sidebar-user">
                    <Show
                        when=move || me.get().is_some()
                        fallback=move || view! { <div class="muted">"未登录"</div> }
                    >
                        {move || me.get().map(|user| view! {
                            <div class="mini-user">
                                <Avatar src=user.avatar.clone() label=user.nickname.clone().unwrap_or_else(|| "我".into())/>
                                <div>
                                    <strong>{user.nickname.unwrap_or_else(|| "CsAC 用户".into())}</strong>
                                    <span>{format!("#{}", user.uid)}</span>
                                </div>
                            </div>
                        })}
                        <button class="ghost wide" on:click=move |_| {
                            if let Some(user) = me.get_untracked() {
                                open_user_detail(user.uid);
                            }
                        }>"我的资料"</button>
                        <button class="ghost wide" on:click=logout>"退出登录"</button>
                    </Show>
                </div>
            </aside>

            <main class="content" class:chat-mode=move || matches!(page.get(), Page::GroupChat(_, _) | Page::PrivateChat(_, _))>
                <header class="topbar">
                    <div>
                        <h1>{move || page_title(&page.get())}</h1>
                        <p>{move || page_subtitle(&page.get())}</p>
                    </div>
                    <div class="top-actions">
                        <Show when=move || loading.get()>
                            <span class="loading">"加载中"</span>
                        </Show>
                        <button class="ghost theme-toggle" on:click=toggle_dark_mode>
                            {move || if dark_mode.get() { "浅色模式" } else { "深色模式" }}
                        </button>
                        <button class="ghost" on:click=move |_| refresh_home(false)>"刷新"</button>
                    </div>
                </header>

                <Show when=move || !status.get().is_empty()>
                    <div class="status">{move || status.get()}</div>
                </Show>

                {move || match page.get() {
                    Page::Login => view! {
                        <LoginView
                            on_done=Callback::new(move |_| refresh_home(true))
                            show_register=Callback::new(move |_| page.set(Page::Register))
                            status=status
                        />
                    }.into_any(),
                    Page::Register => view! {
                        <RegisterView
                            on_done=Callback::new(move |_| refresh_home(true))
                            show_login=Callback::new(move |_| page.set(Page::Login))
                            status=status
                        />
                    }.into_any(),
                    Page::Home => view! {
                        <HomeView
                            home=home
                            found_user=found_user
                            found_group=found_group
                            my_uid=me.get().map(|u| u.uid).unwrap_or_default()
                            open_group=Callback::new(move |group: Group| {
                                let id = group.room_id;
                                let name = group.room_name.unwrap_or_else(|| format!("群组 {id}"));
                                open_group_chat(id, name, messages, page, status);
                            })
                            open_friend=Callback::new(move |friend: Friend| {
                                let id = friend.friend_id;
                                let name = friend.display_name.unwrap_or_else(|| format!("用户 {id}"));
                                clear_friend_unread(home, id);
                                open_private_chat(id, name, messages, page, status);
                            })
                            open_user=Callback::new(move |uid| open_user_detail(uid))
                            report_group=Callback::new(move |(room_id, room_name): (i64, String)| {
                                page.set(Page::Report(ReportTarget::Group { room_id, room_name }));
                                status.set(String::new());
                            })
                            refresh=Callback::new(move |_| refresh_home(false))
                            status=status
                        />
                    }.into_any(),
                    Page::PublicGroups => view! {
                        <PublicGroupsView
                            groups=public_groups
                            refresh=Callback::new(move |_| open_public_groups())
                            report_group=Callback::new(move |(room_id, room_name): (i64, String)| {
                                page.set(Page::Report(ReportTarget::Group { room_id, room_name }));
                                status.set(String::new());
                            })
                            status=status
                        />
                    }.into_any(),
                    Page::Notices => view! {
                        <NoticesView notices=notices refresh=Callback::new(move |_| open_notices()) status=status/>
                    }.into_any(),
                    Page::Account => view! {
                        <AccountView
                            me=me
                            refresh=Callback::new(move |_| refresh_home(false))
                            status=status
                        />
                    }.into_any(),
                    Page::About => view! {
                        <AboutView/>
                    }.into_any(),
                    Page::UserDetail(uid) => view! {
                        <UserDetailView
                            uid=uid
                            data=user_detail
                            my_uid=me.get().map(|u| u.uid).unwrap_or_default()
                            back=Callback::new(move |_| refresh_home(true))
                            refresh=Callback::new(move |uid| open_user_detail(uid))
                            open_private=Callback::new(move |(uid, name): (i64, String)| {
                                clear_friend_unread(home, uid);
                                open_private_chat(uid, name, messages, page, status);
                            })
                            open_group=Callback::new(move |group: Group| {
                                let id = group.room_id;
                                let name = group.room_name.unwrap_or_else(|| format!("群组 {id}"));
                                open_group_chat(id, name, messages, page, status);
                            })
                            report_user=Callback::new(move |target: ReportTarget| {
                                page.set(Page::Report(target));
                                status.set(String::new());
                            })
                            status=status
                        />
                    }.into_any(),
                    Page::Report(target) => view! {
                        <ReportView
                            target=target
                            back=Callback::new(move |_| refresh_home(true))
                            status=status
                        />
                    }.into_any(),
                    Page::GroupChat(room_id, room_name) => view! {
                        <ChatView
                            kind="group"
                            target_id=room_id
                            title=room_name
                            messages=messages
                            my_uid=me.get().map(|u| u.uid).unwrap_or_default()
                            back=Callback::new(move |_| refresh_home(true))
                            manage=Some(Callback::new(move |(room_id, room_name): (i64, String)| {
                                open_group_manage(room_id, room_name, group_manage, page, status);
                            }))
                            open_user=Callback::new(move |uid| open_user_detail(uid))
                            report_group=Some(Callback::new(move |(room_id, room_name): (i64, String)| {
                                page.set(Page::Report(ReportTarget::Group { room_id, room_name }));
                                status.set(String::new());
                            }))
                            status=status
                        />
                    }.into_any(),
                    Page::GroupManage(room_id, room_name) => view! {
                        <GroupManageView
                            room_id=room_id
                            room_name=room_name
                            data=group_manage
                            my_uid=me.get().map(|u| u.uid).unwrap_or_default()
                            back=Callback::new(move |(room_id, room_name): (i64, String)| {
                                open_group_chat(room_id, room_name, messages, page, status);
                            })
                            refresh=Callback::new(move |(room_id, room_name): (i64, String)| {
                                load_group_manage(room_id, room_name, group_manage, status);
                            })
                            home=Callback::new(move |_| refresh_home(true))
                            open_user=Callback::new(move |uid| open_user_detail(uid))
                            report_group=Callback::new(move |(room_id, room_name): (i64, String)| {
                                page.set(Page::Report(ReportTarget::Group { room_id, room_name }));
                                status.set(String::new());
                            })
                            status=status
                        />
                    }.into_any(),
                    Page::PrivateChat(friend_id, name) => view! {
                        <ChatView
                            kind="private"
                            target_id=friend_id
                            title=name
                            messages=messages
                            my_uid=me.get().map(|u| u.uid).unwrap_or_default()
                            back=Callback::new(move |_| refresh_home(true))
                            manage=None
                            open_user=Callback::new(move |uid| open_user_detail(uid))
                            report_group=None
                            status=status
                        />
                    }.into_any(),
                }}
            </main>
        </div>
    }
}

#[component]
fn LoginView(
    on_done: Callback<()>,
    show_register: Callback<()>,
    status: RwSignal<String>,
) -> impl IntoView {
    let username = NodeRef::<Input>::new();
    let pwd = NodeRef::<Input>::new();

    let submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let username_value = username.get().map(|el| el.value()).unwrap_or_default();
        let pwd_value = pwd.get().map(|el| el.value()).unwrap_or_default();
        spawn_local(async move {
            status.set("正在登录...".into());
            match api_post(
                "/auth/login",
                json!({ "username": username_value, "pwd": pwd_value }),
            )
            .await
            {
                Ok(data) if is_success(&data) => on_done.run(()),
                Ok(data) => status.set(message_of(&data, "登录失败")),
                Err(err) => status.set(err),
            }
        });
    };

    view! {
        <section class="auth-panel">
            <div class="auth-copy">
                <h2>"登录 CsAC"</h2>
                <p>"使用原网站账号进入好友、群组和聊天。客户端会在本机保持会话。"</p>
            </div>
            <form class="auth-form" on:submit=submit>
                <label>"账号"<input node_ref=username autocomplete="username" placeholder="登录账号"/></label>
                <label>"密码"<input node_ref=pwd type="password" autocomplete="current-password" placeholder="登录密码"/></label>
                <button class="primary" type="submit">"登录"</button>
                <button class="link-button" type="button" on:click=move |_| show_register.run(())>"没有账号，去注册"</button>
            </form>
        </section>
    }
}

#[component]
fn RegisterView(
    on_done: Callback<()>,
    show_login: Callback<()>,
    status: RwSignal<String>,
) -> impl IntoView {
    let username = NodeRef::<Input>::new();
    let nickname = NodeRef::<Input>::new();
    let pwd = NodeRef::<Input>::new();
    let confirm = NodeRef::<Input>::new();

    let submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let payload = json!({
            "username": username.get().map(|el| el.value()).unwrap_or_default(),
            "nickname": nickname.get().map(|el| el.value()).unwrap_or_default(),
            "pwd": pwd.get().map(|el| el.value()).unwrap_or_default(),
            "confirm_pwd": confirm.get().map(|el| el.value()).unwrap_or_default(),
        });
        spawn_local(async move {
            status.set("正在注册...".into());
            match api_post("/auth/register", payload).await {
                Ok(data) if is_success(&data) => on_done.run(()),
                Ok(data) => status.set(message_of(&data, "注册失败")),
                Err(err) => status.set(err),
            }
        });
    };

    view! {
        <section class="auth-panel">
            <div class="auth-copy">
                <h2>"创建账号"</h2>
                <p>"注册后会自动进入桌面客户端。头像上传可在原站个人资料中继续设置。"</p>
            </div>
            <form class="auth-form" on:submit=submit>
                <label>"账号"<input node_ref=username autocomplete="username" placeholder="用于登录"/></label>
                <label>"昵称"<input node_ref=nickname placeholder="显示昵称"/></label>
                <label>"密码"<input node_ref=pwd type="password" autocomplete="new-password"/></label>
                <label>"确认密码"<input node_ref=confirm type="password" autocomplete="new-password"/></label>
                <button class="primary" type="submit">"注册"</button>
                <button class="link-button" type="button" on:click=move |_| show_login.run(())>"已有账号，返回登录"</button>
            </form>
        </section>
    }
}

#[component]
fn HomeView(
    home: RwSignal<HomeData>,
    found_user: RwSignal<Option<UserSearchResult>>,
    found_group: RwSignal<Option<GroupSearchResult>>,
    my_uid: i64,
    open_group: Callback<Group>,
    open_friend: Callback<Friend>,
    open_user: Callback<i64>,
    report_group: Callback<(i64, String)>,
    refresh: Callback<()>,
    status: RwSignal<String>,
) -> impl IntoView {
    let group_id = NodeRef::<Input>::new();
    let user_id = NodeRef::<Input>::new();
    let room_name = NodeRef::<Input>::new();
    let join_code = NodeRef::<Input>::new();
    let join_answer = NodeRef::<Input>::new();

    let create_group = move |ev: SubmitEvent| {
        ev.prevent_default();
        let name = room_name.get().map(|el| el.value()).unwrap_or_default();
        spawn_local(async move {
            match api_post("/group/create", json!({ "room_name": name })).await {
                Ok(data) if is_success(&data) => {
                    status.set("群组已创建".into());
                    refresh.run(());
                }
                Ok(data) => status.set(message_of(&data, "创建失败")),
                Err(err) => status.set(err),
            }
        });
    };

    let search_group = move |ev: SubmitEvent| {
        ev.prevent_default();
        let id = group_id
            .get()
            .and_then(|el| el.value().trim().parse::<i64>().ok())
            .unwrap_or_default();
        if id <= 0 {
            status.set("请输入有效群组编号".into());
            return;
        }
        spawn_local(async move {
            status.set("正在查找群组...".into());
            match api_get("/group/get_group_view_info", json!({ "rid": id })).await {
                Ok(data) if is_success(&data) => match group_search_result(&data) {
                    Some(group) => {
                        status.set(format!("找到群组：{}", group.room_name));
                        found_group.set(Some(group));
                    }
                    None => status.set("找到群组，但资料解析失败".into()),
                },
                Ok(data) => status.set(message_of(&data, "未找到该群组")),
                Err(err) => status.set(err),
            }
        });
    };

    let search_user = move |ev: SubmitEvent| {
        ev.prevent_default();
        let uid = user_id
            .get()
            .and_then(|el| el.value().trim().parse::<i64>().ok())
            .unwrap_or_default();
        if uid <= 0 || uid == my_uid {
            status.set("请输入有效用户 ID".into());
            return;
        }
        spawn_local(async move {
            match api_get("/user/get_info", json!({ "uid": uid })).await {
                Ok(data) if is_success(&data) => match user_search_result(&data) {
                    Some(user) => {
                        status.set(format!("找到用户：{}", user.nickname));
                        found_user.set(Some(user));
                    }
                    None => status.set("找到用户，但资料解析失败".into()),
                },
                Ok(data) => status.set(message_of(&data, "未找到该用户")),
                Err(err) => status.set(err),
            }
        });
    };

    view! {
        <div class="home-workbench">
            <section class="workspace dashboard-hero">
                <div class="hero-copy">
                    <span class="eyebrow">"CsAC Desktop"</span>
                    <h2>"消息中枢"</h2>
                    <p>"好友、群组和请求集中在一个工作台里。"</p>
                </div>
                <div class="stat-strip">
                    <div class="stat-card">
                        <strong>{move || home.get().friends.len()}</strong>
                        <span>"好友"</span>
                    </div>
                    <div class="stat-card">
                        <strong>{move || home.get().groups.len()}</strong>
                        <span>"群组"</span>
                    </div>
                    <div class="stat-card accent">
                        <strong>{move || home.get().total_unread}</strong>
                        <span>"未读"</span>
                    </div>
                </div>
            </section>

            <div class="home-main">
                <section class="workspace conversations-panel">
                    <div class="section-head">
                        <div>
                            <h2>"会话"</h2>
                            <span>"好友和群组"</span>
                        </div>
                        <button class="ghost" on:click=move |_| refresh.run(())>"同步"</button>
                    </div>
                    <div class="conversation-columns">
                        <div class="conversation-section">
                            <div class="subsection-head">
                                <h3>"好友"</h3>
                                <span>{move || format!("{} 位", home.get().friends.len())}</span>
                            </div>
                            <div class="list dense-list">
                                <Show when=move || !home.get().friends.is_empty() fallback=move || view!{ <Empty text="还没有好友"/> }>
                                    <For
                                        each=move || home.get().friends
                                        key=|friend| friend.friend_id
                                        children=move |friend| {
                                            let item = friend.clone();
                                            let profile_uid = friend.friend_id;
                                            let display_name = friend.display_name.clone().unwrap_or_else(|| "未命名好友".into());
                                            let username = friend.username.clone().unwrap_or_default();
                                            let online = strip_html(friend.online_status.as_deref().unwrap_or(""));
                                            view! {
                                                <div class="row-item conversation-item">
                                                    <button class="avatar-button" type="button" title="查看资料" on:click=move |_| open_user.run(profile_uid)>
                                                        <Avatar src=friend.avatar.clone() label=display_name.clone()/>
                                                    </button>
                                                    <button class="row-main row-open" type="button" on:click=move |_| open_friend.run(item.clone())>
                                                        <strong>{display_name}</strong>
                                                        <small>{format!("@{} {}", username, online)}</small>
                                                    </button>
                                                    <Unread count=friend.unread_count.unwrap_or_default()/>
                                                    <button class="ghost mini-action" type="button" on:click=move |_| open_user.run(profile_uid)>"资料"</button>
                                                </div>
                                            }
                                        }
                                    />
                                </Show>
                            </div>
                        </div>

                        <div class="conversation-section">
                            <div class="subsection-head">
                                <h3>"群组"</h3>
                                <span>{move || format!("{} 个", home.get().groups.len())}</span>
                            </div>
                            <div class="list dense-list">
                                <Show when=move || !home.get().groups.is_empty() fallback=move || view!{ <Empty text="还没有加入群组"/> }>
                                    <For
                                        each=move || home.get().groups
                                        key=|group| group.room_id
                                        children=move |group| {
                                            let item = group.clone();
                                            view! {
                                                <button class="row-item conversation-item" on:click=move |_| open_group.run(item.clone())>
                                                    <div class="room-icon">"#"</div>
                                                    <span class="row-main">
                                                        <strong>{group.room_name.unwrap_or_else(|| "未命名群组".into())}</strong>
                                                        <small>{format!("群组 ID {}", group.room_id)}</small>
                                                    </span>
                                                    <Unread count=group.unread_count.unwrap_or_default()/>
                                                </button>
                                            }
                                        }
                                    />
                                </Show>
                            </div>
                        </div>
                    </div>
                </section>

                <section class="workspace alerts requests-panel">
                    <div class="section-head"><h2>"好友通知"</h2></div>
                    <FriendAlerts home=home my_uid=my_uid refresh=refresh status=status/>
                </section>
            </div>

            <aside class="home-side">
                <section class="workspace tools action-panel">
                    <div class="section-head">
                        <div>
                            <h2>"快速操作"</h2>
                            <span>"创建、查找、加入"</span>
                        </div>
                    </div>
                    <div class="action-stack">
                        <form class="inline-form action-form" on:submit=create_group>
                            <div>
                                <strong>"创建群组"</strong>
                                <small>"输入名称后立即创建并加入"</small>
                            </div>
                            <div class="action-control">
                                <input node_ref=room_name placeholder="新群组名称"/>
                                <button type="submit">"创建"</button>
                            </div>
                        </form>
                        <form class="inline-form action-form" on:submit=search_group>
                            <div>
                                <strong>"查找群组"</strong>
                                <small>"按群组编号搜索并申请加入"</small>
                            </div>
                            <div class="action-control">
                                <input node_ref=group_id placeholder="群组编号"/>
                                <button type="submit">"查找"</button>
                            </div>
                        </form>
                        <form class="inline-form action-form" on:submit=search_user>
                            <div>
                                <strong>"查找用户"</strong>
                                <small>"按用户 ID 添加好友"</small>
                            </div>
                            <div class="action-control">
                                <input node_ref=user_id placeholder="用户 ID"/>
                                <button type="submit">"查找"</button>
                            </div>
                        </form>
                    </div>
                    <div class="result-stack">
                        <Show when=move || found_group.get().is_some()>
                            {move || found_group.get().map(|group| {
                                let room_id = group.room_id;
                                let room_name_value = group.room_name.clone();
                                let intro = if group.intro.trim().is_empty() { "这个群组还没有简介。".to_string() } else { group.intro.clone() };
                                let notice = group.notice.clone();
                                let has_notice = !notice.trim().is_empty();
                                let notice_text = format!("公告：{}", notice);
                                let owner = group.owner_name.clone();
                                let join_type = group.join_type;
                                let is_in_group = group.is_in_group;
                                let has_apply = group.has_apply;
                                let question = group.ask_question.clone();
                                let clear_result = found_group;
                                let open_group_result = open_group;
                                let refresh_home = refresh;
                                let after_join = Callback::new(move |_| {
                                    clear_result.set(None);
                                    refresh_home.run(());
                                });
                                let open_room_name = StoredValue::new(room_name_value.clone());
                                view! {
                                    <div class="search-result group-result">
                                        <div class="room-icon">"#"</div>
                                        <div class="row-main">
                                            <div class="result-title">
                                                <strong>{room_name_value.clone()}</strong>
                                                <span>{group_join_state_text(is_in_group, has_apply, join_type)}</span>
                                            </div>
                                            <small>{format!("群组 ID {} · 创建者 {}", room_id, owner)}</small>
                                            <p>{intro}</p>
                                            <Show when=move || has_notice>
                                                <small>{notice_text.clone()}</small>
                                            </Show>
                                            <Show when=move || !is_in_group && !has_apply && (join_type == 2 || join_type == 3)>
                                                <input node_ref=join_code placeholder="输入邀请码"/>
                                            </Show>
                                            <Show when=move || !is_in_group && !has_apply && join_type == 4>
                                                <div class="question-box">
                                                    <small>{if question.trim().is_empty() { "该群组需要回答入群问题。".to_string() } else { format!("问题：{}", question) }}</small>
                                                    <input node_ref=join_answer placeholder="输入答案"/>
                                                </div>
                                            </Show>
                                        </div>
                                        <div class="result-actions">
                                            <Show
                                                when=move || is_in_group
                                                fallback=move || view! {
                                                    <Show
                                                        when=move || !has_apply
                                                    >
                                                            <button type="button" on:click=move |_| {
                                                                let code = join_code.get().map(|el| el.value()).unwrap_or_default();
                                                                let answer = join_answer.get().map(|el| el.value()).unwrap_or_default();
                                                                apply_join_with_payload(room_id, join_type, code, answer, Some(after_join), status);
                                                            }>{group_join_button_text(join_type)}</button>
                                                    </Show>
                                                }
                                            >
                                                <button type="button" on:click=move |_| {
                                                    open_group_result.run(Group {
                                                        room_id,
                                                        room_name: Some(open_room_name.get_value()),
                                                        intro: None,
                                                        unread_count: None,
                                                        join_type: 1,
                                                        owner_name: None,
                                                        member_count: None,
                                                        is_in_group: true,
                                                        has_apply: false,
                                                    });
                                                }>"进入群组"</button>
                                            </Show>
                                            <button class="ghost" type="button" on:click=move |_| clear_result.set(None)>"关闭"</button>
                                            <button class="ghost" type="button" on:click=move |_| report_group.run((room_id, room_name_value.clone()))>"举报"</button>
                                        </div>
                                    </div>
                                }
                            })}
                        </Show>
                        <Show when=move || found_user.get().is_some()>
                            {move || found_user.get().map(|user| {
                                let add_uid = user.uid;
                                let nickname = user.nickname.clone();
                                let username = user.username.clone();
                                let avatar = user.avatar.clone();
                                let state_text = friend_state_text(&user);
                                let can_add = user.can_add_friend;
                                let clear_result = found_user;
                                let refresh_home = refresh;
                                view! {
                                    <div class="search-result">
                                        <Avatar src=avatar label=nickname.clone()/>
                                        <div class="row-main">
                                            <strong>{nickname}</strong>
                                            <small>{format!("@{}  ID {}", username, add_uid)}</small>
                                            <small>{state_text}</small>
                                        </div>
                                        <button class="ghost" type="button" on:click=move |_| open_user.run(add_uid)>"查看资料"</button>
                                        <Show
                                            when=move || can_add
                                            fallback=move || view! {
                                                <button class="ghost" type="button" on:click=move |_| clear_result.set(None)>"关闭"</button>
                                            }
                                        >
                                            <button type="button" on:click=move |_| send_friend_request(add_uid, refresh_home, status)>"添加好友"</button>
                                        </Show>
                                    </div>
                                }
                            })}
                        </Show>
                    </div>
                </section>
            </aside>
        </div>
    }
}

#[component]
fn FriendAlerts(
    home: RwSignal<HomeData>,
    my_uid: i64,
    refresh: Callback<()>,
    status: RwSignal<String>,
) -> impl IntoView {
    view! {
        <Show
            when=move || !home.get().requests.is_empty() || !home.get().deleted.is_empty()
            fallback=move || view!{ <Empty text="暂无好友请求"/> }
        >
            <div class="list compact">
                <For
                    each=move || home.get().requests
                    key=|req| req.id
                    children=move |req| {
                        let agree = req.id;
                        let refuse = req.id;
                        view! {
                            <div class="notice-row">
                                <Avatar src=req.avatar.clone() label=req.nickname.clone().unwrap_or_default()/>
                                <div class="row-main">
                                    <strong>{req.nickname.unwrap_or_else(|| "未知用户".into())}</strong>
                                    <small>{req.content.unwrap_or_else(|| if req.kind == Some(2) { "请求恢复好友关系".into() } else { "请求添加好友".into() })}</small>
                                </div>
                                <button on:click=move |_| handle_friend_request(agree, "agree", refresh, status)>"同意"</button>
                                <button class="danger" on:click=move |_| handle_friend_request(refuse, "refuse", refresh, status)>"拒绝"</button>
                            </div>
                        }
                    }
                />
                <For
                    each=move || home.get().deleted
                    key=|notice| notice.friend_id
                    children=move |notice| {
                        let friend_id = notice.friend_id;
                        let direct = notice.delete_by.unwrap_or_default() == my_uid;
                        view! {
                            <div class="notice-row">
                                <Avatar src=notice.avatar.clone() label=notice.nickname.clone().unwrap_or_default()/>
                                <div class="row-main">
                                    <strong>{notice.nickname.unwrap_or_else(|| "未知用户".into())}</strong>
                                    <small>{notice.delete_time.unwrap_or_else(|| "好友关系发生变化".into())}</small>
                                </div>
                                <button on:click=move |_| recover_friend(friend_id, direct, refresh, status)>"恢复"</button>
                            </div>
                        }
                    }
                />
            </div>
        </Show>
    }
}

#[component]
fn PublicGroupsView(
    groups: RwSignal<Vec<Group>>,
    refresh: Callback<()>,
    report_group: Callback<(i64, String)>,
    status: RwSignal<String>,
) -> impl IntoView {
    view! {
        <section class="workspace full public-groups-page">
            <div class="section-head">
                <div>
                    <h2>"公开群组"</h2>
                    <span>{move || format!("{} 个可加入群组", groups.get().len())}</span>
                </div>
                <button class="ghost" on:click=move |_| refresh.run(())>"重新加载"</button>
            </div>
            <div class="card-grid">
                <Show when=move || !groups.get().is_empty() fallback=move || view!{ <Empty text="暂无公开群组"/> }>
                    <For
                        each=move || groups.get()
                        key=|group| group.room_id
                        children=move |group| {
                            let room_id = group.room_id;
                            let room_name = group.room_name.clone().unwrap_or_else(|| "未命名群组".into());
                            let intro = group
                                .intro
                                .clone()
                                .filter(|value| !value.trim().is_empty())
                                .unwrap_or_else(|| "这个群组还没有简介。".into());
                            let owner_name = group.owner_name.clone().unwrap_or_default();
                            let member_count = group.member_count;
                            let join_type = group.join_type;
                            let is_in_group = group.is_in_group;
                            let has_apply = group.has_apply;
                            let has_owner_name = !owner_name.trim().is_empty();
                            view! {
                                <article class="group-card public-group-card">
                                    <div class="group-card-top">
                                        <div class="room-icon large">"#"</div>
                                        <span>{group_join_state_text(is_in_group, has_apply, join_type)}</span>
                                    </div>
                                    <h3>{room_name.clone()}</h3>
                                    <p>{intro}</p>
                                    <div class="group-meta-line">
                                        <span>{join_type_text(join_type)}</span>
                                        <Show when=move || member_count.is_some()>
                                            <span>{move || format!("{} 人", member_count.unwrap_or_default())}</span>
                                        </Show>
                                        <Show when=move || has_owner_name>
                                            <span>{format!("群主 {}", owner_name.clone())}</span>
                                        </Show>
                                    </div>
                                    <div class="card-foot">
                                        <span>{format!("ID {}", room_id)}</span>
                                        <div class="button-row compact-actions">
                                            <button on:click=move |_| open_public_group_detail(room_id, status)>"查看 / 加入"</button>
                                            <button class="ghost" on:click=move |_| report_group.run((room_id, room_name.clone()))>"举报"</button>
                                        </div>
                                    </div>
                                </article>
                            }
                        }
                    />
                </Show>
            </div>
        </section>
    }
}

#[component]
fn NoticesView(
    notices: RwSignal<Vec<Notice>>,
    refresh: Callback<()>,
    status: RwSignal<String>,
) -> impl IntoView {
    view! {
        <section class="workspace full">
            <div class="section-head">
                <h2>"系统通知"</h2>
                <div class="button-row">
                    <button class="ghost" on:click=move |_| refresh.run(())>"刷新"</button>
                    <button on:click=move |_| mark_all_read(refresh, status)>"全部已读"</button>
                </div>
            </div>
            <div class="list">
                <Show when=move || !notices.get().is_empty() fallback=move || view!{ <Empty text="暂无通知"/> }>
                    <For
                        each=move || notices.get()
                        key=|notice| notice.id
                        children=move |notice| {
                            let id = notice.id;
                            let expanded = RwSignal::new(false);
                            let title = notice.title.unwrap_or_else(|| "通知".into());
                            let content = notice.content.unwrap_or_default();
                            let time = notice.add_time.unwrap_or_default();
                            view! {
                                <article class=("notice-line notice-detail", move || notice.is_read.unwrap_or_default() == 0)>
                                    <div class="row-main">
                                        <strong>{title}</strong>
                                        <small>{time}</small>
                                        <Show when=move || expanded.get()>
                                            <LinkifiedText text=content.clone()/>
                                        </Show>
                                    </div>
                                    <div class="notice-actions">
                                        <button class="ghost" on:click=move |_| expanded.update(|value| *value = !*value)>
                                            {move || if expanded.get() { "收起" } else { "查看详情" }}
                                        </button>
                                        <button class="ghost" on:click=move |_| mark_notice_read(id, refresh, status)>"已读"</button>
                                    </div>
                                </article>
                            }
                        }
                    />
                </Show>
            </div>
        </section>
    }
}

#[component]
fn AccountView(
    me: RwSignal<Option<User>>,
    refresh: Callback<()>,
    status: RwSignal<String>,
) -> impl IntoView {
    let nickname = NodeRef::<Input>::new();
    let avatar = NodeRef::<Input>::new();
    let old_password = NodeRef::<Input>::new();
    let new_password = NodeRef::<Input>::new();
    let confirm_password = NodeRef::<Input>::new();

    let save_nickname = move |ev: SubmitEvent| {
        ev.prevent_default();
        let value = nickname
            .get()
            .map(|el| el.value().trim().to_string())
            .unwrap_or_default();
        if value.is_empty() {
            status.set("显示名称不能为空".into());
            return;
        }
        spawn_local(async move {
            status.set("正在更新显示名称...".into());
            match api_post(
                "/user/update_profile",
                json!({ "action": "nickname", "nickname": value }),
            )
            .await
            {
                Ok(data) if is_success(&data) => {
                    status.set(message_of(&data, "显示名称已更新"));
                    refresh.run(());
                }
                Ok(data) => status.set(message_of(&data, "显示名称更新失败")),
                Err(err) => status.set(err),
            }
        });
    };

    let upload_avatar = move |ev: Event| {
        let Some(input) = ev
            .target()
            .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
        else {
            return;
        };
        let Some(file) = input.files().and_then(|files| files.get(0)) else {
            return;
        };
        status.set("正在读取头像...".into());
        read_file_base64(
            file,
            5 * 1024 * 1024,
            "头像不能超过 5MB",
            "头像读取失败",
            Callback::new(move |result: Result<PendingFile, String>| match result {
                Ok(file) => {
                    spawn_local(async move {
                        status.set("正在上传头像...".into());
                        match upload_avatar_api(AvatarUploadRequest {
                            filename: file.filename,
                            mime: file.mime,
                            data_base64: file.data_base64,
                        })
                        .await
                        {
                            Ok(data) if is_success(&data) => {
                                status.set(message_of(&data, "头像已更新"));
                                refresh.run(());
                                if let Some(el) = avatar.get() {
                                    el.set_value("");
                                }
                            }
                            Ok(data) => status.set(message_of(&data, "头像上传失败")),
                            Err(err) => status.set(err),
                        }
                    });
                }
                Err(err) => status.set(err),
            }),
        );
    };

    let change_password = move |ev: SubmitEvent| {
        ev.prevent_default();
        let old_value = old_password.get().map(|el| el.value()).unwrap_or_default();
        let new_value = new_password.get().map(|el| el.value()).unwrap_or_default();
        let confirm_value = confirm_password
            .get()
            .map(|el| el.value())
            .unwrap_or_default();
        if old_value.is_empty() || new_value.is_empty() || confirm_value.is_empty() {
            status.set("请填写完整密码信息".into());
            return;
        }
        spawn_local(async move {
            status.set("正在修改密码...".into());
            match api_post(
                "/user/update_profile",
                json!({
                    "action": "password",
                    "old_password": old_value,
                    "new_password": new_value,
                    "confirm_password": confirm_value,
                }),
            )
            .await
            {
                Ok(data) if is_success(&data) => {
                    status.set(message_of(&data, "密码已更新为 SHA-256 存储"));
                    if let Some(el) = old_password.get() {
                        el.set_value("");
                    }
                    if let Some(el) = new_password.get() {
                        el.set_value("");
                    }
                    if let Some(el) = confirm_password.get() {
                        el.set_value("");
                    }
                    refresh.run(());
                }
                Ok(data) => status.set(message_of(&data, "密码修改失败")),
                Err(err) => status.set(err),
            }
        });
    };

    view! {
        <section class="workspace full account-page">
            <div class="section-head">
                <h2>"账户管理"</h2>
                <span>"资料修改会同步到原站账户"</span>
            </div>
            <div class="account-grid">
                <div class="account-profile">
                    {move || me.get().map(|user| {
                        let label = user.nickname.clone().unwrap_or_else(|| "我".into());
                        view! {
                            <>
                                <Avatar src=user.avatar.clone() label=label.clone()/>
                                <div>
                                    <h3>{label}</h3>
                                    <p>{format!("#{}", user.uid)}</p>
                                </div>
                            </>
                        }
                    })}
                </div>

                <form class="settings-card" on:submit=save_nickname>
                    <div>
                        <h3>"显示名称"</h3>
                        <p>"最长 16 个字符，好友和群聊中会显示这个名称。"</p>
                    </div>
                    <input
                        node_ref=nickname
                        maxlength="16"
                        placeholder=move || me.get().and_then(|user| user.nickname).unwrap_or_else(|| "新的显示名称".into())
                    />
                    <button type="submit">"保存名称"</button>
                </form>

                <div class="settings-card">
                    <div>
                        <h3>"头像"</h3>
                        <p>"支持 JPEG、PNG、GIF、WebP、BMP，最大 5MB。"</p>
                    </div>
                    <input node_ref=avatar type="file" accept="image/*" on:change=upload_avatar/>
                </div>

                <form class="settings-card" on:submit=change_password>
                    <div>
                        <h3>"修改密码"</h3>
                        <p>"会验证旧密码。旧 MD5 账户修改成功后将升级为 SHA-256。"</p>
                    </div>
                    <input node_ref=old_password type="password" autocomplete="current-password" placeholder="当前密码"/>
                    <input node_ref=new_password type="password" autocomplete="new-password" placeholder="新密码，至少 6 位"/>
                    <input node_ref=confirm_password type="password" autocomplete="new-password" placeholder="再次输入新密码"/>
                    <button type="submit">"更新密码"</button>
                </form>
            </div>
        </section>
    }
}

#[component]
fn AboutView() -> impl IntoView {
    view! {
        <section class="workspace full about-page">
            <div class="about-header">
                <div class="about-logo">
                    <img src="/icon/favicon.ico" alt="CsAC"/>
                </div>
                <div>
                    <h2>"CsAC Client"</h2>
                    <a href="https://github.com/VasilyZa/CsAC_Client" target="_blank" rel="noreferrer">
                        "https://github.com/VasilyZa/CsAC_Client"
                    </a>
                </div>
            </div>

            <div class="about-section">
                <h3>"软件"</h3>
                <dl class="about-list">
                    <div>
                        <dt>"版本号："</dt>
                        <dd>"v0.3.0"</dd>
                    </div>
                    <div>
                        <dt>"构建框架："</dt>
                        <dd>"Tauri + Leptos + Rust"</dd>
                    </div>
                    <div>
                        <dt>"客户端类型："</dt>
                        <dd>"CsAC 桌面聊天客户端"</dd>
                    </div>
                    <div>
                        <dt>"运行模式："</dt>
                        <dd>"UniCsAC 统一 API"</dd>
                    </div>
                </dl>
            </div>

            <div class="about-section">
                <h3>"项目"</h3>
                <dl class="about-list">
                    <div>
                        <dt>"许可证："</dt>
                        <dd>"Apache License 2.0"</dd>
                    </div>
                    <div>
                        <dt>"源码仓库："</dt>
                        <dd>
                            <a href="https://github.com/VasilyZa/CsAC_Client" target="_blank" rel="noreferrer">
                                "GitHub / VasilyZa / CsAC_Client"
                            </a>
                        </dd>
                    </div>
                    <div>
                        <dt>"项目定位："</dt>
                        <dd>"为 CsAC 网站提供现代化桌面客户端体验"</dd>
                    </div>
                </dl>
            </div>
        </section>
    }
}

#[component]
fn UserDetailView(
    uid: i64,
    data: RwSignal<Option<UserDetailData>>,
    my_uid: i64,
    back: Callback<()>,
    refresh: Callback<i64>,
    open_private: Callback<(i64, String)>,
    open_group: Callback<Group>,
    report_user: Callback<ReportTarget>,
    status: RwSignal<String>,
) -> impl IntoView {
    view! {
        <section class="workspace full profile-page">
            <div class="section-head">
                <div>
                    <h2>"用户详情"</h2>
                    <span>{format!("UID {uid}")}</span>
                </div>
                <div class="button-row">
                    <button class="ghost" on:click=move |_| back.run(())>"返回工作台"</button>
                    <button class="ghost" on:click=move |_| refresh.run(uid)>"刷新资料"</button>
                </div>
            </div>

            <Show
                when=move || data.get().is_some()
                fallback=move || view! { <Empty text="正在加载用户资料"/> }
            >
                {move || data.get().map(|detail| {
                    let user = detail.user.clone();
                    let target_uid = user.uid;
                    let is_self = user.is_self || target_uid == my_uid;
                    let nickname = user.nickname.clone().unwrap_or_else(|| format!("用户 {target_uid}"));
                    let username = user.username.clone().unwrap_or_default();
                    let remark = user.remark.clone().unwrap_or_default();
                    let has_remark = !remark.trim().is_empty();
                    let online = strip_html(user.online_status.as_deref().unwrap_or(""));
                    let state_text = user_profile_state_text(&user);
                    let groups_store = StoredValue::new(detail.created_groups.clone());
                    let groups_count = groups_store.get_value().len();
                    let add_refresh = Callback::new(move |_| refresh.run(target_uid));
                    let chat_button = if !is_self && user.is_friend {
                        let chat_name = nickname.clone();
                        view! {
                            <button type="button" on:click=move |_| open_private.run((target_uid, chat_name.clone()))>"发送消息"</button>
                        }.into_any()
                    } else {
                        view! { <></> }.into_any()
                    };
                    let add_button = if !is_self && user.can_add_friend {
                        view! {
                            <button type="button" on:click=move |_| send_friend_request(target_uid, add_refresh, status)>"添加好友"</button>
                        }.into_any()
                    } else {
                        view! { <></> }.into_any()
                    };
                    let report_button = if !is_self {
                        let report_target = ReportTarget::User {
                            uid: target_uid,
                            username: username.clone(),
                            nickname: nickname.clone(),
                        };
                        view! {
                            <button class="danger" type="button" on:click=move |_| report_user.run(report_target.clone())>"举报用户"</button>
                        }.into_any()
                    } else {
                        view! { <></> }.into_any()
                    };
                    view! {
                        <div class="profile-grid">
                            <aside class="profile-hero">
                                <div class="profile-avatar">
                                    <Avatar src=user.avatar.clone() label=nickname.clone()/>
                                </div>
                                <div>
                                    <span class="eyebrow">{if is_self { "My Profile" } else { "User Profile" }}</span>
                                    <h2>{nickname.clone()}</h2>
                                    <p>{if username.trim().is_empty() { format!("UID {target_uid}") } else { format!("@{} · UID {}", username, target_uid) }}</p>
                                </div>
                                <div class="profile-actions">
                                    {chat_button}
                                    {add_button}
                                    {report_button}
                                </div>
                            </aside>

                            <div class="profile-main">
                                <section class="profile-card">
                                    <div class="section-head">
                                        <h2>"基础资料"</h2>
                                        <span>{state_text}</span>
                                    </div>
                                    <div class="detail-list">
                                        <div><span>"显示名称"</span><strong>{nickname.clone()}</strong></div>
                                        <div><span>"账号"</span><strong>{if username.trim().is_empty() { "未公开".to_string() } else { username.clone() }}</strong></div>
                                        <div><span>"用户 ID"</span><strong>{target_uid}</strong></div>
                                        <div><span>"在线状态"</span><strong>{if online.trim().is_empty() { "未知".to_string() } else { online.clone() }}</strong></div>
                                        <Show when=move || has_remark>
                                            <div><span>"好友备注"</span><strong>{remark.clone()}</strong></div>
                                        </Show>
                                    </div>
                                    <Show when=move || user.request_sent || user.request_received || user.is_blocked>
                                        <p class="inline-note">{state_text}</p>
                                    </Show>
                                </section>

                                <section class="profile-card profile-groups">
                                    <div class="section-head">
                                        <h2>{if is_self { "我创建的群组" } else { "TA 创建的群组" }}</h2>
                                        <span>{format!("{} 个", groups_count)}</span>
                                    </div>
                                    <Show
                                        when=move || !groups_store.get_value().is_empty()
                                        fallback=move || view! { <Empty text="暂无创建的群组"/> }
                                    >
                                        <div class="card-grid profile-group-grid">
                                            <For
                                                each=move || groups_store.get_value()
                                                key=|group| group.room_id
                                                children=move |group| {
                                                    let room_id = group.room_id;
                                                    let room_name = group.room_name.clone().unwrap_or_else(|| format!("群组 {room_id}"));
                                                    let intro = group.intro.clone().filter(|value| !value.trim().is_empty()).unwrap_or_else(|| "这个群组还没有简介。".into());
                                                    let group_action = if is_self {
                                                        let group_item = group.clone();
                                                        view! {
                                                            <button on:click=move |_| open_group.run(group_item.clone())>"进入"</button>
                                                        }.into_any()
                                                    } else {
                                                        view! {
                                                            <button class="ghost" on:click=move |_| open_public_group_detail(room_id, status)>"查看"</button>
                                                        }.into_any()
                                                    };
                                                    view! {
                                                        <article class="group-card profile-group-card">
                                                            <div class="group-card-top">
                                                                <div class="room-icon">"#"</div>
                                                                <span>{join_type_text(group.join_type)}</span>
                                                            </div>
                                                            <h3>{room_name.clone()}</h3>
                                                            <p>{intro}</p>
                                                            <div class="card-foot">
                                                                <span>{format!("ID {}", room_id)}</span>
                                                                {group_action}
                                                            </div>
                                                        </article>
                                                    }
                                                }
                                            />
                                        </div>
                                    </Show>
                                </section>
                            </div>
                        </div>
                    }
                })}
            </Show>
        </section>
    }
}

#[component]
fn ReportView(target: ReportTarget, back: Callback<()>, status: RwSignal<String>) -> impl IntoView {
    let reason = NodeRef::<Textarea>::new();
    let anonymous = NodeRef::<Select>::new();
    let target_store = StoredValue::new(target.clone());
    let (kind, title, meta) = report_target_display(&target);

    let submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let reason_value = reason
            .get()
            .map(|el| el.value().trim().to_string())
            .unwrap_or_default();
        if reason_value.chars().count() < 10 {
            status.set("举报原因至少需要 10 个字符".into());
            return;
        }
        let anonymous_value = anonymous
            .get()
            .map(|el| el.value())
            .unwrap_or_else(|| "0".into());
        let target = target_store.get_value();
        let payload = match target {
            ReportTarget::User {
                uid,
                username,
                nickname,
            } => json!({
                "type": "user",
                "uid": uid,
                "username": username,
                "nickname": nickname,
                "reason": reason_value,
                "anonymous": anonymous_value,
            }),
            ReportTarget::Group { room_id, room_name } => json!({
                "type": "group",
                "rid": room_id,
                "room_name": room_name,
                "reason": reason_value,
                "anonymous": anonymous_value,
            }),
        };
        spawn_local(async move {
            status.set("正在提交举报...".into());
            match api_post("/report/submit_report", payload).await {
                Ok(data) if is_success(&data) => {
                    status.set(message_of(&data, "举报已提交"));
                    if let Some(el) = reason.get() {
                        el.set_value("");
                    }
                }
                Ok(data) => status.set(message_of(&data, "举报提交失败")),
                Err(err) => status.set(err),
            }
        });
    };

    view! {
        <section class="workspace full report-page">
            <div class="section-head">
                <div>
                    <h2>"举报中心"</h2>
                    <span>"提交后将交由网站管理员处理"</span>
                </div>
                <button class="ghost" on:click=move |_| back.run(())>"返回工作台"</button>
            </div>
            <div class="report-shell">
                <aside class="report-card report-target-card">
                    <span class="eyebrow">"Report Target"</span>
                    <h3>{title}</h3>
                    <p>{kind}</p>
                    <div class="meta-list">
                        <span>{meta}</span>
                    </div>
                </aside>
                <form class="report-card report-form" on:submit=submit>
                    <div class="report-warning">
                        <strong>"举报须知"</strong>
                        <p>"请如实填写举报原因。实名举报会记录你的用户身份，匿名举报不会向处理视图显示你的身份。"</p>
                    </div>
                    <label>"举报原因"
                        <textarea node_ref=reason rows="7" placeholder="请详细描述问题，至少 10 个字符。"></textarea>
                    </label>
                    <label>"举报方式"
                        <select node_ref=anonymous>
                            <option value="0">"实名举报"</option>
                            <option value="1">"匿名举报"</option>
                        </select>
                    </label>
                    <div class="button-row">
                        <button class="danger" type="submit">"提交举报"</button>
                        <button class="ghost" type="button" on:click=move |_| back.run(())>"取消"</button>
                    </div>
                </form>
            </div>
        </section>
    }
}

#[component]
fn GroupManageView(
    room_id: i64,
    room_name: String,
    data: RwSignal<Option<GroupManageData>>,
    my_uid: i64,
    back: Callback<(i64, String)>,
    refresh: Callback<(i64, String)>,
    home: Callback<()>,
    open_user: Callback<i64>,
    report_group: Callback<(i64, String)>,
    status: RwSignal<String>,
) -> impl IntoView {
    let name_input = NodeRef::<Input>::new();
    let intro_input = NodeRef::<Textarea>::new();
    let notice_input = NodeRef::<Textarea>::new();
    let join_type_input = NodeRef::<Select>::new();
    let fixed_code_input = NodeRef::<Input>::new();
    let question_input = NodeRef::<Input>::new();
    let answer_input = NodeRef::<Input>::new();
    let show_in_list_input = NodeRef::<Select>::new();
    let allow_invite_input = NodeRef::<Select>::new();
    let transfer_uid_input = NodeRef::<Input>::new();
    let room_name_store = StoredValue::new(room_name.clone());

    let refresh_now = move || refresh.run((room_id, room_name_store.get_value()));

    let save_name = move |ev: SubmitEvent| {
        ev.prevent_default();
        let value = name_input
            .get()
            .map(|el| el.value().trim().to_string())
            .unwrap_or_default();
        edit_group_info(
            room_id,
            "name",
            value,
            refresh,
            room_name_store.get_value(),
            status,
        );
    };

    let save_intro = move |ev: SubmitEvent| {
        ev.prevent_default();
        let value = intro_input
            .get()
            .map(|el| el.value().trim().to_string())
            .unwrap_or_default();
        edit_group_info(
            room_id,
            "intro",
            value,
            refresh,
            room_name_store.get_value(),
            status,
        );
    };

    let save_notice = move |ev: SubmitEvent| {
        ev.prevent_default();
        let value = notice_input
            .get()
            .map(|el| el.value().trim().to_string())
            .unwrap_or_default();
        edit_group_info(
            room_id,
            "notice",
            value,
            refresh,
            room_name_store.get_value(),
            status,
        );
    };

    let save_join_settings = move |ev: SubmitEvent| {
        ev.prevent_default();
        let join_type = join_type_input
            .get()
            .and_then(|el| el.value().parse::<i64>().ok())
            .unwrap_or(1);
        let fixed_code = fixed_code_input
            .get()
            .map(|el| el.value().trim().to_string())
            .unwrap_or_default();
        let question = question_input
            .get()
            .map(|el| el.value().trim().to_string())
            .unwrap_or_default();
        let answer = answer_input
            .get()
            .map(|el| el.value().trim().to_string())
            .unwrap_or_default();
        let show_in_list = show_in_list_input
            .get()
            .and_then(|el| el.value().parse::<i64>().ok())
            .unwrap_or(1);
        let allow_invite = allow_invite_input
            .get()
            .and_then(|el| el.value().parse::<i64>().ok())
            .unwrap_or(1);
        update_group_settings(
            room_id,
            join_type,
            fixed_code,
            question,
            answer,
            show_in_list,
            allow_invite,
            refresh,
            room_name_store.get_value(),
            status,
        );
    };

    let reset_invite = move |_| {
        group_simple_action(
            "/group/reset_invite_code",
            json!({ "room_id": room_id }),
            "邀请码已重置",
            refresh,
            room_name_store.get_value(),
            status,
        );
    };

    let transfer_owner = move |ev: SubmitEvent| {
        ev.prevent_default();
        let target_uid = transfer_uid_input
            .get()
            .and_then(|el| el.value().trim().parse::<i64>().ok())
            .unwrap_or_default();
        if target_uid <= 0 || target_uid == my_uid {
            status.set("请输入有效的接收人 UID".into());
            return;
        }
        group_simple_action(
            "/group/transfer",
            json!({ "room_id": room_id, "target_uid": target_uid }),
            "群组转让申请已发送",
            refresh,
            room_name_store.get_value(),
            status,
        );
    };

    let disband = move |_| {
        group_simple_action(
            "/group/disband",
            json!({ "room_id": room_id }),
            "群组已提交解散",
            Callback::new(move |_| home.run(())),
            room_name_store.get_value(),
            status,
        );
    };

    view! {
        <section class="workspace full manage-page">
            <div class="manage-hero">
                <div>
                    <span class="eyebrow">"Group Control"</span>
                    <h2>{move || data.get().map(|info| info.room.room_name).unwrap_or_else(|| room_name_store.get_value())}</h2>
                    <p>{format!("群组 ID {room_id}")}</p>
                </div>
                <div class="manage-actions">
                    <button class="ghost" on:click=move |_| back.run((room_id, room_name_store.get_value()))>"返回群聊"</button>
                    <button class="ghost" on:click=move |_| refresh_now()>"刷新资料"</button>
                    <button class="ghost" on:click=move |_| report_group.run((room_id, room_name_store.get_value()))>"举报群组"</button>
                </div>
            </div>

            <Show
                when=move || data.get().is_some()
                fallback=move || view! { <Empty text="正在加载群组管理数据"/> }
            >
                {move || data.get().map(|info| {
                    let room = info.room.clone();
                    let is_owner = info.is_owner;
                    let is_admin = info.is_admin;
                    let can_manage = is_owner || is_admin;
                    let member_count = info.members.len();
                    let admins = info.members.iter().filter(|member| member.is_admin).count();
                    let muted = info.members.iter().filter(|member| member.is_muted).count();
                    let intro = if room.intro.trim().is_empty() { "这个群组还没有简介。".to_string() } else { room.intro.clone() };
                    let notice = if room.notice.trim().is_empty() { "暂无公告。".to_string() } else { room.notice.clone() };
                    let current_join_type = room.join_type.to_string();
                    let current_show = room.show_in_list.to_string();
                    let current_allow = room.allow_invite.to_string();
                    let invite_code = if info.can_view_invite && !room.invite_code.trim().is_empty() {
                        room.invite_code.clone()
                    } else if info.can_view_invite {
                        "当前没有可显示的邀请码".into()
                    } else {
                        "当前身份不可查看邀请码".into()
                    };
                    let members_store = StoredValue::new(info.members.clone());
                    let applications_store = StoredValue::new(info.applications.clone());
                    let application_error_store = StoredValue::new(info.application_error.clone());
                    let applications_count = applications_store.get_value().len();
                    let has_application_error = application_error_store.get_value().is_some();
                    let no_applications = applications_store.get_value().is_empty();
                    let fixed_code_text = if room.fixed_code.trim().is_empty() {
                        "未设置".to_string()
                    } else {
                        room.fixed_code.clone()
                    };
                    let ask_question_text = if room.ask_question.trim().is_empty() {
                        "未设置".to_string()
                    } else {
                        room.ask_question.clone()
                    };
                    let room_name_value = room.room_name.clone();
                    let intro_value = room.intro.clone();
                    let notice_value = room.notice.clone();
                    let fixed_code_value = room.fixed_code.clone();
                    let question_value = room.ask_question.clone();
                    view! {
                        <div class="manage-grid">
                            <aside class="manage-panel summary-panel">
                                <div class="room-icon large">"#"</div>
                                <h3>{room.room_name.clone()}</h3>
                                <p>{intro}</p>
                                <div class="summary-list">
                                    <span><strong>{member_count}</strong>"成员"</span>
                                    <span><strong>{admins}</strong>"管理"</span>
                                    <span><strong>{muted}</strong>"禁言"</span>
                                </div>
                                <div class="meta-list">
                                    <span>"群主："{room.owner_name.clone()}</span>
                                    <span>"我的权限："{role_text(is_owner, is_admin)}</span>
                                    <span>"加入方式："{join_type_text(room.join_type)}</span>
                                    <span>"公开展示："{switch_text(room.show_in_list)}</span>
                                    <span>"成员邀请："{switch_text(room.allow_invite)}</span>
                                </div>
                            </aside>

                            <div class="manage-main">
                                <section class="manage-panel">
                                    <div class="section-head">
                                        <h2>"群组资料"</h2>
                                        <span>{if can_manage { "可编辑" } else { "只读" }}</span>
                                    </div>
                                    <div class="info-surface">
                                        <div>
                                            <strong>"群公告"</strong>
                                            <p>{notice}</p>
                                        </div>
                                        <div>
                                            <strong>"当前邀请码"</strong>
                                            <p>{invite_code}</p>
                                        </div>
                                    </div>
                                    <Show
                                        when=move || can_manage
                                        fallback=move || view! {
                                            <p class="muted">"你当前不是群主或管理员，只能查看群组信息和成员列表。"</p>
                                        }
                                    >
                                        <div class="settings-grid">
                                            <form class="settings-card compact" on:submit=save_name>
                                                <div>
                                                    <h3>"群组名称"</h3>
                                                    <p>"用于群组列表和聊天标题。"</p>
                                                </div>
                                                <input node_ref=name_input maxlength="40" prop:value=room_name_value.clone()/>
                                                <button type="submit">"保存名称"</button>
                                            </form>
                                            <form class="settings-card compact" on:submit=save_intro>
                                                <div>
                                                    <h3>"群组简介"</h3>
                                                    <p>"展示在公开群组和搜索结果中。"</p>
                                                </div>
                                                <textarea node_ref=intro_input rows="4" prop:value=intro_value.clone()></textarea>
                                                <button type="submit">"保存简介"</button>
                                            </form>
                                            <form class="settings-card compact" on:submit=save_notice>
                                                <div>
                                                    <h3>"群公告"</h3>
                                                    <p>"进入管理页和群组详情时可见。"</p>
                                                </div>
                                                <textarea node_ref=notice_input rows="4" prop:value=notice_value.clone()></textarea>
                                                <button type="submit">"保存公告"</button>
                                            </form>
                                        </div>
                                    </Show>
                                </section>

                                <section class="manage-panel">
                                    <div class="section-head">
                                        <h2>"加入与可见性"</h2>
                                        <span>"兼容原站规则"</span>
                                    </div>
                                    <Show
                                        when=move || can_manage
                                        fallback=move || view! {
                                            <div class="info-surface">
                                                <div><strong>"加入方式"</strong><p>{join_type_text(room.join_type)}</p></div>
                                                <div><strong>"固定邀请码"</strong><p>{fixed_code_text.clone()}</p></div>
                                                <div><strong>"入群问题"</strong><p>{ask_question_text.clone()}</p></div>
                                            </div>
                                        }
                                    >
                                        <form class="settings-card settings-wide" on:submit=save_join_settings>
                                            <div class="field-grid">
                                                <label>"加入方式"
                                                    <select node_ref=join_type_input prop:value=current_join_type.clone()>
                                                        <option value="1">"直接加入"</option>
                                                        <option value="2">"自动轮换邀请码"</option>
                                                        <option value="3">"固定邀请码"</option>
                                                        <option value="4">"问答审核"</option>
                                                    </select>
                                                </label>
                                                <label>"公开群组"
                                                    <select node_ref=show_in_list_input prop:value=current_show.clone()>
                                                        <option value="1">"显示"</option>
                                                        <option value="0">"隐藏"</option>
                                                    </select>
                                                </label>
                                                <label>"成员邀请"
                                                    <select node_ref=allow_invite_input prop:value=current_allow.clone()>
                                                        <option value="1">"允许"</option>
                                                        <option value="0">"禁止"</option>
                                                    </select>
                                                </label>
                                                <label>"固定邀请码"
                                                    <input node_ref=fixed_code_input prop:value=fixed_code_value.clone() placeholder="固定邀请码模式使用"/>
                                                </label>
                                                <label>"入群问题"
                                                    <input node_ref=question_input prop:value=question_value.clone() placeholder="问答审核模式使用"/>
                                                </label>
                                                <label>"问题答案"
                                                    <input node_ref=answer_input placeholder="留空则不更新答案"/>
                                                </label>
                                            </div>
                                            <div class="button-row">
                                                <button type="submit">"保存设置"</button>
                                                <Show when=move || is_owner>
                                                    <button class="ghost" type="button" on:click=reset_invite>"重置邀请码"</button>
                                                </Show>
                                            </div>
                                        </form>
                                    </Show>
                                </section>

                                <section class="manage-panel">
                                    <div class="section-head">
                                        <h2>"成员管理"</h2>
                                        <span>{format!("{} 人", member_count)}</span>
                                    </div>
                                    <div class="member-list">
                                        <For
                                            each=move || members_store.get_value()
                                            key=|member| member.uid
                                            children=move |member| {
                                                let uid = member.uid;
                                                let can_operate_member = can_manage && uid != my_uid && !member.is_owner;
                                                let can_admin_member = is_owner && uid != my_uid && !member.is_owner;
                                                let is_muted = member.is_muted;
                                                let is_member_admin = member.is_admin;
                                                let member_name = if member.nickname.trim().is_empty() { format!("用户 {uid}") } else { member.nickname.clone() };
                                                let member_avatar = member.avatar.clone();
                                                view! {
                                                    <div class="member-row">
                                                        <button class="avatar-button" type="button" title="查看资料" on:click=move |_| open_user.run(uid)>
                                                            <Avatar src=member_avatar label=member_name.clone()/>
                                                        </button>
                                                        <button class="row-main row-open" type="button" on:click=move |_| open_user.run(uid)>
                                                            <div class="member-title">
                                                                <strong>{member_name}</strong>
                                                                <span>"#"{uid}</span>
                                                                <Show when=move || member.is_owner>
                                                                    <em class="role owner">"群主"</em>
                                                                </Show>
                                                                <Show when=move || member.is_admin && !member.is_owner>
                                                                    <em class="role admin">"管理"</em>
                                                                </Show>
                                                                <Show when=move || is_muted>
                                                                    <em class="role muted-role">"禁言"</em>
                                                                </Show>
                                                            </div>
                                                            <small>{format!("{} · {}", strip_html(&member.online_status), mute_text(member.mute_until, member.is_muted))}</small>
                                                        </button>
                                                        <Show when=move || can_operate_member>
                                                            <div class="member-actions">
                                                                <button class="ghost" on:click=move |_| group_member_action(room_id, uid, if is_muted { "unmute" } else { "mute" }, refresh, room_name_store.get_value(), status)>
                                                                    {if is_muted { "解禁" } else { "禁言" }}
                                                                </button>
                                                                <button class="danger" on:click=move |_| group_member_action(room_id, uid, "kick", refresh, room_name_store.get_value(), status)>"踢出"</button>
                                                                <Show when=move || can_admin_member>
                                                                    <button class="ghost" on:click=move |_| group_admin_action(room_id, uid, if is_member_admin { "remove" } else { "set" }, refresh, room_name_store.get_value(), status)>
                                                                        {if is_member_admin { "撤销管理" } else { "设为管理" }}
                                                                    </button>
                                                                </Show>
                                                            </div>
                                                        </Show>
                                                    </div>
                                                }
                                            }
                                        />
                                    </div>
                                </section>

                                <Show when=move || is_admin>
                                    <section class="manage-panel">
                                        <div class="section-head">
                                            <h2>"入群申请"</h2>
                                            <span>{format!("{} 条", applications_count)}</span>
                                        </div>
                                        <Show
                                            when=move || !has_application_error
                                            fallback=move || view! {
                                                <div class="empty subtle">
                                                    {application_error_store.get_value().unwrap_or_else(|| "入群申请列表加载失败。".into())}
                                                </div>
                                            }
                                        >
                                            <Show
                                                when=move || !no_applications
                                                fallback=move || view! { <Empty text="暂无待处理申请"/> }
                                            >
                                                <div class="list compact">
                                                    <For
                                                        each=move || applications_store.get_value()
                                                        key=|apply| apply.id
                                                        children=move |apply| {
                                                            let apply_id = apply.id;
                                                            view! {
                                                                <div class="notice-row">
                                                                    <Avatar src=None label=apply.nickname.clone()/>
                                                                    <div class="row-main">
                                                                        <strong>{if apply.nickname.trim().is_empty() { format!("用户 {}", apply.uid.unwrap_or_default()) } else { apply.nickname.clone() }}</strong>
                                                                        <small>{apply.apply_time.clone()}</small>
                                                                        <p class="inline-note">{if apply.answer_content.trim().is_empty() { "未填写申请内容".to_string() } else { apply.answer_content.clone() }}</p>
                                                                    </div>
                                                                    <button on:click=move |_| handle_group_apply(room_id, apply_id, "pass", refresh, room_name_store.get_value(), status)>"通过"</button>
                                                                    <button class="danger" on:click=move |_| handle_group_apply(room_id, apply_id, "refuse", refresh, room_name_store.get_value(), status)>"拒绝"</button>
                                                                </div>
                                                            }
                                                        }
                                                    />
                                                </div>
                                            </Show>
                                        </Show>
                                    </section>
                                </Show>

                                <Show when=move || is_owner>
                                    <section class="manage-panel danger-zone">
                                        <div class="section-head">
                                            <h2>"群主操作"</h2>
                                            <span>"谨慎处理"</span>
                                        </div>
                                        <div class="owner-actions">
                                            <form class="inline-form transfer-form" on:submit=transfer_owner>
                                                <input node_ref=transfer_uid_input placeholder="接收人 UID"/>
                                                <button class="ghost" type="submit">"发起转让"</button>
                                            </form>
                                            <button class="danger" on:click=disband>"解散群组"</button>
                                        </div>
                                    </section>
                                </Show>
                            </div>
                        </div>
                    }
                })}
            </Show>
        </section>
    }
}

#[component]
fn ChatView(
    kind: &'static str,
    target_id: i64,
    title: String,
    messages: RwSignal<Vec<Message>>,
    my_uid: i64,
    back: Callback<()>,
    manage: Option<Callback<(i64, String)>>,
    open_user: Callback<i64>,
    report_group: Option<Callback<(i64, String)>>,
    status: RwSignal<String>,
) -> impl IntoView {
    let input = NodeRef::<Textarea>::new();
    let image_input = NodeRef::<Input>::new();
    let voice_input = NodeRef::<Input>::new();
    let voice_duration_input = NodeRef::<Input>::new();
    let messages_el = NodeRef::<Div>::new();
    let pending_image = RwSignal::new(None::<PendingFile>);
    let pending_voice = RwSignal::new(None::<PendingFile>);
    let chat_title = title.clone();
    let manage_button = manage.map(|manage| {
        let room_name = chat_title.clone();
        view! {
            <button class="ghost" on:click=move |_| manage.run((target_id, room_name.clone()))>"群管理"</button>
        }
    });
    let report_button = report_group.map(|report| {
        let room_name = chat_title.clone();
        view! {
            <button class="ghost" on:click=move |_| report.run((target_id, room_name.clone()))>"举报群组"</button>
        }
    });

    let refresh = move |_| {
        load_chat(kind, target_id, messages, status);
    };

    Effect::new(move |_| {
        let _ = messages.get().len();
        scroll_messages_to_bottom(messages_el);
    });

    let send = move |ev: SubmitEvent| {
        ev.prevent_default();
        let content = input.get().map(|el| el.value()).unwrap_or_default();
        let image = pending_image.get_untracked();
        let voice = pending_voice.get_untracked();
        if content.trim().is_empty() && image.is_none() && voice.is_none() {
            return;
        }
        if image.is_some() && voice.is_some() {
            status.set("图片和语音请分开发送".into());
            return;
        }
        let path = if kind == "group" {
            "/message/send_group_msg"
        } else {
            "/message/send_private_msg"
        };
        let payload = if kind == "group" {
            json!({ "room_id": target_id, "content": content })
        } else {
            json!({ "friend_id": target_id, "content": content })
        };
        spawn_local(async move {
            let result = if let Some(file) = image {
                upload_chat_file_api(ChatFileUploadRequest {
                    kind: kind.to_string(),
                    target_id,
                    file_kind: "image".into(),
                    filename: file.filename,
                    mime: file.mime,
                    data_base64: file.data_base64,
                    duration: None,
                })
                .await
            } else if let Some(file) = voice {
                let duration = voice_duration_input
                    .get()
                    .and_then(|el| el.value().trim().parse::<i64>().ok())
                    .unwrap_or_default();
                upload_chat_file_api(ChatFileUploadRequest {
                    kind: kind.to_string(),
                    target_id,
                    file_kind: "voice".into(),
                    filename: file.filename,
                    mime: file.mime,
                    data_base64: file.data_base64,
                    duration: Some(duration),
                })
                .await
            } else {
                api_post(path, payload).await
            };
            match result {
                Ok(data) if is_success(&data) => {
                    if let Some(el) = input.get() {
                        el.set_value("");
                    }
                    if let Some(el) = image_input.get() {
                        el.set_value("");
                    }
                    if let Some(el) = voice_input.get() {
                        el.set_value("");
                    }
                    if let Some(el) = voice_duration_input.get() {
                        el.set_value("");
                    }
                    pending_image.set(None);
                    pending_voice.set(None);
                    load_chat(kind, target_id, messages, status);
                }
                Ok(data) => status.set(message_of(&data, "发送失败")),
                Err(err) => status.set(err),
            }
        });
    };

    let choose_image = move |ev: Event| {
        let Some(input_el) = ev
            .target()
            .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
        else {
            return;
        };
        let Some(file) = input_el.files().and_then(|files| files.get(0)) else {
            pending_image.set(None);
            return;
        };
        status.set("正在读取图片...".into());
        read_file_base64(
            file,
            5 * 1024 * 1024,
            "图片不能超过 5MB",
            "图片读取失败",
            Callback::new(move |result| match result {
                Ok(file) => {
                    pending_voice.set(None);
                    if let Some(el) = voice_input.get() {
                        el.set_value("");
                    }
                    pending_image.set(Some(file));
                    status.set("图片已选择，点击发送上传".into());
                }
                Err(err) => status.set(err),
            }),
        );
    };

    let choose_voice = move |ev: Event| {
        let Some(input_el) = ev
            .target()
            .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
        else {
            return;
        };
        let Some(file) = input_el.files().and_then(|files| files.get(0)) else {
            pending_voice.set(None);
            return;
        };
        status.set("正在读取语音...".into());
        read_file_base64(
            file,
            10 * 1024 * 1024,
            "语音不能超过 10MB",
            "语音读取失败",
            Callback::new(move |result| match result {
                Ok(file) => {
                    pending_image.set(None);
                    if let Some(el) = image_input.get() {
                        el.set_value("");
                    }
                    pending_voice.set(Some(file));
                    status.set("语音已选择，点击发送上传".into());
                }
                Err(err) => status.set(err),
            }),
        );
    };

    view! {
        <section class=if kind == "group" { "chat-layout group-chat" } else { "chat-layout private-chat" }>
            <div class="chat-header">
                <button class="ghost back-button" on:click=move |_| back.run(())>"返回"</button>
                <div class="chat-title-block">
                    <h2>{title}</h2>
                    <span>{if kind == "group" { format!("群组 ID {target_id}") } else { format!("用户 ID {target_id}") }}</span>
                </div>
                <div class="chat-actions">
                    {manage_button}
                    {report_button}
                    <button class="primary" on:click=refresh>"刷新消息"</button>
                </div>
            </div>
            <div class="messages" node_ref=messages_el>
                <Show when=move || !messages.get().is_empty() fallback=move || view!{ <Empty text="暂无消息"/> }>
                    <For
                        each=move || messages.get()
                        key=|msg| msg.id
                        children=move |msg| {
                            let sender = msg.from_uid.or(msg.uid).unwrap_or_default();
                            let is_me = sender == my_uid;
                            let class_name = if is_me { "message mine" } else { "message other" };
                            let sender_name = if is_me {
                                "我".to_string()
                            } else {
                                msg.nickname.clone().unwrap_or_else(|| format!("用户 {sender}"))
                            };
                            let time = message_time(&msg);
                            let avatar = msg.avatar.clone();
                            let show_alert = msg.is_mentioned.unwrap_or(false) || msg.reply_to_me.unwrap_or(false);
                            let recalled = msg.is_recalled.unwrap_or_default() != 0;
                            let read_state = if kind == "private" && is_me && !recalled {
                                let read = msg.is_read.unwrap_or_default() != 0;
                                let class_name = if read {
                                    "read-state read"
                                } else {
                                    "read-state unread"
                                };
                                view! {
                                    <em class=class_name>{if read { "已读" } else { "未读" }}</em>
                                }
                                .into_any()
                            } else {
                                view! { <></> }.into_any()
                            };
                            let image_url = msg.image_url.clone().filter(|url| !url.trim().is_empty());
                            let voice_url = msg.voice_url.clone().filter(|url| !url.trim().is_empty());
                            let content_url = msg.content.clone().filter(|value| looks_like_media_path(value));
                            let is_image = msg.msg_type == Some(2) || image_url.is_some() || content_url.is_some();
                            let is_voice = msg.msg_type == Some(3) || voice_url.is_some();
                            let image_url = absolute_media(image_url.or(content_url).unwrap_or_default());
                            let voice_url = absolute_media(voice_url.unwrap_or_default());
                            let voice_seconds = msg.duration.or(msg.voice_duration).unwrap_or_default();
                            let text = msg.content.clone().unwrap_or_default();
                            let body = if recalled {
                                view! { <p class="muted">"消息已撤回"</p> }.into_any()
                            } else if is_image {
                                view! {
                                    <a class="image-preview" href=image_url.clone() target="_blank" rel="noreferrer">
                                        <img src=image_url.clone() alt="聊天图片"/>
                                        <span>"打开原图"</span>
                                    </a>
                                }.into_any()
                            } else if is_voice {
                                let has_voice_url = !voice_url.trim().is_empty();
                                view! {
                                    <div class="voice-preview">
                                        <div>
                                            <strong>"语音消息"</strong>
                                            <span>{format!("{} 秒", voice_seconds.max(0))}</span>
                                        </div>
                                        <Show
                                            when=move || has_voice_url
                                            fallback=move || view! { <p>"语音文件地址为空"</p> }
                                        >
                                            <audio controls preload="metadata" src=voice_url.clone()></audio>
                                        </Show>
                                    </div>
                                }.into_any()
                            } else {
                                view! { <p>{text}</p> }.into_any()
                            };
                            let avatar_label = sender_name.clone();
                            let avatar_node = view! {
                                <button class="chat-avatar-button" type="button" title="查看发送人资料" on:click=move |_| {
                                    if sender > 0 {
                                        open_user.run(sender);
                                    }
                                }>
                                    <Avatar src=avatar label=avatar_label/>
                                </button>
                            }.into_any();
                            let bubble_node = view! {
                                <div class="bubble">
                                    <div class="msg-meta">
                                        <strong>{sender_name}</strong>
                                        <span>{time}</span>
                                        {read_state}
                                        <Show when=move || show_alert>
                                            <em>"提醒"</em>
                                        </Show>
                                    </div>
                                    {body}
                                </div>
                            }.into_any();
                            let message_body = if is_me {
                                view! { {bubble_node} {avatar_node} }.into_any()
                            } else {
                                view! { {avatar_node} {bubble_node} }.into_any()
                            };
                            view! {
                                <div class=class_name>
                                    {message_body}
                                </div>
                            }
                        }
                    />
                </Show>
            </div>
            <form class="composer" on:submit=send>
                <textarea node_ref=input rows="3" placeholder="输入消息，按发送同步到 UniCsAC"></textarea>
                <div class="composer-tools">
                    <label class="file-button">
                        "图片"
                        <input node_ref=image_input type="file" accept="image/jpeg,image/png,image/gif,image/webp,image/bmp" on:change=choose_image/>
                    </label>
                    <label class="file-button">
                        "语音"
                        <input node_ref=voice_input type="file" accept="audio/webm,audio/ogg,audio/mpeg,audio/wav,audio/mp4" on:change=choose_voice/>
                    </label>
                    <input class="duration-input" node_ref=voice_duration_input type="number" min="0" placeholder="秒"/>
                    <button class="primary" type="submit">"发送"</button>
                </div>
                <Show when=move || pending_image.get().is_some() || pending_voice.get().is_some()>
                    <div class="attachment-preview">
                        {move || {
                            if let Some(file) = pending_image.get() {
                                format!("待发送图片：{}", file.filename)
                            } else if let Some(file) = pending_voice.get() {
                                format!("待发送语音：{}", file.filename)
                            } else {
                                String::new()
                            }
                        }}
                        <button class="ghost" type="button" on:click=move |_| {
                            pending_image.set(None);
                            pending_voice.set(None);
                            if let Some(el) = image_input.get() {
                                el.set_value("");
                            }
                            if let Some(el) = voice_input.get() {
                                el.set_value("");
                            }
                        }>"清除"</button>
                    </div>
                </Show>
            </form>
        </section>
    }
}

#[component]
fn Avatar(src: Option<String>, label: String) -> impl IntoView {
    let initial = label.chars().next().unwrap_or('C').to_string();
    let src = src.map(absolute_media).unwrap_or_default();
    if src.is_empty() {
        view! { <div class="avatar"><span>{initial}</span></div> }.into_any()
    } else {
        view! { <div class="avatar"><img src=src alt="avatar"/></div> }.into_any()
    }
}

fn scroll_messages_to_bottom(messages_el: NodeRef<Div>) {
    if let Some(el) = messages_el.get() {
        el.set_scroll_top(el.scroll_height());
    }

    leptos::prelude::request_animation_frame(move || {
        if let Some(el) = messages_el.get() {
            el.set_scroll_top(el.scroll_height());
        }

        leptos::prelude::request_animation_frame(move || {
            if let Some(el) = messages_el.get() {
                el.set_scroll_top(el.scroll_height());
            }
        });
    });
}

#[component]
fn Unread(count: i64) -> impl IntoView {
    view! {
        <Show when=move || { count > 0 }>
            <span class="unread">{count}</span>
        </Show>
    }
}

#[component]
fn Empty(text: &'static str) -> impl IntoView {
    view! { <div class="empty">{text}</div> }
}

fn load_dark_mode() -> bool {
    web_sys::window()
        .and_then(|window| window.local_storage().ok().flatten())
        .and_then(|storage| storage.get_item("csac.dark_mode").ok().flatten())
        .map(|value| value == "1")
        .unwrap_or(false)
}

fn save_dark_mode(enabled: bool) {
    if let Some(storage) =
        web_sys::window().and_then(|window| window.local_storage().ok().flatten())
    {
        let _ = storage.set_item("csac.dark_mode", if enabled { "1" } else { "0" });
    }
}

fn read_file_base64(
    file: web_sys::File,
    max_size: usize,
    size_message: &'static str,
    read_message: &'static str,
    done: Callback<Result<PendingFile, String>>,
) {
    if file.size() as usize > max_size {
        done.run(Err(size_message.into()));
        return;
    }

    let filename = file.name();
    let mime = file.type_();
    let Ok(reader) = web_sys::FileReader::new() else {
        done.run(Err(read_message.into()));
        return;
    };
    let callback_reader = reader.clone();
    let callback = Closure::<dyn FnMut(_)>::new(move |_ev: web_sys::ProgressEvent| {
        let result = callback_reader
            .result()
            .ok()
            .and_then(|value| value.as_string())
            .ok_or_else(|| read_message.to_string())
            .and_then(|data_url| {
                data_url
                    .split_once(',')
                    .map(|(_, data)| data.to_string())
                    .ok_or_else(|| "文件数据格式异常".to_string())
            })
            .map(|data_base64| PendingFile {
                filename: filename.clone(),
                mime: mime.clone(),
                data_base64,
            });
        done.run(result);
    });
    reader.set_onload(Some(callback.as_ref().unchecked_ref()));
    callback.forget();

    if reader.read_as_data_url(&file).is_err() {
        done.run(Err(read_message.into()));
    }
}

#[component]
fn LinkifiedText(text: String) -> impl IntoView {
    view! {
        <p class="notice-content">
            {link_segments(&text)
                .into_iter()
                .map(|segment| match segment {
                    LinkSegment::Text(value) => view! { <span>{value}</span> }.into_any(),
                    LinkSegment::Url(value) => {
                        let href = value.clone();
                        view! {
                            <a href=href target="_blank" rel="noreferrer">{value}</a>
                        }
                        .into_any()
                    }
                })
                .collect_view()}
        </p>
    }
}

enum LinkSegment {
    Text(String),
    Url(String),
}

fn link_segments(text: &str) -> Vec<LinkSegment> {
    let mut segments = Vec::new();
    let mut rest = text;

    while let Some(index) = rest.find("http://").or_else(|| rest.find("https://")) {
        let (before, after) = rest.split_at(index);
        if !before.is_empty() {
            segments.push(LinkSegment::Text(before.to_string()));
        }

        let end = after
            .find(|ch: char| {
                ch.is_whitespace() || matches!(ch, '"' | '\'' | '<' | '>' | '，' | '。')
            })
            .unwrap_or(after.len());
        let (url, tail) = after.split_at(end);
        segments.push(LinkSegment::Url(url.to_string()));
        rest = tail;
    }

    if !rest.is_empty() {
        segments.push(LinkSegment::Text(rest.to_string()));
    }

    segments
}

async fn load_home_data() -> Result<(User, HomeData), String> {
    let info = api_get("/user/get_info", json!({})).await?;
    if !is_success(&info) {
        return Err("请先登录".to_string());
    }
    let user = serde_json::from_value(info.get("user").cloned().unwrap_or_default())
        .map_err(|_| "用户信息解析失败".to_string())?;
    let (friends, groups, notifications, requests, deleted) = join!(
        api_get("/user/get_friends", json!({})),
        api_get("/user/get_groups", json!({})),
        api_get("/user/get_notifications", json!({})),
        api_get("/friend/get_friend_requests", json!({})),
        api_get("/friend/get_deleted_notices", json!({}))
    );
    let friends = friends.unwrap_or_default();
    let groups = groups.unwrap_or_default();
    let notifications = notifications.unwrap_or_default();
    let requests = requests.unwrap_or_default();
    let deleted = deleted.unwrap_or_default();

    Ok((
        user,
        HomeData {
            friends: list_from_field(&friends, "friends"),
            groups: group_list_from_field(&groups, "groups"),
            requests: list_from_field(&requests, "requests"),
            deleted: list_from_field(&deleted, "notices"),
            total_unread: notifications
                .get("total_unread")
                .and_then(Value::as_i64)
                .unwrap_or_default(),
        },
    ))
}

async fn load_user_detail(uid: i64) -> Result<UserDetailData, String> {
    let info = api_get("/user/get_info", json!({ "uid": uid })).await?;
    if !is_success(&info) {
        return Err(message_of(&info, "用户资料加载失败"));
    }
    let user_value = info
        .get("user")
        .cloned()
        .ok_or_else(|| "用户资料响应缺少 user 字段".to_string())?;
    let user: UserProfile =
        serde_json::from_value(user_value).map_err(|_| "用户资料解析失败".to_string())?;
    let groups = match api_get("/user/get_created_groups", json!({ "uid": user.uid })).await {
        Ok(data) if is_success(&data) => group_list_from_field(&data, "groups"),
        _ => Vec::new(),
    };
    Ok(UserDetailData {
        user,
        created_groups: groups,
    })
}

fn open_group_chat(
    room_id: i64,
    name: String,
    messages: RwSignal<Vec<Message>>,
    page: RwSignal<Page>,
    status: RwSignal<String>,
) {
    messages.set(Vec::new());
    page.set(Page::GroupChat(room_id, name));
    load_chat("group", room_id, messages, status);
}

fn open_private_chat(
    friend_id: i64,
    name: String,
    messages: RwSignal<Vec<Message>>,
    page: RwSignal<Page>,
    status: RwSignal<String>,
) {
    messages.set(Vec::new());
    page.set(Page::PrivateChat(friend_id, name));
    load_chat("private", friend_id, messages, status);
}

fn clear_friend_unread(home: RwSignal<HomeData>, friend_id: i64) {
    home.update(|data| {
        if let Some(friend) = data
            .friends
            .iter_mut()
            .find(|friend| friend.friend_id == friend_id)
        {
            friend.unread_count = Some(0);
        }
    });
}

fn open_group_manage(
    room_id: i64,
    name: String,
    group_manage: RwSignal<Option<GroupManageData>>,
    page: RwSignal<Page>,
    status: RwSignal<String>,
) {
    page.set(Page::GroupManage(room_id, name.clone()));
    load_group_manage(room_id, name, group_manage, status);
}

fn load_group_manage(
    room_id: i64,
    room_name: String,
    group_manage: RwSignal<Option<GroupManageData>>,
    status: RwSignal<String>,
) {
    spawn_local(async move {
        status.set("正在加载群组管理...".into());
        let (info, members, applications) = join!(
            api_get("/group/get_group_view_info", json!({ "rid": room_id })),
            api_get("/group/get_members", json!({ "room_id": room_id })),
            api_get("/group/get_applications", json!({ "room_id": room_id }))
        );

        let info = match info {
            Ok(data) if is_success(&data) => data,
            Ok(data) => {
                status.set(message_of(&data, "群组资料加载失败"));
                return;
            }
            Err(err) => {
                status.set(err);
                return;
            }
        };

        let mut room: GroupManageRoom =
            serde_json::from_value(info.get("room").cloned().unwrap_or_default())
                .unwrap_or_default();
        if room.room_name.trim().is_empty() {
            room.room_name = room_name;
        }
        if room.owner_name.trim().is_empty() {
            room.owner_name = "未知".into();
        }

        let members = match members {
            Ok(data) if is_success(&data) => list_from_field(&data, "members"),
            Ok(data) => {
                status.set(message_of(&data, "成员列表加载失败"));
                Vec::new()
            }
            Err(err) => {
                status.set(err);
                Vec::new()
            }
        };

        let (applications, application_error) = match applications {
            Ok(data) if is_success(&data) => {
                let list = list_from_field(&data, "applications")
                    .into_iter()
                    .chain(list_from_field(&data, "applies"))
                    .chain(list_from_field(&data, "requests"))
                    .collect();
                (list, None)
            }
            Ok(data) => (
                Vec::new(),
                Some(message_of(&data, "入群申请列表加载失败。")),
            ),
            Err(err) => (Vec::new(), Some(err)),
        };

        group_manage.set(Some(GroupManageData {
            room,
            members,
            applications,
            application_error,
            is_in_group: info
                .get("is_in_group")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            is_owner: info
                .get("is_owner")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            is_admin: info
                .get("is_admin")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            can_view_invite: info
                .get("can_view_invite")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        }));
        status.set(String::new());
    });
}

fn load_chat(
    kind: &'static str,
    target_id: i64,
    messages: RwSignal<Vec<Message>>,
    status: RwSignal<String>,
) {
    spawn_local(async move {
        let result = if kind == "group" {
            api_get("/message/get_group_msg", json!({ "room_id": target_id })).await
        } else {
            api_get(
                "/message/get_private_msg",
                json!({ "friend_id": target_id, "last_id": 0 }),
            )
            .await
        };
        match result {
            Ok(data) if is_success(&data) => {
                messages.set(list_from_field(&data, "messages"));
                status.set(String::new());
                mark_chat_read(kind, target_id, &data).await;
            }
            Ok(data) => status.set(message_of(&data, "消息加载失败")),
            Err(err) => status.set(err),
        }
    });
}

async fn mark_chat_read(kind: &str, target_id: i64, data: &Value) {
    if kind == "group" {
        let last_id = data
            .get("messages")
            .and_then(Value::as_array)
            .and_then(|items| items.last())
            .and_then(|item| item.get("id"))
            .and_then(Value::as_i64)
            .unwrap_or_default();
        if last_id > 0 {
            let _ = api_post(
                "/message/mark_read",
                json!({ "room_id": target_id, "last_msg_id": last_id }),
            )
            .await;
        }
    } else {
        let _ = api_post("/message/mark_read", json!({ "friend_id": target_id })).await;
    }
}

fn handle_friend_request(
    id: i64,
    action: &'static str,
    refresh: Callback<()>,
    status: RwSignal<String>,
) {
    spawn_local(async move {
        match api_post(
            "/friend/handle_request",
            json!({ "request_id": id, "action": action }),
        )
        .await
        {
            Ok(data) if is_success(&data) => {
                status.set(message_of(&data, "已处理好友请求"));
                refresh.run(());
            }
            Ok(data) => status.set(message_of(&data, "处理失败")),
            Err(err) => status.set(err),
        }
    });
}

fn recover_friend(friend_id: i64, direct: bool, refresh: Callback<()>, status: RwSignal<String>) {
    spawn_local(async move {
        let payload = if direct {
            json!({ "friend_id": friend_id, "direct": "1" })
        } else {
            json!({ "friend_id": friend_id, "message": "希望恢复好友关系" })
        };
        match api_post("/friend/recover_friend", payload).await {
            Ok(data) if is_success(&data) => {
                status.set(message_of(&data, "已提交恢复请求"));
                refresh.run(());
            }
            Ok(data) => status.set(message_of(&data, "恢复失败")),
            Err(err) => status.set(err),
        }
    });
}

fn send_friend_request(uid: i64, refresh: Callback<()>, status: RwSignal<String>) {
    spawn_local(async move {
        status.set("正在发送好友请求...".into());
        match api_post(
            "/friend/send_request",
            json!({ "to_uid": uid, "message": "请求添加你为好友" }),
        )
        .await
        {
            Ok(data) if is_success(&data) => {
                status.set(message_of(&data, "好友请求已发送"));
                refresh.run(());
            }
            Ok(data) => status.set(message_of(&data, "好友请求发送失败")),
            Err(err) => status.set(err),
        }
    });
}

fn user_search_result(data: &Value) -> Option<UserSearchResult> {
    let user = data.get("user")?;
    Some(UserSearchResult {
        uid: user.get("uid").and_then(Value::as_i64).unwrap_or_default(),
        username: user
            .get("username")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        nickname: user
            .get("nickname")
            .and_then(Value::as_str)
            .unwrap_or("未命名用户")
            .to_string(),
        avatar: user
            .get("avatar")
            .and_then(Value::as_str)
            .map(|value| value.to_string()),
        is_friend: user
            .get("is_friend")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        request_sent: user
            .get("friend_request_sent")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        request_received: user
            .get("friend_request_received")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        can_add_friend: user
            .get("can_add_friend")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    })
}

fn group_search_result(data: &Value) -> Option<GroupSearchResult> {
    let room = data
        .get("room")
        .or_else(|| data.get("group"))
        .or_else(|| data.get("data").and_then(|data| data.get("room")))
        .or_else(|| data.get("data").and_then(|data| data.get("group")))?;
    Some(GroupSearchResult {
        room_id: room
            .get("id")
            .and_then(value_to_i64)
            .or_else(|| room.get("room_id").and_then(value_to_i64))
            .or_else(|| room.get("rid").and_then(value_to_i64))
            .or_else(|| room.get("group_id").and_then(value_to_i64))
            .unwrap_or_default(),
        room_name: room
            .get("room_name")
            .or_else(|| room.get("name"))
            .or_else(|| room.get("title"))
            .and_then(value_to_string)
            .unwrap_or_else(|| "未命名群组".to_string()),
        intro: room
            .get("intro")
            .or_else(|| room.get("description"))
            .or_else(|| room.get("desc"))
            .and_then(value_to_string)
            .unwrap_or_default(),
        notice: room
            .get("notice")
            .and_then(value_to_string)
            .unwrap_or_default(),
        owner_name: room
            .get("owner_name")
            .or_else(|| room.get("owner_nickname"))
            .and_then(value_to_string)
            .unwrap_or_else(|| "未知".to_string()),
        join_type: room.get("join_type").and_then(value_to_i64).unwrap_or(1),
        ask_question: room
            .get("ask_question")
            .or_else(|| room.get("question"))
            .and_then(value_to_string)
            .unwrap_or_default(),
        is_in_group: data
            .get("is_in_group")
            .or_else(|| room.get("is_in_group"))
            .and_then(value_to_bool)
            .unwrap_or(false),
        has_apply: data
            .get("has_apply")
            .or_else(|| room.get("has_apply"))
            .and_then(value_to_bool)
            .unwrap_or(false),
    })
}

fn public_group_list(data: &Value) -> Vec<Group> {
    if let Some(groups) = parse_group_vec(data) {
        return groups
            .into_iter()
            .filter(|group| group.room_id > 0)
            .collect();
    }

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
        if let Some(groups) = value_at_path(data, path).and_then(parse_group_vec) {
            return groups
                .into_iter()
                .filter(|group| group.room_id > 0)
                .collect();
        }
    }
    Vec::new()
}

fn parse_group_vec(value: &Value) -> Option<Vec<Group>> {
    let values = value.as_array()?;
    Some(values.iter().filter_map(group_from_value).collect())
}

fn group_list_from_field(data: &Value, field: &str) -> Vec<Group> {
    data.get(field)
        .and_then(parse_group_vec)
        .unwrap_or_default()
        .into_iter()
        .filter(|group| group.room_id > 0)
        .collect()
}

fn group_from_value(value: &Value) -> Option<Group> {
    let room_id = value
        .get("room_id")
        .and_then(value_to_i64)
        .or_else(|| value.get("id").and_then(value_to_i64))
        .or_else(|| value.get("rid").and_then(value_to_i64))
        .or_else(|| value.get("group_id").and_then(value_to_i64))
        .or_else(|| value.get("roomId").and_then(value_to_i64))?;
    let join_type = value
        .get("join_type")
        .and_then(value_to_i64)
        .filter(|value| *value > 0)
        .unwrap_or(1);
    Some(Group {
        room_id,
        room_name: value
            .get("room_name")
            .or_else(|| value.get("name"))
            .or_else(|| value.get("title"))
            .and_then(value_to_string)
            .filter(|value| !value.trim().is_empty()),
        intro: value
            .get("intro")
            .or_else(|| value.get("description"))
            .or_else(|| value.get("desc"))
            .and_then(value_to_string),
        unread_count: value.get("unread_count").and_then(value_to_i64),
        join_type,
        owner_name: value.get("owner_name").and_then(value_to_string),
        member_count: value.get("member_count").and_then(value_to_i64),
        is_in_group: value
            .get("is_in_group")
            .and_then(value_to_bool)
            .unwrap_or(false),
        has_apply: value
            .get("has_apply")
            .and_then(value_to_bool)
            .unwrap_or(false),
    })
}

fn value_at_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    Some(current)
}

fn open_public_group_detail(room_id: i64, status: RwSignal<String>) {
    spawn_local(async move {
        status.set("正在读取群组详情...".into());
        match api_get("/group/get_group_view_info", json!({ "rid": room_id })).await {
            Ok(data) if is_success(&data) => match group_search_result(&data) {
                Some(group) => {
                    if group.is_in_group {
                        status.set(format!(
                            "你已加入「{}」，可在我的群组中进入聊天",
                            group.room_name
                        ));
                    } else if group.has_apply {
                        status.set(format!("「{}」的入群申请正在等待审核", group.room_name));
                    } else if group.join_type == 1 {
                        apply_join_with_payload(
                            room_id,
                            group.join_type,
                            String::new(),
                            String::new(),
                            None,
                            status,
                        );
                    } else {
                        status.set(format!(
                            "「{}」{}，请在首页按群组 ID 搜索后填写验证信息",
                            group.room_name,
                            group_join_state_text(false, false, group.join_type)
                        ));
                    }
                }
                None => status.set("群组详情解析失败".into()),
            },
            Ok(data) => status.set(message_of(&data, "群组详情加载失败")),
            Err(err) => status.set(err),
        }
    });
}

fn friend_state_text(user: &UserSearchResult) -> &'static str {
    if user.is_friend {
        "已经是好友，可在好友列表进入私聊"
    } else if user.request_sent {
        "好友请求已发送，等待对方确认"
    } else if user.request_received {
        "对方已发来好友请求，请在好友通知中处理"
    } else if user.can_add_friend {
        "可以发送好友请求"
    } else {
        "暂时无法添加"
    }
}

fn user_profile_state_text(user: &UserProfile) -> &'static str {
    if user.is_self {
        "这是你自己的账号"
    } else if user.is_friend {
        "已经是好友"
    } else if user.request_sent {
        "好友请求已发送"
    } else if user.request_received {
        "对方已发来好友请求"
    } else if user.is_blocked {
        "当前无法添加"
    } else if user.can_add_friend {
        "可以发送好友请求"
    } else {
        "暂时无法添加"
    }
}

fn report_target_display(target: &ReportTarget) -> (&'static str, String, String) {
    match target {
        ReportTarget::User {
            uid,
            username,
            nickname,
        } => (
            "被举报对象：用户",
            nickname.clone(),
            if username.trim().is_empty() {
                format!("UID {uid}")
            } else {
                format!("@{} · UID {}", username, uid)
            },
        ),
        ReportTarget::Group { room_id, room_name } => (
            "被举报对象：群组",
            room_name.clone(),
            format!("群组 ID {room_id}"),
        ),
    }
}

fn group_join_state_text(is_in_group: bool, has_apply: bool, join_type: i64) -> &'static str {
    if is_in_group {
        "已加入"
    } else if has_apply {
        "等待审核"
    } else {
        match join_type {
            1 => "可直接加入",
            2 | 3 => "需要邀请码",
            4 => "需要回答问题",
            _ => "可申请加入",
        }
    }
}

fn group_join_button_text(join_type: i64) -> &'static str {
    match join_type {
        2 | 3 => "用邀请码加入",
        4 => "提交答案",
        _ => "加入群组",
    }
}

fn join_type_text(join_type: i64) -> &'static str {
    match join_type {
        1 => "直接加入",
        2 => "自动轮换邀请码",
        3 => "固定邀请码",
        4 => "问答审核",
        _ => "未知规则",
    }
}

fn role_text(is_owner: bool, is_admin: bool) -> &'static str {
    if is_owner {
        "群主"
    } else if is_admin {
        "管理员"
    } else {
        "成员"
    }
}

fn switch_text(value: i64) -> &'static str {
    if value == 0 {
        "关闭"
    } else {
        "开启"
    }
}

fn mute_text(mute_until: i64, is_muted: bool) -> String {
    if is_muted {
        if mute_until > 0 {
            format!("禁言至 {mute_until}")
        } else {
            "禁言中".into()
        }
    } else {
        "可发言".into()
    }
}

fn apply_join_with_payload(
    room_id: i64,
    join_type: i64,
    code: String,
    answer: String,
    refresh: Option<Callback<()>>,
    status: RwSignal<String>,
) {
    spawn_local(async move {
        let mut payload = json!({ "room_id": room_id });
        if join_type == 2 || join_type == 3 {
            if code.trim().is_empty() {
                status.set("请输入邀请码".into());
                return;
            }
            payload["code"] = json!(code.trim());
        }
        if join_type == 4 {
            payload["answer"] = json!(answer.trim());
        }
        match api_post("/group/apply_join", payload).await {
            Ok(data) if is_success(&data) => {
                status.set(message_of(&data, "已提交加入群组申请"));
                if let Some(refresh) = refresh {
                    refresh.run(());
                }
            }
            Ok(data) => status.set(message_of(&data, "加入群组失败")),
            Err(err) => status.set(err),
        }
    });
}

fn edit_group_info(
    room_id: i64,
    action: &'static str,
    value: String,
    refresh: Callback<(i64, String)>,
    room_name: String,
    status: RwSignal<String>,
) {
    if action == "name" && value.trim().is_empty() {
        status.set("群组名称不能为空".into());
        return;
    }
    spawn_local(async move {
        status.set("正在保存群组资料...".into());
        match api_post(
            "/group/edit_info",
            json!({ "room_id": room_id, "action": action, "value": value }),
        )
        .await
        {
            Ok(data) if is_success(&data) => {
                status.set(message_of(&data, "群组资料已更新"));
                refresh.run((room_id, room_name));
            }
            Ok(data) => status.set(message_of(&data, "群组资料保存失败")),
            Err(err) => status.set(err),
        }
    });
}

fn update_group_settings(
    room_id: i64,
    join_type: i64,
    fixed_code: String,
    question: String,
    answer: String,
    show_in_list: i64,
    allow_invite: i64,
    refresh: Callback<(i64, String)>,
    room_name: String,
    status: RwSignal<String>,
) {
    spawn_local(async move {
        status.set("正在保存群组设置...".into());
        let mut payload = json!({
            "room_id": room_id,
            "join_type": join_type,
            "show_in_list": show_in_list,
            "allow_invite": allow_invite
        });
        if !fixed_code.trim().is_empty() {
            payload["fixed_code"] = json!(fixed_code.trim());
        }
        if !question.trim().is_empty() {
            payload["question"] = json!(question.trim());
        }
        if !answer.trim().is_empty() {
            payload["answer"] = json!(answer.trim());
        }
        match api_post("/group/update_settings", payload).await {
            Ok(data) if is_success(&data) => {
                status.set(message_of(&data, "群组设置已更新"));
                refresh.run((room_id, room_name));
            }
            Ok(data) => status.set(message_of(&data, "群组设置保存失败")),
            Err(err) => status.set(err),
        }
    });
}

fn group_simple_action(
    path: &'static str,
    payload: Value,
    fallback: &'static str,
    refresh: Callback<(i64, String)>,
    room_name: String,
    status: RwSignal<String>,
) {
    let room_id = payload
        .get("room_id")
        .and_then(value_to_i64)
        .unwrap_or_default();
    spawn_local(async move {
        status.set("正在提交群组操作...".into());
        match api_post(path, payload).await {
            Ok(data) if is_success(&data) => {
                status.set(message_of(&data, fallback));
                refresh.run((room_id, room_name));
            }
            Ok(data) => status.set(message_of(&data, "群组操作失败")),
            Err(err) => status.set(err),
        }
    });
}

fn group_member_action(
    room_id: i64,
    target_uid: i64,
    action: &'static str,
    refresh: Callback<(i64, String)>,
    room_name: String,
    status: RwSignal<String>,
) {
    let (path, payload) = match action {
        "mute" => (
            "/group/mute_member",
            json!({
                "room_id": room_id,
                "target_uid": target_uid,
                "action": "mute",
                "minutes": 60
            }),
        ),
        "unmute" => (
            "/group/mute_member",
            json!({
                "room_id": room_id,
                "target_uid": target_uid,
                "action": "unmute"
            }),
        ),
        "kick" => (
            "/group/kick_member",
            json!({ "room_id": room_id, "target_uid": target_uid }),
        ),
        _ => {
            status.set("未知成员操作".into());
            return;
        }
    };
    group_simple_action(path, payload, "成员操作已完成", refresh, room_name, status);
}

fn group_admin_action(
    room_id: i64,
    target_uid: i64,
    action: &'static str,
    refresh: Callback<(i64, String)>,
    room_name: String,
    status: RwSignal<String>,
) {
    group_simple_action(
        "/group/set_admin",
        json!({ "room_id": room_id, "target_uid": target_uid, "action": action }),
        "管理员设置已更新",
        refresh,
        room_name,
        status,
    );
}

fn handle_group_apply(
    room_id: i64,
    apply_id: i64,
    action: &'static str,
    refresh: Callback<(i64, String)>,
    room_name: String,
    status: RwSignal<String>,
) {
    group_simple_action(
        "/group/handle_apply",
        json!({ "room_id": room_id, "apply_id": apply_id, "action": action }),
        "入群申请已处理",
        refresh,
        room_name,
        status,
    );
}

fn mark_notice_read(id: i64, refresh: Callback<()>, status: RwSignal<String>) {
    spawn_local(async move {
        match api_post("/user/mark_notice_read", json!({ "notice_id": id })).await {
            Ok(data) => status.set(message_of(&data, "已标记已读")),
            Err(err) => status.set(err),
        }
    });
    refresh.run(());
}

fn mark_all_read(refresh: Callback<()>, status: RwSignal<String>) {
    spawn_local(async move {
        match api_post("/user/mark_notice_read", json!({ "read_all": "1" })).await {
            Ok(data) => status.set(message_of(&data, "已全部标记为已读")),
            Err(err) => status.set(err),
        }
    });
    refresh.run(());
}

async fn api_get(path: &str, params: Value) -> Result<Value, String> {
    api_request("GET", path, params).await
}

async fn api_post(path: &str, params: Value) -> Result<Value, String> {
    api_request("POST", path, params).await
}

async fn upload_avatar_api(req: AvatarUploadRequest) -> Result<Value, String> {
    let args =
        serde_wasm_bindgen::to_value(&AvatarUploadArgs { req }).map_err(|err| err.to_string())?;
    let value = invoke("upload_avatar", args).await;
    let response: ApiResponse =
        serde_wasm_bindgen::from_value(value).map_err(|err| err.to_string())?;
    if response.status == 401 {
        return Err("登录已失效，请重新登录".into());
    }
    if response.status == 403 {
        return Err(message_of(&response.data, "账号不可用"));
    }
    Ok(response.data)
}

async fn upload_chat_file_api(req: ChatFileUploadRequest) -> Result<Value, String> {
    let args =
        serde_wasm_bindgen::to_value(&ChatFileUploadArgs { req }).map_err(|err| err.to_string())?;
    let value = invoke("upload_chat_file", args).await;
    let response: ApiResponse =
        serde_wasm_bindgen::from_value(value).map_err(|err| err.to_string())?;
    if response.status == 401 {
        return Err("登录已失效，请重新登录".into());
    }
    if response.status == 403 {
        return Err(message_of(&response.data, "账号不可用"));
    }
    Ok(response.data)
}

async fn api_request(method: &str, path: &str, params: Value) -> Result<Value, String> {
    let args = serde_wasm_bindgen::to_value(&InvokeArgs {
        req: ApiRequest {
            method: method.to_string(),
            path: path.to_string(),
            params,
        },
    })
    .map_err(|err| err.to_string())?;
    let value = invoke("api_request", args).await;
    let response: ApiResponse =
        serde_wasm_bindgen::from_value(value).map_err(|err| err.to_string())?;
    if response.status == 401 {
        return Err("登录已失效，请重新登录".into());
    }
    if response.status == 403 {
        return Err(message_of(&response.data, "账号不可用"));
    }
    Ok(response.data)
}

fn list_from_field<T>(data: &Value, field: &str) -> Vec<T>
where
    T: for<'de> Deserialize<'de>,
{
    data.get(field)
        .cloned()
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or_default()
}

fn is_success(data: &Value) -> bool {
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

fn message_of(data: &Value, fallback: &str) -> String {
    let message = data
        .get("message")
        .and_then(Value::as_str)
        .filter(|message| !message.trim().is_empty())
        .unwrap_or(fallback)
        .to_string();
    let Some(attempts) = data.get("attempts").and_then(Value::as_array) else {
        return message;
    };
    let detail = attempts
        .iter()
        .rev()
        .take(3)
        .filter_map(|attempt| {
            let endpoint = attempt.get("endpoint")?.as_str()?;
            let status = attempt
                .get("status")
                .and_then(Value::as_i64)
                .unwrap_or_default();
            let item_message = attempt
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("请求失败");
            Some(format!("{status} {endpoint}: {item_message}"))
        })
        .collect::<Vec<_>>()
        .join(" | ");
    if detail.is_empty() {
        message
    } else {
        format!("{message}。尝试过：{detail}")
    }
}

fn public_group_empty_message(data: &Value) -> String {
    let endpoint = data
        .get("endpoint")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty());
    let base = if is_success(data) {
        "公开群组接口返回成功，但列表为空"
    } else {
        "没有读取到公开群组"
    };
    let message = message_of(data, base);
    if let Some(endpoint) = endpoint {
        format!("{message}。最终来源：{endpoint}")
    } else {
        message
    }
}

fn absolute_media(path: String) -> String {
    if path.starts_with("http://") || path.starts_with("https://") || path.is_empty() {
        path
    } else {
        format!(
            "https://cschat.ccccocccc.cc/{}",
            path.trim_start_matches('/')
        )
    }
}

fn looks_like_media_path(value: &str) -> bool {
    let value = value.trim();
    if value.is_empty() || value == "[图片]" || value == "[语音]" {
        return false;
    }
    let lower = value.to_ascii_lowercase();
    (lower.starts_with("upload/")
        || lower.starts_with("uploads/")
        || lower.starts_with("/upload/")
        || lower.starts_with("/uploads/")
        || lower.starts_with("http://")
        || lower.starts_with("https://"))
        && matches!(
            lower.rsplit('.').next(),
            Some("jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp")
        )
}

fn page_title(page: &Page) -> String {
    match page {
        Page::Login => "登录".into(),
        Page::Register => "注册".into(),
        Page::Home => "工作台".into(),
        Page::PublicGroups => "公开群组".into(),
        Page::Notices => "通知中心".into(),
        Page::Account => "账户管理".into(),
        Page::About => "关于".into(),
        Page::UserDetail(_) => "用户详情".into(),
        Page::Report(_) => "举报中心".into(),
        Page::GroupChat(_, name) | Page::GroupManage(_, name) | Page::PrivateChat(_, name) => {
            name.clone()
        }
    }
}

fn page_subtitle(page: &Page) -> &'static str {
    match page {
        Page::Login | Page::Register => "连接到 UniCsAC 统一聊天服务",
        Page::Home => "好友、群组和请求集中管理",
        Page::PublicGroups => "浏览并申请加入公开群组",
        Page::Notices => "查看系统消息与未读提醒",
        Page::Account => "更新资料、头像和密码",
        Page::About => "版本、许可证和项目来源",
        Page::UserDetail(_) => "查看用户资料、好友关系和创建群组",
        Page::Report(_) => "向网站管理员提交用户或群组举报",
        Page::GroupChat(_, _) => "群聊消息",
        Page::GroupManage(_, _) => "群组资料、成员和权限设置",
        Page::PrivateChat(_, _) => "私聊消息",
    }
}

fn message_time(msg: &Message) -> String {
    msg.add_time
        .clone()
        .or_else(|| msg.created_at.map(|time| time.to_string()))
        .unwrap_or_default()
}

fn strip_html(input: &str) -> String {
    let mut output = String::new();
    let mut in_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => output.push(ch),
            _ => {}
        }
    }
    output
}
