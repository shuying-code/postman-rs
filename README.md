# postman-rs (`pmrs`)

一个用 Rust 编写的 **本地化、无账号、无付费、无会员限制** 的 Postman 风格 API 客户端。
它复刻了 Postman 最常用的**本地**能力，采用 **Postman Collection v2.1** 兼容格式，跨平台单二进制（Windows / macOS / Linux）。

<p>
  <img alt="Rust" src="https://img.shields.io/badge/rust-1.80%2B-orange">
  <img alt="Platform" src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue">
  <img alt="License" src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green">
  <img alt="Postman format" src="https://img.shields.io/badge/format-Postman%20v2.1-ff6c37">
</p>

> 本项目不连接 Postman 云端，不含登录、订阅、试用、Stripe、遥测。**所有功能对任何人无条件可用。**

---

## 为什么「无限制」

Postman 的付费与登录主要约束**云端能力**（同步、团队、云端 Mock/Monitor、Vault、Governance、AI 等）。本工具只实现**不依赖云端**的本地能力，因此天然不需要账号、不需要付费、没有配额。设计上移除了登录/订阅/会员限制，所有本地功能无条件可用。

---

## 功能

- 任意 HTTP 方法发送；Header/Query/Body（raw/json/urlencoded/form-data/file/graphql）
- 变量：`{{name}}` 递归解析；作用域 `cli vars > iteration data > environment > collection > globals`
- 动态变量：`{{$randomInt}}`、`{{$randomUUID}}`、`{{$guid}}`、`{{$timestamp}}`、`{{$isoTimestamp}}`、`{{$randomAlphaNumeric}}`、`{{$randomBoolean}}`、`{{$randomFirstName}}`、`{{$randomLastName}}`、`{{$randomEmail}}`、`{{$randomPassword}}`、`{{$randomCity}}`、`{{$randomCountry}}`、`{{$randomPhoneNumber}}`、`{{$randomIP}}`、`{{$randomColor}}`、`{{$randomHexColor}}`、`{{$randomAbbreviation}}`
- 鉴权：Bearer / Basic / API Key（header 或 query）
- 集合 / 文件夹 / 请求 树；集合变量
- 环境 / 全局变量；历史记录
- Collection Runner：多迭代、CSV/JSON 数据驱动、延迟、断言统计、CI 退出码
- 代码生成：`curl` / `http` / `rust(reqwest)` / `python(requests)` / `js(fetch)` / `go(net/http)` / `nodejs(axios)` / `java(okhttp)` / `csharp` / `php` / `ruby` / `powershell` / `swift`
- 导入：Postman v2.1/v2.0/v1、OpenAPI 3/Swagger 2（JSON）、cURL、HAR
- 导出：Postman Collection v2.1 / Environment
- 断言：声明式（status / json_path / json_path_exists / body_contains / body_equals / header / max_time_ms / response_time / regex）+ 常见 `pm.test(...status(N))` / `to.have.header(...)` 解析
- 协议/进阶：WebSocket（`pmrs ws`）、SSE（`pmrs sse`）、MQTT（`pmrs mqtt`）、GraphQL（`pmrs graphql`）、本地 Mock 服务（`pmrs mock`）、性能压测（`pmrs perf`）
- 网络：代理（`PMRS_PROXY`）、CA 证书包（`PMRS_CA_BUNDLE`）、客户端证书（`PMRS_CLIENT_CERT`/`PMRS_CLIENT_KEY`）、跳过校验（`PMRS_INSECURE=1`）、连接超时与最大重定向、持久 Cookie jar（`~/.postman-rs/cookies.json`）
- OAuth2：client_credentials / password / refresh_token / accessToken

### 与 Postman 云端能力的差异（有意不实现）

- 完整 JS 脚本引擎（Hermes 的 `pm.*` 全量）——当前为声明式断言 + 常见形态解析
- Socket.IO / gRPC 工作台、云端同步、团队协作、云端 Mock/Monitor、Vault、Governance、AI 等需要账号或付费的云端能力

这些属于 Postman 的**云端/付费**能力；本工具刻意去除登录、订阅、试用、会员等限制，所有本地功能无条件可用。

## 图形界面（GUI，pmrs-gui）

除了命令行 `pmrs`，项目还提供一个基于 **egui/eframe** 的桌面窗口程序 `pmrs-gui`（纯 Rust、单 exe、无需 WebView、双击即用）：

