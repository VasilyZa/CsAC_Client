use futures::join;
use leptos::ev::{Event, SubmitEvent};
use leptos::html::{Input, Textarea};
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
    GroupChat(i64, String),
    PrivateChat(i64, String),
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
    #[serde(alias = "id")]
    room_id: i64,
    room_name: Option<String>,
    intro: Option<String>,
    unread_count: Option<i64>,
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
    #[allow(dead_code)]
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

fn value_to_i64(value: &Value) -> Option<i64> {
    match value {
        Value::Number(value) => value.as_i64(),
        Value::String(value) => value.parse().ok(),
        Value::Bool(value) => Some(if *value { 1 } else { 0 }),
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

#[component]
pub fn App() -> impl IntoView {
    let page = RwSignal::new(Page::Login);
    let me = RwSignal::new(None::<User>);
    let home = RwSignal::new(HomeData::default());
    let public_groups = RwSignal::new(Vec::<Group>::new());
    let notices = RwSignal::new(Vec::<Notice>::new());
    let messages = RwSignal::new(Vec::<Message>::new());
    let found_user = RwSignal::new(None::<UserSearchResult>);
    let found_group = RwSignal::new(None::<GroupSearchResult>);
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
            match api_get("/group/get_public_list.php", json!({})).await {
                Ok(data) => {
                    public_groups.set(list_from_field(&data, "groups"));
                    status.set(String::new());
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
            match api_get("/user/get_notice_list.php", json!({})).await {
                Ok(data) => {
                    notices.set(list_from_field(&data, "notices"));
                    status.set(String::new());
                }
                Err(err) => status.set(err),
            }
            loading.set(false);
        });
    };

    let logout = move |_| {
        spawn_local(async move {
            let _ = api_post("/auth/logout.php", json!({})).await;
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
                        <button class="ghost wide" on:click=logout>"退出登录"</button>
                    </Show>
                </div>
            </aside>

            <main class="content">
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
                                open_private_chat(id, name, messages, page, status);
                            })
                            refresh=Callback::new(move |_| refresh_home(false))
                            status=status
                        />
                    }.into_any(),
                    Page::PublicGroups => view! {
                        <PublicGroupsView groups=public_groups refresh=Callback::new(move |_| open_public_groups()) status=status/>
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
                    Page::GroupChat(room_id, room_name) => view! {
                        <ChatView
                            kind="group"
                            target_id=room_id
                            title=room_name
                            messages=messages
                            my_uid=me.get().map(|u| u.uid).unwrap_or_default()
                            back=Callback::new(move |_| refresh_home(true))
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
                "/auth/login.php",
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
            match api_post("/auth/register.php", payload).await {
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
            match api_post("/group/create.php", json!({ "room_name": name })).await {
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
            match api_get("/group/get_group_view_info.php", json!({ "rid": id })).await {
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
            match api_get("/user/get_info.php", json!({ "uid": uid })).await {
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
        <div class="dashboard-grid">
            <section class="workspace dashboard-hero">
                <div>
                    <span class="eyebrow">"CsAC Desktop"</span>
                    <h2>"消息工作台"</h2>
                    <p>"集中查看好友、群组、通知和待处理请求。"</p>
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

            <section class="workspace">
                <div class="section-head">
                    <h2>"我的好友"</h2>
                    <span>{move || format!("{} 位", home.get().friends.len())}</span>
                </div>
                <div class="list">
                    <Show when=move || !home.get().friends.is_empty() fallback=move || view!{ <Empty text="还没有好友"/> }>
                        <For
                            each=move || home.get().friends
                            key=|friend| friend.friend_id
                            children=move |friend| {
                                let item = friend.clone();
                                view! {
                                    <button class="row-item" on:click=move |_| open_friend.run(item.clone())>
                                        <Avatar src=friend.avatar.clone() label=friend.display_name.clone().unwrap_or_default()/>
                                        <span class="row-main">
                                            <strong>{friend.display_name.unwrap_or_else(|| "未命名好友".into())}</strong>
                                            <small>{format!("@{} {}", friend.username.unwrap_or_default(), strip_html(friend.online_status.as_deref().unwrap_or("")))}</small>
                                        </span>
                                        <Unread count=friend.unread_count.unwrap_or_default()/>
                                    </button>
                                }
                            }
                        />
                    </Show>
                </div>
            </section>

            <section class="workspace">
                <div class="section-head">
                    <h2>"我的群组"</h2>
                    <span>{move || format!("{} 个", home.get().groups.len())}</span>
                </div>
                <div class="list">
                    <Show when=move || !home.get().groups.is_empty() fallback=move || view!{ <Empty text="还没有加入群组"/> }>
                        <For
                            each=move || home.get().groups
                            key=|group| group.room_id
                            children=move |group| {
                                let item = group.clone();
                                view! {
                                    <button class="row-item" on:click=move |_| open_group.run(item.clone())>
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
            </section>

            <section class="workspace tools">
                <div class="section-head">
                    <h2>"快速操作"</h2>
                    <span>"群组与好友"</span>
                </div>
                <div class="action-stack">
                    <form class="inline-form action-form" on:submit=create_group>
                        <div>
                            <strong>"创建群组"</strong>
                            <small>"输入名称后立即创建并加入"</small>
                        </div>
                        <input node_ref=room_name placeholder="新群组名称"/>
                        <button type="submit">"创建"</button>
                    </form>
                    <form class="inline-form action-form" on:submit=search_group>
                        <div>
                            <strong>"查找群组"</strong>
                            <small>"按群组编号搜索并申请加入"</small>
                        </div>
                        <input node_ref=group_id placeholder="群组编号"/>
                        <button type="submit">"查找"</button>
                    </form>
                    <form class="inline-form action-form" on:submit=search_user>
                        <div>
                            <strong>"查找用户"</strong>
                            <small>"按用户 ID 添加好友"</small>
                        </div>
                        <input node_ref=user_id placeholder="用户 ID"/>
                        <button type="submit">"查找"</button>
                    </form>
                </div>
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
                                            });
                                        }>"进入群组"</button>
                                    </Show>
                                    <button class="ghost" type="button" on:click=move |_| clear_result.set(None)>"关闭"</button>
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
            </section>

            <section class="workspace alerts">
                <div class="section-head"><h2>"好友通知"</h2></div>
                <FriendAlerts home=home my_uid=my_uid refresh=refresh status=status/>
            </section>
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
    status: RwSignal<String>,
) -> impl IntoView {
    view! {
        <section class="workspace full">
            <div class="section-head">
                <h2>"公开群组"</h2>
                <button class="ghost" on:click=move |_| refresh.run(())>"重新加载"</button>
            </div>
            <div class="card-grid">
                <Show when=move || !groups.get().is_empty() fallback=move || view!{ <Empty text="暂无公开群组"/> }>
                    <For
                        each=move || groups.get()
                        key=|group| group.room_id
                        children=move |group| {
                            let room_id = group.room_id;
                            view! {
                                <article class="group-card">
                                    <div class="room-icon large">"#"</div>
                                    <h3>{group.room_name.unwrap_or_else(|| "未命名群组".into())}</h3>
                                    <p>{group.intro.unwrap_or_else(|| "这个群组还没有简介。".into())}</p>
                                    <div class="card-foot">
                                        <span>{format!("ID {}", room_id)}</span>
                                        <button on:click=move |_| apply_join(room_id, status)>"申请加入"</button>
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
                "/user/update_profile.php",
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
            Callback::new(move |result| match result {
                Ok(upload) => {
                    spawn_local(async move {
                        status.set("正在上传头像...".into());
                        match upload_avatar_api(upload).await {
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
                "/user/update_profile.php",
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
fn ChatView(
    kind: &'static str,
    target_id: i64,
    title: String,
    messages: RwSignal<Vec<Message>>,
    my_uid: i64,
    back: Callback<()>,
    status: RwSignal<String>,
) -> impl IntoView {
    let input = NodeRef::<Textarea>::new();

    let refresh = move |_| {
        load_chat(kind, target_id, messages, status);
    };

    let send = move |ev: SubmitEvent| {
        ev.prevent_default();
        let content = input.get().map(|el| el.value()).unwrap_or_default();
        if content.trim().is_empty() {
            return;
        }
        let path = if kind == "group" {
            "/message/send_group_msg.php"
        } else {
            "/message/send_private_msg.php"
        };
        let payload = if kind == "group" {
            json!({ "room_id": target_id, "content": content })
        } else {
            json!({ "friend_id": target_id, "content": content })
        };
        spawn_local(async move {
            match api_post(path, payload).await {
                Ok(data) if is_success(&data) => {
                    if let Some(el) = input.get() {
                        el.set_value("");
                    }
                    load_chat(kind, target_id, messages, status);
                }
                Ok(data) => status.set(message_of(&data, "发送失败")),
                Err(err) => status.set(err),
            }
        });
    };

    view! {
        <section class="chat-layout">
            <div class="chat-header">
                <button class="ghost" on:click=move |_| back.run(())>"返回"</button>
                <div>
                    <h2>{title}</h2>
                    <span>{if kind == "group" { format!("群组 ID {target_id}") } else { format!("用户 ID {target_id}") }}</span>
                </div>
                <button on:click=refresh>"刷新消息"</button>
            </div>
            <div class="messages">
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
                            let show_alert = msg.is_mentioned.unwrap_or(false) || msg.reply_to_me.unwrap_or(false);
                            let recalled = msg.is_recalled.unwrap_or_default() != 0;
                            let image_url = msg.image_url.clone().filter(|url| !url.trim().is_empty());
                            let voice_url = msg.voice_url.clone().filter(|url| !url.trim().is_empty());
                            let content_url = msg.content.clone().filter(|value| looks_like_media_path(value));
                            let is_image = msg.msg_type == Some(2) || image_url.is_some() || content_url.is_some();
                            let is_voice = msg.msg_type == Some(3) || voice_url.is_some();
                            let image_url = absolute_media(image_url.or(content_url).unwrap_or_default());
                            let voice_seconds = msg.duration.or(msg.voice_duration).unwrap_or_default();
                            let text = msg.content.clone().unwrap_or_default();
                            let body = if recalled {
                                view! { <p class="muted">"消息已撤回"</p> }.into_any()
                            } else if is_image {
                                view! { <a class="image-link" href=image_url target="_blank">"查看图片"</a> }.into_any()
                            } else if is_voice {
                                view! { <p>"[语音] "{voice_seconds}" 秒"</p> }.into_any()
                            } else {
                                view! { <p>{text}</p> }.into_any()
                            };
                            view! {
                                <div class=class_name>
                                    <div class="bubble">
                                        <div class="msg-meta">
                                            <strong>{sender_name}</strong>
                                            <span>{time}</span>
                                            <Show when=move || show_alert>
                                                <em>"提醒"</em>
                                            </Show>
                                        </div>
                                        {body}
                                    </div>
                                </div>
                            }
                        }
                    />
                </Show>
            </div>
            <form class="composer" on:submit=send>
                <textarea node_ref=input rows="3" placeholder="输入消息。当前桌面版支持文字消息，图片和语音可在原站继续使用。"></textarea>
                <button class="primary" type="submit">"发送"</button>
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

fn read_file_base64(file: web_sys::File, done: Callback<Result<AvatarUploadRequest, String>>) {
    if file.size() as usize > 5 * 1024 * 1024 {
        done.run(Err("头像不能超过 5MB".into()));
        return;
    }

    let filename = file.name();
    let mime = file.type_();
    let Ok(reader) = web_sys::FileReader::new() else {
        done.run(Err("无法读取头像文件".into()));
        return;
    };
    let callback_reader = reader.clone();
    let callback = Closure::<dyn FnMut(_)>::new(move |_ev: web_sys::ProgressEvent| {
        let result = callback_reader
            .result()
            .ok()
            .and_then(|value| value.as_string())
            .ok_or_else(|| "头像读取失败".to_string())
            .and_then(|data_url| {
                data_url
                    .split_once(',')
                    .map(|(_, data)| data.to_string())
                    .ok_or_else(|| "头像数据格式异常".to_string())
            })
            .map(|data_base64| AvatarUploadRequest {
                filename: filename.clone(),
                mime: mime.clone(),
                data_base64,
            });
        done.run(result);
    });
    reader.set_onload(Some(callback.as_ref().unchecked_ref()));
    callback.forget();

    if reader.read_as_data_url(&file).is_err() {
        done.run(Err("头像读取失败".into()));
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
    let info = api_get("/user/get_info.php", json!({})).await?;
    if !is_success(&info) {
        return Err("请先登录".to_string());
    }
    let user = serde_json::from_value(info.get("user").cloned().unwrap_or_default())
        .map_err(|_| "用户信息解析失败".to_string())?;
    let (friends, groups, notifications, requests, deleted) = join!(
        api_get("/user/get_friends.php", json!({})),
        api_get("/user/get_groups.php", json!({})),
        api_get("/user/get_notifications.php", json!({})),
        api_get("/friend/get_friend_requests.php", json!({})),
        api_get("/friend/get_deleted_notices.php", json!({}))
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
            groups: list_from_field(&groups, "groups"),
            requests: list_from_field(&requests, "requests"),
            deleted: list_from_field(&deleted, "notices"),
            total_unread: notifications
                .get("total_unread")
                .and_then(Value::as_i64)
                .unwrap_or_default(),
        },
    ))
}

fn open_group_chat(
    room_id: i64,
    name: String,
    messages: RwSignal<Vec<Message>>,
    page: RwSignal<Page>,
    status: RwSignal<String>,
) {
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
    page.set(Page::PrivateChat(friend_id, name));
    load_chat("private", friend_id, messages, status);
}

fn load_chat(
    kind: &'static str,
    target_id: i64,
    messages: RwSignal<Vec<Message>>,
    status: RwSignal<String>,
) {
    spawn_local(async move {
        let result = if kind == "group" {
            api_get(
                "/message/get_group_msg.php",
                json!({ "room_id": target_id }),
            )
            .await
        } else {
            api_get(
                "/message/get_private_msg.php",
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
                "/message/mark_read.php",
                json!({ "room_id": target_id, "last_msg_id": last_id }),
            )
            .await;
        }
    } else {
        let _ = api_post("/message/mark_read.php", json!({ "friend_id": target_id })).await;
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
            "/friend/handle_request.php",
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
        match api_post("/friend/recover_friend.php", payload).await {
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
            "/friend/send_request.php",
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
    let room = data.get("room")?;
    Some(GroupSearchResult {
        room_id: room
            .get("id")
            .and_then(value_to_i64)
            .or_else(|| room.get("room_id").and_then(value_to_i64))
            .unwrap_or_default(),
        room_name: room
            .get("room_name")
            .and_then(Value::as_str)
            .unwrap_or("未命名群组")
            .to_string(),
        intro: room
            .get("intro")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        notice: room
            .get("notice")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        owner_name: room
            .get("owner_name")
            .and_then(Value::as_str)
            .unwrap_or("未知")
            .to_string(),
        join_type: room
            .get("join_type")
            .and_then(value_to_i64)
            .unwrap_or_default(),
        ask_question: room
            .get("ask_question")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        is_in_group: data
            .get("is_in_group")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        has_apply: data
            .get("has_apply")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    })
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

fn apply_join(room_id: i64, status: RwSignal<String>) {
    apply_join_with_payload(room_id, 1, String::new(), String::new(), None, status);
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
        match api_post("/group/apply_join.php", payload).await {
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

fn mark_notice_read(id: i64, refresh: Callback<()>, status: RwSignal<String>) {
    spawn_local(async move {
        match api_post("/user/mark_notice_read.php", json!({ "notice_id": id })).await {
            Ok(data) => status.set(message_of(&data, "已标记已读")),
            Err(err) => status.set(err),
        }
    });
    refresh.run(());
}

fn mark_all_read(refresh: Callback<()>, status: RwSignal<String>) {
    spawn_local(async move {
        match api_post("/user/mark_notice_read.php", json!({ "read_all": "1" })).await {
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
    data.get("success")
        .and_then(Value::as_bool)
        .unwrap_or(false)
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
        Page::GroupChat(_, name) | Page::PrivateChat(_, name) => name.clone(),
    }
}

fn page_subtitle(page: &Page) -> &'static str {
    match page {
        Page::Login | Page::Register => "连接到 CsAC 原站服务",
        Page::Home => "好友、群组和请求集中管理",
        Page::PublicGroups => "浏览并申请加入公开群组",
        Page::Notices => "查看系统消息与未读提醒",
        Page::Account => "更新资料、头像和密码",
        Page::GroupChat(_, _) => "群聊消息",
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
