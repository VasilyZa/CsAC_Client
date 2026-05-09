# CsAC Client

CsAC 桌面客户端，使用 Tauri + Leptos + Rust 构建。

## 功能

- 登录、注册和会话保持
- 好友、群组、通知、私聊和用户详情
- 添加好友、好友请求处理
- 私聊已读 / 未读回执
- 图片、语音消息发送与聊天内预览 / 播放
- 用户和群组举报
- 账户资料管理、头像上传、密码更新
- 浅色 / 深色模式
- 适配 UniCsAC 统一后端 API

## 本地开发

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk tauri-cli
cargo tauri dev
```

客户端请求只走统一入口：

```text
https://cschat.ccccocccc.cc/rpc/UniCsAC.php?route=...
https://csac.ccccocccc.cc/rpc/UniCsAC.php?route=...
```

旧的散文件入口、`web/rpc`、`rpc.php`、`x.php` 和 `.php` 后缀 route 不再作为客户端 fallback。

常用检查：

```bash
cargo check --manifest-path src-tauri/Cargo.toml
cargo check --target wasm32-unknown-unknown
```

## 构建 Windows 安装程序

Arch Linux 交叉编译依赖：

```bash
sudo pacman -S mingw-w64-gcc nsis
rustup target add x86_64-pc-windows-gnu
scripts/prepare-webview2-runtime.sh
cargo tauri build --target x86_64-pc-windows-gnu
```

NSIS 安装程序会内置 Microsoft WebView2 Fixed Version Runtime x64，不再依赖用户系统自带 WebView2 是否可用。

产物路径：

```text
target/x86_64-pc-windows-gnu/release/csac_client.exe
target/x86_64-pc-windows-gnu/release/bundle/nsis/
```

## GitHub Actions

仓库内置 Windows 打包 workflow。推送到 `main` 或手动运行 workflow 后，会生成并上传 Windows x64 NSIS 安装程序 artifact。