- **中文界面**：自动加载系统中文字体（Windows 优先 `simhei/msyh/simsun`，macOS `PingFang`，Linux `Noto Sans CJK`），不会出现缺字方框
- 左侧：集合列表（可展开选择请求）、环境下拉、历史记录
- 右侧：**多标签**请求构建器（`＋ 新建标签` / `✕` 关闭，每个标签独立的 方法 / URL / Headers / Body / 响应）
- **请求体编辑器（齐全，对齐 Postman）**：类型可选 `无 / form-data 表单 / x-www-form-urlencoded 表单 / raw 原始 / binary 二进制 / GraphQL 查询`；raw 支持 `Text / JSON / XML / HTML / JavaScript` 子语言与一键格式化；表单类支持逐行启用/键/值/类型（文本或文件），GraphQL 有独立 Query 与 Variables 编辑框
- **全中文界面与统一输入尺寸**：菜单、标签、按钮、提示、请求体类型、授权类型（无 / Bearer 令牌 / 基本认证 / API 密钥）、导入格式等均为中文；键值行输入框按窗口宽度自适应，键列约 30%、值列占其余，行高与间距对齐 Postman
- **保存到集合**：把当前标签的请求存进指定/当前集合（集合名留空=当前集合，请求名留空=标签标题）
- **导出**：把当前集合导出为 Postman v2.1 JSON（导出路径留空=`~/.postman-rs/exports/`）
- 支持 `{{变量}}` 与环境下拉解析；发送在后台线程执行，界面不卡顿
- **环境编辑器**：左栏「环境」可增删改变量并保存
- **工具面板**：导入（Postman/OpenAPI/cURL/HAR，自动合并同名集合）、集合运行器（多迭代/失败即停）、代码生成（13 种语言）
- **响应面板**：响应体（JSON 语法高亮 + 美化 + 复制）/ 响应头 / Cookies / 测试（断言结果）/ 计时
- **Postman 控制台**：侧栏「控制台」分段记录每次请求/响应与脚本测试日志（可清空 / 复制全部）
- **请求描述**：请求区「描述」子标签，支持 Markdown，保存时写入 `item.description`（导出 v2.1 一并保留）
- **继承语义**：发送时自动继承集合/文件夹级鉴权、文件夹变量与集合/文件夹级 pre-request/test 脚本；全局变量在 GUI 发送时生效
- **更全的细节**：方法下拉 16 种；`pm.expect` 新增 `.not`/`lengthOf`/`property`/`a`/`match`/`oneOf`/`closeTo` 等匹配器；动态变量再补 ~30 个（`$randomLorem*` / `$randomImageUrl` / `$randomAvatar` 等）
- **8 套主题 + 渐变背景**：Postman 浅色 / Postman 深色 / 午夜蓝 / 石墨灰 / 暖沙（浅）/ 森林绿 / 樱粉（浅）/ 高对比；顶栏「主题」下拉即切，选择会自动记忆（`~/.postman-rs/gui_theme.txt`），也可用环境变量 `PMRS_THEME=<0-7>` 指定
- **统一设计系统**：圆角卡片 + 柔和阴影、下划线式标签页、分段式侧栏导航、顶部品牌渐变条、品牌色药丸按钮；深/浅主题下原生控件（下拉、输入框、复选框）均随主题一致着色
- Windows 为 **Windows GUI 子系统**（无黑色控制台窗口），静态链接 C 运行库

启动：

```powershell
F:\codex工作区\postman-rs\dist\pmrs-gui-windows-x64.exe
```

首次打开时，左栏若为空，是因为本地工作区还没有集合。可先导入示例：

```powershell
.\pmrs.exe import .\examples\demo_collection.json --format postman
.\pmrs.exe import .\examples\demo_environment.json --format postman
```

> 界面为中文；程序会按平台自动加载系统 CJK 字体，避免缺字方框。

---

## 构建

```bash
cd F:\codex工作区\postman-rs
cargo build --release
# 产物: target/release/pmrs(.exe)       —— 命令行
#       target/release/pmrs-gui(.exe)   —— 图形界面
```

打包到 `dist/`：

```powershell
Copy-Item target/release/pmrs.exe     dist/pmrs-windows-x64.exe -Force
Copy-Item target/release/pmrs-gui.exe dist/pmrs-gui-windows-x64.exe -Force
```

跨平台：Rust + reqwest(rustls) 无 OpenSSL 依赖，可在 Windows / macOS / Linux 直接编译。

## 预编译 / 跨平台产物

