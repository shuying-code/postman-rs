# Postman 12.34.4 (x64) 逆向分析总报告

> 分析对象：`F:\Postman (x64).exe`
> 分析工具：REA（Reverse Engineering Anything）MCP 工具链
> 分析日期：2026-10-31
> 文档版本：v1（总报告）。另有补充一/二/三三份文档深入功能矩阵、限制去除、Rust 复刻设计。

---

## 0. 摘要与关键结论

- 这不是一个普通原生程序，而是一个 **Squirrel.Windows 自解压安装器**，内部只有一个 NuGet 包 `Postman-12.34.4-full.nupkg`，NuGet 包内是标准 **Electron 应用**（`Postman.exe` + `resources/app.asar`）。
- 应用版本 **12.34.4**，内置 Node **24.14.0**、yarn 4.18.0、React 渲染层。
- 主业务代码全部在 `resources/app.asar` 内，共 **600 个文件 / 164,120,994 字节**；核心渲染 UI 是若干 webpack 大包（`desktop-offline.js`、`js/scratchpad/*`）。
- 核心能力：HTTP 请求构建与发送、集合/环境/变量、历史、Collection Runner、代码生成、脚本（`pm.*` 沙箱）、导入导出（OpenAPI/Swagger/RAML/GraphQL/cURL/HAR/WADL/WSDL）、多协议（WebSocket、Socket.IO、MQTT、gRPC、GraphQL、SSE）、Mock Server、性能测试、DuckDB 数据集引擎、内置 Postman CLI 1.80.0。
- **登录/收费/会员门槛的真相**：客户端本身带有 **离线本地模式（Scratch Pad）**，编辑与发送请求 **无需登录**；且启动逻辑存在 `process.env.SKIP_SIGNIN` 开关可直接跳过登录初始化。付费墙（Paywall）主要围绕 **云端额度**（Mock 调用、Monitor 运行、团队席位、API Governance、Vault 等），本地 runtime 无强制配额。
- 因此「去掉登录/收费/会员限制」在实现层面等价于：**只实现本地能力，不接入任何云端计费与鉴权后端**。本报告配套的 Rust 复刻项目即采用此思路（见 `04_补充三`）。

---

## 1. 目标标识与完整性

| 属性 | 值 |
|---|---|
| 文件名 | `Postman (x64).exe` |
| 大小 | 216,938,976 字节（约 206.9 MiB） |
| SHA-256 | `4b1afdc49185abc9651974bf63ea5c34c691b24002a7af4525f4a66cbb02bb1d` |
| PE 架构 | x86_64（PE32+） |
| 安装器类型 | Squirrel.Windows（electron-winstaller） |
| 产品版本 | Postman 12.34.4 |
| 内置运行时 | Electron（Chromium）+ Node 24.14.0，yarn 4.18.0 |
| 版权 | Copyright 2026 Postman. All rights reserved. |

REA 证据：
- `open_binary` → `ev_1f96...`（`binary_overview`，PE 段表、`procedure_count=894`、`string_count=516`）
- `inspect_artifact` → `ev_4fd1...`（顶层：单文件 PE，无子容器）
- `inspect_pe_resources` → `ev_0d57...`（资源目录，命中 `DATA` 资源）

---

## 2. 解包链路（可复现）

### 2.1 PE 资源定位
PE 的 `.rsrc` 段几乎占满整个文件（`0x14003c000`–`0x14cee4c00`，约 216.7 MB）。`inspect_pe_resources` 显示存在类型名为 `DATA`、资源 ID `131` 的 RCDATA 资源：

| 字段 | 值 |
|---|---|
| 资源类型 | `DATA`（UTF-16LE `4400410054004100`） |
| 资源 ID / 语言 | 131 / 1033 |
| 数据文件偏移 | `229392`（0x38010） |
| 数据长度 | `216,665,153` 字节 |
| 数据 SHA-256 | `4ca4d5982261e355c0932083788692dae2b7bf6869a99e085d2fe805ca75c8dc` |

另有 `FLAGS`（资源 ID 132，12 字节）用于记录 Squirrel 安装标志（是否创建快捷方式等），以及标准 `RT_ICON` / `RT_GROUP_ICON` / `RT_VERSION` / `RT_MANIFEST` 等资源。

### 2.2 逐层解包结果

