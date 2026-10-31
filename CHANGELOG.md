# Changelog

All notable changes to this project are documented here.
Format based on [Keep a Changelog](https://keepachangelog.com/), versioning follows [SemVer](https://semver.org/).

## [Unreleased]

### Added
- **pm.* assertion matchers**: `.not` negation, `lengthOf`/`length`, `property`, `a`/`an`, `match(/regex/)`,
  `oneOf`, `closeTo`, `include` (array membership), `empty`, `null`, `exist`, and aliases.
- **GUI Postman Console**: new sidebar section logging each request/response and test results (clear / copy all).
- **Request Description editor**: new "描述" sub-tab, saved to `item.description`, kept on v2.1 export.
- **Inheritance on send**: GUI now inherits collection/folder auth, folder variables and collection/folder
  pre-request/test scripts (Runner already did).
- **macOS packaging**: `.github/workflows/build.yml` (macos job → universal arm64+x86_64), `scripts/build-macos.sh`,
  source bundle `dist/pmrs-macos-src.tar.gz`, and `docs/13_macOS打包说明.md`.
- **Dynamic variables**: ~30 more (`$randomLorem*`, `$randomImageUrl`, `$randomAvatar`, `$randomUserName`,
  `$randomBankAccount*`, `$randomCreditCardNumber`, `$randomAmount`, …).
- **Extended HTTP method dropdown** (COPY/LINK/UNLINK/PURGE/LOCK/UNLOCK/PROPFIND/VIEW/TRACE).
- **Container image (GHCR)**: `Dockerfile` + `.github/workflows/docker.yml` build and push the CLI
  image to GitHub Packages (`ghcr.io/zhouyulong123/postman-rs`).
- **cURL 导出增强**：请求卡片新增「导出」按钮，代码生成面板新增「保存为文件」，可把当前请求
  导出为 cURL/各语言文件到 `~/.postman-rs/exports/`（原有「复制 cURL」「从 cURL 导入」保留）。
- **新建请求对话框**：新建标签时可命名并选择方法；`POST/PUT/PATCH` 自动带上默认请求头
  （`Content-Type: application/json`、`Accept: application/json`）与 JSON 请求体。
- **删除二次确认弹窗**：删除集合/请求/文件夹、关闭标签、删除环境/全局变量行前，均弹出确认框，
  确认后才执行（带半透明遮罩，点击遮罩=取消）。
- **环境/全局变量编辑**：键、值两个输入框各占左侧栏一半宽度；**请求头编辑**改为纯键/值输入框
  呈现（移除“启用”勾选框），默认请求头同样以输入框显示。
- `LICENSE-MIT`, `LICENSE-APACHE`, `CHANGELOG.md`, `.gitignore`.

### Fixed
- **GUI ignored global variables** when sending (`ctx_from` did not merge `globals`).
- `pm.response.json()` no longer collapses JSON `null` into empty string.
- **GUI 左侧栏黑边**：侧栏内多控件行（如「应用强调色/恢复默认」「监听条数/等待/连接并发送」）
  横向溢出，使面板分配宽度大于实际绘制宽度，露出窗口清屏黑底形成竖直黑边。现将侧栏各行改为
  `horizontal_wrapped` 自动换行，并为窗口设置主题背景 `clear_color`，彻底消除黑边。

## [0.1.0]

### Added
- Local, account-free, payment-free Postman-style API client in Rust (`pmrs` CLI + `pmrs-gui` egui desktop).
- HTTP requests (all methods), variables/scopes + dynamic variables, collections/folders/requests (v2.1),
  environments, globals, history.
- Collection Runner (iterations, CSV/JSON data, delay, assertions, CI exit code).
- Import: Postman v1/v2.1, OpenAPI 3 / Swagger 2, cURL, HAR. Export: Collection v2.1 / Environment.
- Code generation for 13 languages.
- Assertions: declarative + common `pm.test(...)` parsing.
- Protocols: WebSocket, SSE, MQTT (plain TCP), GraphQL, local Mock, performance test.
- Networking: proxy, CA bundle, client cert, insecure, persistent cookie jar.
- Auth: Bearer / Basic / API Key / OAuth2 / OAuth1 / JWT / AWS SigV4 / Digest (NTLM honest "unsupported").