| 平台 | 产物 | 获取方式 |
|---|---|---|
| Windows x64 | `dist/pmrs-windows-x64.exe`（CLI，已生成）<br>`dist/pmrs-gui-windows-x64.exe`（GUI，已生成） | 本机 `cargo build --release` 后打包 |
| macOS（Universal：arm64 + x86_64） | `pmrs-macos-universal` + `pmrs-gui-macos-universal` | **必须在 macOS 上构建**：运行 `scripts/build-macos.sh`，或用 CI |
| Linux x64 | `pmrs-linux-x64` + `pmrs-gui-linux-x64` | 在 Linux 上 `cargo build --release`，或用 CI |

### 为什么 macOS 不能在这台 Windows 上直接编译

生成 macOS 可执行文件（Mach-O）需要 **macOS SDK + `ld64`/`clang` 交叉工具链**（osxcross 等，且 Apple SDK 有许可限制）。本机仅有 `x86_64-pc-windows-msvc` 目标、无 `clang(o64/oa64)` 与 `ld64`，因此无法在 Windows 上链接出真正的 macOS 二进制。macOS 的 Rust 程序需在 **Mac 真机** 上编译。

### 在 Mac 上生成（Apple Silicon 与 Intel 通用）

```bash
cd postman-rs
chmod +x scripts/build-macos.sh
./scripts/build-macos.sh
# 产物: dist/pmrs-macos-universal  （lipo 合并 arm64 + x86_64）
```

首次运行下载者若被 Gatekeeper 拦截：

```bash
xattr -d com.apple.quarantine ./pmrs-macos-universal
# 或右键 -> 打开；本项目已在构建时做 ad-hoc 签名（codesign -s -）
```

### 用 CI 自动产出三平台

仓库已包含 `.github/workflows/build.yml`：推送后自动在 `windows-latest` / `macos-latest` / `ubuntu-latest` 上构建，并上传 `pmrs-windows-x64.zip`、`pmrs-gui-windows-x64.zip`、`pmrs-macos-universal.tar.gz`、`pmrs-linux-x64.tar.gz`。

### 容器镜像（GitHub Packages / GHCR）

仓库包含 `Dockerfile` 与 `.github/workflows/docker.yml`：推送到 `main` 或打 `v*` 标签时，会自动构建 **CLI 镜像**并推送到 GitHub 容器注册表，可在 `Packages` 页面下载 `ghcr.io/zhouyulong123/postman-rs`。

```bash
# 拉取并运行 CLI
docker pull ghcr.io/zhouyulong123/postman-rs:latest
docker run --rm ghcr.io/zhouyulong123/postman-rs:latest send "https://httpbin.org/get" -m GET

# 挂载当前目录，方便读写集合 / 环境文件
docker run --rm -v "$PWD:/work" -w /work \
  ghcr.io/zhouyulong123/postman-rs:latest run examples/demo_collection.json -i 2
```

---

## 使用

### 1) 发送请求
```bash
pmrs send "https://httpbin.org/get" -m GET -H "Accept: application/json"
pmrs send "{{baseUrl}}/users" -m POST -e examples/demo_environment.json \
  --body-type json -d '{"name":"z"}'
```

### 2) 集合 / 环境
```bash
pmrs collection new "My API"
pmrs collection list
pmrs env new "dev"
pmrs env set <env.json> baseUrl https://api.test
pmrs export collection.json -o out.json
```

### 3) 导入
```bash
pmrs import examples/openapi.json --format openapi
pmrs import "curl 'https://api.test/x' -X POST -d '{\"a\":1}'" --format curl
pmrs import har.json --format har
pmrs import postman.json --format postman
```

### 4) 运行集合（Runner）
```bash
pmrs run examples/demo_collection.json -e examples/demo_environment.json -i 3
pmrs run collection.json -d data.csv --delay 200
```

### 5) 生成代码
```bash
pmrs codegen --collection examples/demo_collection.json --item "Get Hello" --lang rust
pmrs codegen --url https://api.test/x --method POST --lang curl
```

### 6) 历史
```bash
pmrs send https://httpbin.org/get --save-history
pmrs history list
pmrs history clear
```

---

## 工作区（本地）

默认目录 `~/.postman-rs`（可用环境变量 `PMRS_HOME` 覆盖）：
```
~/.postman-rs/
├─ collections/*.json
├─ environments/*.json
└─ history.jsonl
```
纯文件、可纳入 git、无数据库、无云端。

---

## 验收

```bash
cargo test          # 单元 + 集成（本地起 HTTP 服务器）
cargo build --release
```

集成测试 `tests/integration.rs` 会启动本地 HTTP 服务器并验证：
- 单请求发送（200 + 正文）；
- 集合 Runner 端到端（两个请求、断言全通过）。

「无条件」核验：业务代码中不存在 login/token/paywall/subscription/stripe/license 的强制分支。

---

## 常见问题（FAQ）