```
Postman (x64).exe  (PE32+, 216,938,976 B)
└─ PE .rsrc: DATA resource #131 @0x38010, len 216,665,153
   └─ payload.bin  → 实为 ZIP (magic 50 4B 03 04)
      ├─ background.gif                      685,854 B
      ├─ Postman-12.34.4-full.nupkg      216,498,585 B   ← 真正的应用包
      ├─ RELEASES                                 80 B
      ├─ setupIcon.ico                        21,672 B
      └─ Update.exe                        1,881,568 B   ← Squirrel 更新器
         Postman-12.34.4-full.nupkg (ZIP/NuGet)
         ├─ Postman.nuspec
         ├─ [Content_Types].xml
         ├─ lib/net45/Postman.exe            211,156,448 B  ← Electron 主可执行
         ├─ lib/net45/Postman_ExecutionStub.exe   362,976 B
         ├─ lib/net45/squirrel.exe             1,904,608 B
         ├─ lib/net45/resources/app.asar     164,288,006 B  ← 业务代码
         ├─ lib/net45/resources/icon.png           6,566 B
         ├─ lib/net45/resources/profile.json          77 B
         ├─ lib/net45/resources/data/...      (CLI / worker / dataset-engine)
         ├─ lib/net45/*.dll / *.pak / *.bin    (Chromium/Electron 运行库)
         └─ lib/net45/locales/*.pak            (60+ 语言资源)
```

`RELEASES` 内容：
```
7F19EC3765EAA149D5D77D8EC9B788ED90457309 Postman-12.34.4-full.nupkg 216498585
```

### 2.3 app.asar 结构
ASAR 头：`headerSize=167004`，JSON 头长度 `166995`，数据区基址 `8+headerSize=167012`。解析得到：

| 指标 | 值 |
|---|---|
| 文件数 | 600 |
| 解压后总字节 | 164,120,994 |
| JS 文件 | 385 个 |
| 原生 `.node` | 8 个 |
| `.dll` | 28 个 |
| `.proto` | 24 个 |
| 图片/字体/主题 | 多种（含 10 套编辑器主题） |

顶层结构：

| 顶层项 | 说明 |
|---|---|
| `main.js` (1.36 MB) | Electron 主进程入口 |
| `startup.js` (0.30 MB) | 启动流程 |
| `preload.js` / `preload_desktop.js` | 渲染进程预加载桥 |
| `js/` | 渲染层 webpack 产物（见 4 节） |
| `services/` | 内置扩展（crx 拦截器扩展） |
| `mock-server-runtime/` | 本地 Mock 服务器 worker |
| `toolset-mcp-server/` | MCP 服务器 worker（4.2 MB） |
| `node_modules/` | 原生模块（ripgrep、DuckDB、libcrypto 等） |
| `js/scratchpad/` | 离线主 UI（22 MB 的 `scratchpad.js` + runtime + codegen + converters） |
| `config.json` | 构建期注入的服务端点与开关 |
| `package.json` | 依赖清单与产品元数据 |

---

## 3. 技术栈

`package.json` 关键依赖（节选）：

- **运行时引擎**：`postman-runtime`、`postman-collection`、`postman-request`、`postman-ws`、`postman-runtime.platform`
- **多协议**：`@postman/grpc-js`、`@postman/grpc-reflection-js`、`@postman/proto-loader`、`@postman/protobufjs`、`mqtt`、`graphql`、`graphql-ws`、`subscriptions-transport-ws`、`socket.io`、`socket.io-client-v4`、`ws`
- **数据/存储**：`nedb`（嵌入式文档库）、`duckdb`（数据集引擎）、`electron-json-storage`
- **安全/鉴权**：`keytar`（系统密钥链）、`node-forge`、`@postman/pem`、`openid-client`、`@azure/msal-node`、`@1password/sdk`、`aws4`
- **AI/MCP**：`@modelcontextprotocol/sdk`
- **系统集成**：`@parcel/watcher`、`@vscode/ripgrep`、`systeminformation`、`@postman/node-pty`、`shell-env`、`sudo-prompt`
- **转换器**：`openapi-to-postmanv2`、`raml-to-postman`、`graphql-to-postman`、`curl-to-postmanv2`、`postman-code-generators`

`config.json` 关键端点/开关（构建期注入，带 `__WP_` 前缀）：

| 键 | 含义 |
|---|---|
| `__WP_IDENTITY_URL__` | `https://identity.getpostman.com` 身份服务 |
| `__WP_GLOBAL_IDENTITY_URL__` | `https://auth.postman.com` |
| `__WP_GOD_INTEGRATIONS__URL` | `https://app.getpostman.com` 云端 Web |
| `__WP_ANALYTICS_URL__` | `https://events.getpostman.com/events` 埋点 |
| `__WP_PRELOGIN_PURCHASE_URL__` | `https://www.postman.com` 购买页 |
| `__WP_ALLOWED_PAYMENT_HOSTS__` | Stripe / Google Pay / reCAPTCHA 等支付域 |
| `__WP_TRIAL_DURATION__` | `2592000000` ms = 30 天试用 |
| `__WP_BUILD_SCRATCHPAD__` | `true` 构建离线面板 |
| `__WP_INTERCEPTOR_APP_ID__` | `aicmkgpgakddgnaphhhpliifpcfhicfo` 拦截器扩展 |

