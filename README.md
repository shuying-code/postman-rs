# postman-rs (`pmrs`)

一个用 Rust 编写的 **本地化、无账号、无付费、无会员限制** 的 Postman 风格 API 客户端。
它复刻了 Postman 最常用的**本地**能力，采用 **Postman Collection v2.1** 兼容格式，跨平台单二进制（Windows / macOS / Linux）。

> 本项目不连接 Postman 云端，不含登录、订阅、试用、Stripe、遥测。**所有功能对任何人无条件可用。**

---

## 为什么「无限制」

Postman 的付费与登录主要约束**云端能力**（同步、团队、云端 Mock/Monitor、Vault、Governance、AI 等）。本工具只实现**不依赖云端**的本地能力，因此天然不需要账号、不需要付费、没有配额。详见 `../docs/03_补充二_登录·收费·会员限制与去除方案.md`。

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

## 许可

MIT OR Apache-2.0。本项目为独立实现，不包含 Postman 商标资产，仅做格式互操作。