**Q：编译后的文件在哪里？**

- GUI：`F:\codex工作区\postman-rs\dist\pmrs-gui-windows-x64.exe`（`target\release\pmrs-gui.exe` 的副本）
- CLI：`F:\codex工作区\postman-rs\dist\pmrs-windows-x64.exe`
- 压缩包：`dist\pmrs-gui-windows-x64.zip`、`dist\pmrs-windows-x64.zip`

**Q：Windows 产物为什么“安装不了”？**

本项目是**免安装的绿色单文件程序**，不是 MSI/EXE 安装包，没有安装向导。直接**双击 `pmrs-gui-windows-x64.exe` 即可运行**，无需安装。如果系统提示“需要管理员权限/找不到安装程序”，那是把它误当成安装包了。若被 SmartScreen 拦截，点“更多信息 → 仍要运行”。

**Q：为什么之前没有图形界面？**

早期 `dist` 里只有命令行 `pmrs`。现在已提供基于 **egui/eframe** 的纯 Rust 桌面程序 `pmrs-gui`（编译为 Windows GUI 子系统，双击即开、无黑色控制台窗口），界面为中文。

**Q：怎么换主题 / 背景？**

顶栏右侧「主题」下拉可选 8 套配色（含深/浅），主内容区带纵向渐变背景；选择会自动记忆。也可用 `PMRS_THEME=0..7` 启动参数强制指定。列表顺序：0 Postman 浅色、1 Postman 深色、2 午夜蓝、3 石墨灰、4 暖沙、5 森林绿、6 樱粉、7 高对比。

**Q：macOS 版本怎么获得？**

macOS 的 Mach-O 二进制**无法在 Windows 上交叉编译**（需要 macOS SDK + ld64/clang）。在 Mac 真机上运行 `scripts/build-macos.sh` 生成 `pmrs-macos-universal` 与 `pmrs-gui-macos-universal`（arm64+x86_64 通用），或使用仓库内 CI（`.github/workflows/build.yml`）自动产出。

---

## 问题反馈（Issues）

遇到问题或有建议？欢迎通过 GitHub Issues 反馈：<https://github.com/zhouyulong123/postman-rs/issues/new/choose>。提交前请先阅读 **[使用反馈指南](docs/FEEDBACK.md)**。

- **🐞 Bug / 问题反馈**：可复现的功能异常、崩溃、报错
- **💡 功能建议**：新功能或改进
- **❓ 使用咨询 / 反馈**：用法、配置与一般性反馈

为便于**复现与修复**，请在报告中尽量提供：版本/构建号、运行方式（GUI / CLI / 容器）、系统与架构、**复现步骤**、预期与实际结果、**日志 / 控制台输出**、以及**最小复现样例**（Collection / 环境 JSON，或一条 cURL 命令）、出现频率与网络环境（代理 / 证书）、截图或录屏等。

> 维护者通常**每周集中处理 / 回复一次** issue（约每周更新）；同一问题请勿重复提交，补充信息请在原 issue 追加评论。

---

## CLI 命令参考

`pmrs` 提供 16 组子命令（`pmrs --help` 或 `pmrs <cmd> --help` 查看全部参数）：

| 命令 | 作用 | 常用参数 |
|---|---|---|
| `send <URL>` | 发送单条请求 | `-m` 方法、`-H` 头、`-d` 体、`--body-type`、`-e` 环境、`--save-history`、`--timeout` |
| `request` | 按名称运行集合中的请求 | 集合路径 + 请求名 + `-e` |
| `run <collection>` | Collection Runner | `-i` 迭代、`-d` 数据(CSV/JSON)、`--delay`、`--folder`、`--bail`、`--report` |
| `import <file>` | 导入集合/环境 | `--format postman\|openapi\|curl\|har\|auto`、`--no-save` |
| `export <file>` | 导出为 v2.1 | `-o` 输出路径 |
| `codegen` | 生成客户端代码 | `-l` 语言、`-c`/`-i` 集合+请求、`-u`/`-m` URL+方法 |
| `collection` | 集合操作 | `list/show/new/add-folder/add-request/rm/rename/move/duplicate/export` |
| `env` | 环境操作 | `list/show/new/set/export/delete` |
| `globals` | 全局变量 | `list/show/set/clear` |
| `history` | 历史记录 | `list/clear` |
| `ws <URL>` | WebSocket | `--send`、`--listen`、`--header`、`--subprotocol`、`--binary` |
| `sse <URL>` | Server-Sent Events | `-H`、`--count`、`--timeout` |
| `mqtt <HOST> <TOPIC>` | MQTT（明文 TCP） | `-p`、`--publish/--message`、`--qos`、`--username/--password` |
| `graphql <URL>` | GraphQL over HTTP | `-q`、`--variables`、`-e` |
| `mock <collection>` | 本地 Mock 服务 | `--port`、`--delay` |
| `perf` | 性能压测 | `-c/-i/-u/-m`、`--iterations`、`--concurrency`、`-e` |