---

## 4. 进程与运行时架构

Electron 多进程 + Webview 架构：

| 层 | 文件 | 职责 |
|---|---|---|
| 主进程 | `main.js`、`startup.js` | 窗口管理、IPC、服务注册、文件系统、协议处理、自动更新、登录流（AuthHandler） |
| 预加载桥 | `preload.js`、`preload_desktop.js`、`browser_tab_recorder_preload.js`、`tlsDenylistPreload.js`、`visualizer_view_preload.js` | 暴露受限 API 给渲染层 |
| 主请求页 | `html/requester.html` + `js/desktop-offline.js` | 登录态主界面外壳（含 webview 容器、离线错误页、跳转 Scratch Pad） |
| 离线主 UI | `html/scratchpad.html` + `js/scratchpad/*` | 无需登录的本地工作台（集合/环境/历史/请求/运行器/代码生成） |
| 运行器 | `html/runner.html` + `js/runner.js` | Collection Runner |
| 认证页 | `html/auth.html` + `js/authentication.js` | OAuth/SSO 登录窗口 |
| Mock | `mock-server-runtime/*.js` | 本地 HTTP Mock 服务器 |
| 执行 worker | `execution-worker.js`、`AgentWorker.js`、`_thread.js`、`ThreadGroupExecutable.js` | 沙箱脚本执行线程 |
| 数据集守护 | `main-dataset-daemon*.js`、`duckdb-*.js` | DuckDB 数据集/性能测试 |
| CLI | `resources/data/postman-cli/postman.bundled.js` (32 MB) | 内置 Postman CLI 1.80.0 |
| MCP | `toolset-mcp-server/worker.js` | Postman 对外 MCP 服务 |

REA 对 `package.json/config.json/main.js/preload*.js/startup.js` 的作用域图分析（`ev_3344...`）得到：
- 相关文件 449，实际解析 JS 4 个，AST 节点 469,623，findings 2607，modules 967；
- Electron IPC：20 个操作、7 个字面 channel、9 个 main handler、10 个 renderer 发送、1 个 renderer 监听；
- 覆盖为 partial（`startup.js` 语义图超出父结果预算，静态事实已保留）。这属于 REA 的静态语法证据，不代表运行时执行。

`main.js` 中注册的部分 IPC 能力（抽样）：
`accessProxySetup/Status/SyncCa/Teardown`、`dataset-daemon-*`、`initialize-duckdb`、`query-duckdb-aggregation`、`ripgrep-search`、`generateCert`、`checkRootCAValidity`、`runNewPerformanceTestSystem`、`getHardwareMetrics`、`handle-export-as-PDF`、`getLoggedInUsers`、`fetchFromThirdParty`、`file`。

---

## 5. 功能总览

| 功能域 | 是否本地可用 | 说明 |
|---|---|---|
| HTTP 请求发送 | ✅ 本地 | 全部方法、Header/Query/Body、鉴权、代理、证书 |
| 集合 / 文件夹 / 请求管理 | ✅ 本地 | Postman Collection v2.1 兼容 |
| 环境 / 全局 / 集合变量 | ✅ 本地 | `{{var}}` 解析 + 动态变量 |
| 历史记录 | ✅ 本地 | History Sidebar |
| Collection Runner | ✅ 本地 | 多迭代、数据文件、断言、保存结果 |
| 代码生成 | ✅ 本地 | 30+ 语言/变体 |
| 导入 / 导出 | ✅ 本地 | OpenAPI/Swagger/RAML/GraphQL/cURL/HAR/WADL/WSDL |
| 脚本（`pm.*`） | ✅ 本地 | 沙箱执行前置/测试脚本 |
| WebSocket / Socket.IO / MQTT / gRPC / GraphQL / SSE | ✅ 本地 | 多协议工作台 |
| 本地 Mock Server | ✅ 本地 | `mock-server-runtime` |
| 性能测试 | ⚠️ 本地引擎 + 云端报告 | DuckDB 引擎本地，云端报告需登录 |
| 工作区同步 / 团队库 / 共享 | ❌ 需登录 | 云端 |
| 云端 Mock / 云端 Monitor | ❌ 需登录 + 配额 | 云端 |
| Vault / API Governance / SSO / 在线文档发布 | ❌ 需登录 + 付费 | 云端 |
| AI（Postman AI / MCP） | ⚠️ 需配置 | 依赖云端或本地 MCP |

---

## 6. 数据模型

核心对象（与 `postman-collection` 一致）：

- **Collection**：`info{name,_postman_id,schema,description}` + `item[]` + `variable[]` + `auth` + `event[]`(prerequest/test)。
- **Item/Folder**：可嵌套 `item`，叶子为 **Request**。
- **Request**：`method`、`url`(raw/host/path/query/variable)、`header[]`、`body{mode:raw|urlencoded|formdata|file|graphql}`、`auth`、`event[]`。
- **Response**：`status`、`code`、`header[]`、`body`、`responseTime`、`cookie[]`。
- **Environment**：`{id,name,values:[{key,value,type,enabled}],_postman_variable_scope:"environment"}`。
- **Globals**：`_postman_variable_scope:"globals"`。
- **Collection Variables**：集合内 `variable[]`。

集合格式版本（从 bundle 提取的 schema 串）：
`.../json/collection/v1.0.0/collection.json`、`v2.0.0/collection.json`、`v2.1.0/collection.json`。

---

## 7. 协议支持

| 协议 | 组件 | 备注 |
|---|---|---|
| HTTP/1.1、HTTP/2 | `postman-request` | 核心请求引擎 |
| WebSocket | `postman-ws`、`ws`、`ECWebsocketWorkbench` | 双向消息 |
| Socket.IO | `socket.io-client-v2/v4` | v2/v4 双版本 |
| MQTT | `mqtt`、`mqtt-client` | 发布订阅 |
| gRPC / gRPC Reflection | `@postman/grpc-js` + protobuf | 支持反射与服务调用 |
| GraphQL | `graphql`、`graphql-ws`、`subscriptions-transport-ws` | 查询/订阅 |
| SSE / EventStream | `EventSource` | 服务端推送 |
| SOAP / WSDL / WADL | 导入器 | 转换为 HTTP 请求 |

鉴权类型（bundle 命中次数为静态出现次数，仅作规模参考）：
OAuth 1/2（`oauth` 685）、Digest（248）、NTLM（141）、AWS v4（62）、Hmac（104）、Bearer、Basic、API Key、Hawk、JWT、Akamai EdgeGrid 等。

---

## 8. 脚本与执行沙箱

沙箱 API（`pm.*`，静态命中）：

| API | 用途 | 命中次数 |
|---|---|---|
| `pm.sendRequest` | 脚本内发请求 | 35 |
| `pm.test` | 定义测试 | 13 |
| `pm.expect` | chai 断言 | 8 |
| `pm.response` | 响应对象（`.to.have` 等） | 24 |
| `pm.environment` | 环境变量读写 | 15 |
| `pm.globals` | 全局变量读写 | 15 |
| `pm.collectionVariables` | 集合变量 | 3 |
| `pm.variables` | 临时变量 | 2 |
| `pm.iterationData` | 数据驱动 | 4 |
| `pm.cookies` | Cookie 罐 | 5 |
| `pm.visualizer` | 可视化模板 | 1 |
| `pm.execution` | 跳过请求/设置下一步 | 25 |
| `pm.vault` | 密钥保险箱 | — |

执行引擎：`postman-runtime` + `@postman/hermes`（Hermes JS 引擎）在 `execution-worker.js` 线程中运行前置/测试脚本。

---

## 9. 代码生成矩阵（完整）

从 `postman-code-generators.js` 提取的 `lang/variant` 全量：

```
C / libcurl              csharp / HttpClient         csharp / RestSharp
curl / cURL              Dart / dio                  Dart / http
go / Native              http / HTTP                 java / OkHttp
java / Unirest           JavaScript / Fetch           javascript / jQuery
JavaScript / XHR         Kotlin / Okhttp             nodejs / Axios
nodejs / Native          nodejs / Request            nodejs / Unirest
Objective-C / NSURLSession   ocaml / Cohttp          php / cURL
php / Guzzle             php / HTTP_Request2         php / pecl_http
postman-cli / Postman CLI  powershell / RestMethod  python / http.client
python / Requests        R / httr                     R / RCurl
Ruby / Net::HTTP         Rust / reqwest               shell / Httpie
shell / wget             swift / URLSession
```