> `codegen -l` 支持：`curl / http / rust / python / js / go / nodejs / java / csharp / php / ruby / powershell / swift`。

## 环境变量

| 变量 | 作用 |
|---|---|
| `PMRS_HOME` | 工作区根目录（默认 `~/.postman-rs`） |
| `PMRS_PROXY` | HTTP/SOCKS 代理，如 `http://127.0.0.1:7890`、`socks5h://127.0.0.1:1080` |
| `PMRS_CA_BUNDLE` | 自定义 CA 证书（PEM）路径 |
| `PMRS_CLIENT_CERT` / `PMRS_CLIENT_KEY` | 客户端证书 / 私钥（PEM），自动拼接用于 mTLS |
| `PMRS_INSECURE` | 设为 `1` 时跳过 TLS 校验 |
| `PMRS_CONNECT_TIMEOUT` | 连接超时（秒） |
| `PMRS_MAX_REDIRECTS` | 最大重定向次数（默认 10） |
| `PMRS_THEME` | GUI 主题编号 `0..7` |
| `PMRS_GUI_SIDEBAR` | GUI 初始侧栏页 `0..5`（集合 / 环境 / 全局 / 历史 / 工具 / 控制台） |

## 图形界面（快速上手）

1. 双击 `pmrs-gui-windows-x64.exe`（免安装；macOS 用 `pmrs-gui-macos-universal`）。首次左栏为空属正常，先导入示例或新建集合。
2. 顶栏选择「环境」「主题」；右侧多标签构建请求（参数 / 授权 / 请求头 / 请求体 / 测试 / 脚本 / 设置 / 描述）。
3. 点「发送」查看响应（响应体 / 响应头 / Cookies / 测试 / 计时）；侧栏「控制台」可查看请求/响应/脚本日志。
4. 左栏「工具」页可导入、运行集合、生成代码、切换主题、管理 Cookie、使用 WebSocket 工作台。

## 项目结构

```
postman-rs/
├─ src/
│  ├─ main.rs          # CLI（clap）入口与 16 组子命令
│  ├─ bin/gui.rs       # pmrs-gui（egui/eframe 桌面程序，中文界面）
│  ├─ model.rs         # Postman v2.1 数据模型（Collection/Item/Request/UrlSpec/Auth/Body…）
│  ├─ http.rs          # reqwest 执行、鉴权、Cookie、计时、NetConfig
│  ├─ variables.rs     # 变量作用域 + 动态变量
│  ├─ scripting.rs     # 轻量 pm.* 脚本（pm.test / pm.expect / pm.response…）
│  ├─ tests_engine.rs  # 声明式断言引擎
│  ├─ runner.rs        # Collection Runner
│  ├─ import_export.rs # 导入/导出（postman / openapi / curl / har）
│  ├─ codegen.rs       # 13 语言代码生成
│  ├─ ws.rs / sse.rs / mqtt.rs / mock.rs / perf.rs
│  └─ storage.rs       # 本地文件工作区
├─ tests/integration.rs
├─ examples/           # demo collection / environment / sample.curl
├─ scripts/build-macos.sh
├─ .github/workflows/build.yml   # Windows / macOS / Linux 三平台 CI
└─ dist/               # 预编译产物（git 忽略）
```

## 本次更新摘要（P0–P3）

- **P0**：`pm.expect` 匹配器大幅补全（`.not / lengthOf / property / a / match / oneOf / closeTo / …`）。
- **P1**：GUI 全局变量生效；GUI 发送继承集合/文件夹 auth·变量·脚本；新增 Postman 控制台。
- **P0/P1**：请求「描述」编辑与持久化（`item.description`）。
- **P3**：方法下拉 16 种；动态变量再补 ~30 个。
- **GUI 布局修复**：参数 / 路径变量 / 请求体表单 / 授权扩展的「键 / 值」输入框统一各占一半，详见 `docs/14_键值输入框各占一半修复.md`。
- 详见根目录 `CHANGELOG.md`；macOS 打包含见 `docs/13_macOS打包说明.md` 与 `.github/workflows/build.yml`。

## 许可

MIT OR Apache-2.0。本项目为独立实现，不包含 Postman 商标资产，仅做格式互操作。