共 35 个语言/变体。Rust 复刻项目至少实现 `curl / Rust(reqwest) / Python(requests) / JavaScript(fetch) / Go(Native) / nodejs(Axios)` 等常用项（见 `04_补充三`）。

---

## 10. 导入 / 导出转换器

| 格式 | 组件文件 | 方向 |
|---|---|---|
| OpenAPI 3.x / Swagger 2.0 | `openapi-to-postmanv2.js`（0.93 MB） | 导入 |
| Swagger 1.x | `swagger1-to-postman.js` | 导入 |
| RAML 0.8 / 1.0 | `raml-to-postman.js`、`raml1-to-postman.js` | 导入 |
| GraphQL | `graphql-to-postman.js` | 导入 |
| cURL | `curl-to-postmanv2.js` | 导入 |
| HAR | 内置解析（`har` 命中 5907） | 导入 |
| WADL / WSDL / SOAP | 内置 | 导入 |
| Postman Collection v1/v2/v2.1 | 原生 | 导入 + 导出 |
| Postman Environment | 原生 | 导入 + 导出 |

---

## 11. 内置 CLI

`resources/data/postman-cli/`：`@postman/cli-bundle-windows-x64` 版本 **1.80.0**。

- `postman.bundled.js`（32.5 MB）单文件打包；`mock-server-worker.js`（0.9 MB）；`bin/postman(.cmd)` 启动器。
- 典型用途：CI/CD 中运行集合、上报结果到云端（云端部分需 API Key/登录）。

---

## 12. 登录 / 收费 / 会员限制（概述）

详见 `03_补充二_登录·收费·会员限制与去除方案.md`。要点：

1. **离线模式**：主界面加载失败或未登录时会引导切换到 **Scratch Pad**（`js/desktop-offline.js` 中 `Switch to Scratch Pad` 按钮，`isSignedOutUser` 分支），离线 UI 功能完整。
2. **跳过登录开关**：`main.js` 启动链：`if("true"!==process.env.SKIP_SIGNIN) AuthHandler.init()` —— 设置 `SKIP_SIGNIN=true` 即不初始化登录流程。
3. **试用/付费**：`__WP_TRIAL_DURATION__=30 天`，`__WP_PRELOGIN_PURCHASE_URL__` 与 Stripe 支付域；付费主要针对云端资源额度，本地无硬性配额。
4. **免费/付费边界**：本地请求、集合、环境、历史、runner、代码生成、导入导出不受限；云端协同、Mock 云调用、Monitor、Vault、Governance、团队席位受订阅约束。

---

## 13. 证据与可复现附录

REA 证据 ID：

| 证据 | evidence_id | 内容 |
|---|---|---|
| PE 打开 | `ev_1f96df20...` (subject 同上) | `binary_overview` 段表/统计 |
| 产物清单 | `ev_4fd1a0f8...` | `inspect_artifact` 顶层图 |
| PE 资源 | `ev_0d57406f...` | `inspect_pe_resources` 资源树 |
| ASAR 作用域图 | `ev_3344ef66f3a2380235c1e2666eef69154db0062bde1aef2c236c8ab9979a59f3` | JS 应用图（partial） |

关键摘要值：
- 安装器 SHA-256：`4b1afdc49185abc9651974bf63ea5c34c691b24002a7af4525f4a66cbb02bb1d`
- DATA 资源 SHA-256：`4ca4d5982261e355c0932083788692dae2b7bf6869a99e085d2fe805ca75c8dc`
- app.asar SHA-256：`e6aa559d07765e045bd245057c294d5500644dffed0ce5860e399b5e043cf023`

复现步骤（PowerShell 摘要）：
1. 从 PE 偏移 `229392` 读取 `216665153` 字节 → `payload.bin`。
2. `Expand-Archive payload.bin`（改名为 `.zip`）→ 得到 `Postman-12.34.4-full.nupkg`。
3. `Expand-Archive *.nupkg` → `lib/net45/resources/app.asar`。
4. 解析 ASAR 头（`headerSize=167004`，JSON 从偏移 16 起，长度 166995）→ 600 个文件。

---

## 14. 局限与声明

- 本报告基于**静态**逆向（PE 资源、归档清单、JS bundle 文本、REA 静态语法图），**未运行**目标程序，因此不声称任何运行时行为已被观测。
- 完整 164 MB ASAR 的语义图超出单次调用时限（450 s），仅对入口模块做了作用域分析（partial）。
- 功能命中次数是静态字符串出现次数，仅用于规模参考，不代表功能被实际启用。
- 本报告用于互操作与自建替代实现的工程分析；未复制 Postman 商标资产用于再分发。
