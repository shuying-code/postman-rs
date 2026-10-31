# Postman 功能复刻差距（第四轮 · 两遍核对）

> 对象：`F:\codex工作区\postman-rs`（Rust + egui/eframe 0.29，已含 P0–P3 十批修补、8+ 主题、UI 重设计、左栏宽度修复、请求头等分、cURL 导入/复制）
> 方法：**第一遍**＝逐模块读源码，记录"实际上做到了什么"（带文件:行号证据）；**第二遍**＝对照 Postman 官方功能面，逐项判定 已实现 / 部分 / 缺失 / 无需复刻。
> 日期：2026-10-31。
> 图例：✅已实现　🟡部分实现　❌未复刻（本地可补）　🚫按设计不做（云端/登录/付费/会员）

---

## 0. 一句话结论

本地"单人、纯客户端"的 HTTP 核心已相当完整：**发送请求 / 集合 / 环境 / 全局变量 / 历史 / 集合运行器 / 导入导出 / 13 语言代码生成 / 声明式+pm.test 断言 / 多标签 GUI / 8+ 主题 / WS·SSE·MQTT·Mock·Perf CLI** 都可用，且无登录、无收费、无会员限制。

**当前真正没复刻 / 只是半成品的，集中在四类：**
1. **脚本引擎不是 JS 引擎**——`pm.*` 只是正则匹配的子集，缺 `pm.sendRequest / pm.visualizer / pm.cookies / pm.iterationData / pm.variables.get / pm.response.headers.get / pm.execution.*` 等，复杂脚本会静默失效。（P0）
2. **若干 GUI 刚需缺入口**——响应体"保存/下载到文件"❌、Cookie 管理器增删改❌、全局变量编辑器❌、快捷键❌、Postman Console❌、请求/文件夹描述文档❌。（P0/P1）
3. **协议与工作台**——gRPC❌、Socket.IO❌、WS 交互式常驻双工会话❌、MQTT over TLS❌、NTLM 真实现❌、SSE/MQTT/GraphQL 仅 CLI 无 GUI 工作台❌、响应 Visualizer❌。（P1/P2）
4. **组织级继承**——集合级/文件夹级的 auth·脚本·变量 继承在 GUI 里不完整（runner 里继承了 collection.auth，但 GUI 缺"继承父级"的显式语义与文件夹级脚本执行）。（P1）

云协作 / 团队 Workspace / 登录账号 / 会员计费 / 云端 Mock·Monitor / API Network 属于 🚫 按设计不做；本地 Mock·Monitor·Perf 可作为替代补齐。

---

## 1. 第一遍：源码事实盘点（逐模块，均为当前真实实现）

| 模块 | 实际能力（证据） |
|---|---|
| `http.rs` | reqwest(blocking) + rustls；任意方法字符串；`Auth`：none/bearer/basic/apikey/oauth2/oauth1/jwt/awsv4/digest（**digest 走 401 challenge 重发**）；**ntlm 执行时报未实现**；body：raw/urlencoded/formdata/file/graphql；gzip/br/deflate；持久 Cookie jar（`~/.postman-rs/cookies.json`）；二进制→Base64；`PMRS_PROXY/PMRS_CA_BUNDLE/PMRS_CLIENT_CERT/PMRS_CLIENT_KEY/PMRS_INSECURE/PMRS_CONNECT_TIMEOUT/PMRS_MAX_REDIRECTS`；计时 dns/ttfb/download（tcp_ms 恒 None） |
| `model.rs` | `Collection/CollectionInfo/Item/ResponseExample/Request/UrlSpec/Header/KV/FileRef/GraphQL/Body/Auth/Script/Event/Variable/Environment/TestAssertion`；**`UrlSpec` 现已含 raw/host/path/query/variable/hash**（model.rs:93-101，较上一轮已补齐） |
| `variables.rs` | 作用域优先（cli>iteration>env>collection>globals）+ 递归解析；动态变量约 60 种（randomInt/UUID/timestamp/姓名/公司/职位/经纬度/货币/文件/UA 等） |
| `tests_engine.rs` | kind：status / json_path / json_path_exists / body_contains / body_equals / header / max_time_ms / response_time / regex；并能从 `pm.test(...)` 中提取 `to.have.status` / `to.have.header` |
| `scripting.rs` | **自研正则子集**（非 JS 引擎，scripting.rs:1-9）：`pm.environment/globals/collectionVariables/variables.set(...)`；`pm.test("名", ()=>{...})`；`pm.response.to.have.status(N)`；`pm.expect(EXPR).to.<eql/equal/above/below/least/most/include/true/false/ok>`；值域 `pm.response.code/status/responseTime/json().a.b/text()` |
| `runner.rs` | `RunOptions{folder,bail,report}`；iterations；数据文件 JSON/CSV；delay；继承 `collection.auth` + globals；执行 `req.event`+`item.event` 的 pre-request(`pm.*.set`) 与 test(`pm.test`)；CI 报告 JSON |
| `import_export.rs` | 导入：postman(**v1+v2.1**)/globals/openapi·swagger(**2.0+3.x**)/curl(含 `-F/-b/-A/-e/-G/--data-urlencode/-T/--json` 等)/har/auto；同名集合合并；导出：集合 v2.1、环境 |
| `codegen.rs` | 13 语言：curl/http/rust(reqwest)/python(requests)/js/go/nodejs/java(okhttp)/csharp/php/ruby/powershell/swift；**body 全模式** raw/graphql/urlencoded/formdata(含文件)/file（codegen.rs:83-135） |
| `ws.rs` | WebSocket 客户端：自定义 header、子协议、二进制发送、listen 条数、wait 超时；**请求-响应批处理式**（连接→发 N 条→收 M 条→关闭），非交互式常驻 |
| `sse.rs` | SSE 消费：url/headers/count/timeout（仅 CLI 流式打印） |
| `mqtt.rs` | MQTT 3.1.1 **纯 TCP**：用户名/密码、client-id、订阅 QoS、QoS0/1 发布；**明确不做 mqtts/TLS**（mqtt.rs 头注释） |
| `mock.rs` | 本地 HTTP Mock：按 `path` 回放集合内保存的 example 响应；可选端口/延迟 |
| `perf.rs` | 轻量压测：total/ok/err/rps/mean/p50/p90/p99/max |
| `main.rs` | CLI 16 组命令：send/request/collection/env/globals/history/import/export/codegen/run/ws/sse/mqtt/graphql/mock/perf |
| `bin/gui.rs` | 中文 GUI 4500+ 行：多标签（新建/关闭）、请求构建(参数/授权/请求头/请求体/测试/脚本/设置)、响应(体/头/Cookies/测试/计时)、**10 种鉴权**、**6 种 body 类型**、环境变量编辑保存、历史(搜索/存为请求/重放)、示例(保存/回放)、工具(主题+跟随系统+强调色 / WS 工作台 / 导入集合·环境 / cURL 粘贴导入 / 集合运行器 / 代码生成 / 请求级网络设置) |

---

## 2. 第二遍：对照 Postman 的逐项差距矩阵（P0–P3）

> P0＝影响日常基本可用/数据安全；P1＝常用功能缺失明显；P2＝进阶/专业化；P3＝锦上添花或低优先级。

### P0 — 本地核心可用性 / 数据安全

| 功能 | 状态 | 现状与证据 | 差距说明 |
|---|---|---|---|
| 发送请求 / 集合 / 环境 / 历史 | ✅ | gui.rs 请求构建、`ui_side_env`(2230)、历史区(2345) | 基础齐备 |
| **响应体保存 / 下载到文件** | ❌ | 响应区仅「保存为示例 / 复制 / 美化」(gui.rs:3945-3975)，无"保存响应到文件" | **缺失**。无法把响应正文/头存成本地文件 |
| **全局变量编辑器（GUI）** | ❌ | GUI 只有"环境变量"编辑器(ui_side_env)；`pm.globals.set` 与 CLI 有 globals，但 GUI 无全局变量管理页(全文件仅 3 处 globals 提及均为脚本/提示文本) | **缺失**。只能改文件或 CLI |
| **Cookie 管理器（增删改）** | 🟡 | 响应页有 Cookies 只读列表(gui.rs:4042-4064)；jar 持久化自动携带 | 无独立 Cookie 管理器 UI、不能手工增删改 |
| **真实 JS 脚本引擎 pm.*** | 🟡 | scripting.rs 为正则子集；`pm.sendRequest / pm.visualizer / pm.cookies / pm.iterationData / pm.variables.get / pm.response.headers.get / pm.execution.setNextRequest / pm.request.*` 均无 | 复杂脚本静默失效，与 Postman 语义差距大 |
| 断言覆盖（chai） | 🟡 | 支持 eql/equal/above/below/least/most/include/true/false/ok + status/header/json_path 等 | 缺 `a/an`、`lengthOf`、`property`、`match`、`oneOf`、`closeTo`、`.not`、嵌套链、自定义错误信息等 |
| 多标签 / 标签管理 | ✅ | tabs 新建/关闭/切换(gui.rs:2977-3045) | 无"复制标签/重命名标签/拖拽排序"（小缺口） |
| 请求保存到集合 | ✅ | 保存请求到集合(gui.rs:1480-1486)、树增删改/上下移/重命名 | 齐备 |
| 请求 / 文件夹 描述文档 | ❌ | 各 `description: None`(多处)；GUI 无描述输入 | **缺失**（Postman 的 request/folder description） |
| 快捷键（Ctrl+Enter 发送等） | ❌ | 全文件无 keymap / shortcut / Ctrl 处理 | **缺失** |

### P1 — 常用功能缺失明显

| 功能 | 状态 | 现状与证据 | 差距说明 |
|---|---|---|---|
| 集合级 / 文件夹级 脚本·变量·auth 继承 | 🟡 | runner 继承 `collection.auth`(runner.rs)；collection 变量参与作用域 | GUI 无"继承父级/文件夹级 pre-request·test 脚本"的显式执行；文件夹级变量/auth 继承不完整 |
| Postman Console（请求/脚本日志面板） | ❌ | 无 | 缺失，排障困难 |
| GraphQL 专用工作台（schema/introspection/自动补全） | 🟡 | body 类型可选 graphql，仅 query+variables 文本框(gui.rs:3654-3678) | 无 introspection、schema 浏览器、字段补全 |
| **SSE / MQTT / WebSocket GUI 工作台完整化** | 🟡 | 仅有 WS 工作台(ui_side_tools)；SSE、MQTT **仅 CLI** | SSE/MQTT 无 GUI；WS 仅批处理非交互 |
| WebSocket 交互式常驻双工会话 | ❌ | ws.rs 连接→批量发→接收→关闭 | 无"保持连接、随时发/收、消息历史"交互界面 |
| MQTT over TLS（mqtts/8883） | ❌ | mqtt.rs 头注释明确不做 TLS | 无法连加密 broker |
| NTLM 真实现 | ❌ | gui 下拉标注"NTLM（不支持）"，http.rs 报未实现 | 企业内网认证不可用 |
| 响应 Visualizer（模板渲染 HTML/图表） | ❌ | 无 | 缺失 |
| 请求对比 / diff | ❌ | 无（"对比"仅主题名"高对比"） | 缺失 |
| Mock 服务 GUI 与高级匹配 | 🟡 | mock.rs 按 path 回放；仅 CLI | 无 GUI、无按 header/query/方法/状态码的规则匹配、无动态响应 |
| 性能测试高级场景 | 🟡 | perf.rs 基础统计；CLI | 无 ramp-up、并发曲线、断言校验、GUI、逐步加压 |
| 测试片段库（Tests snippets） | ❌ | 无 | 无常用断言/脚本片段一键插入 |

### P2 — 进阶 / 专业化

| 功能 | 状态 | 现状与证据 | 差距说明 |
|---|---|---|---|
| gRPC（含 proto/reflection/流式） | ❌ | 无 grpc 模块 | 未复刻 |
| Socket.IO 客户端 | ❌ | 无 | 未复刻 |
| OAuth 2.0 authorization_code / PKCE | 🟡 | GUI 有 oauth2 字段；headless 无浏览器回调流程 | 仅 client_credentials/password/refresh_token/accessToken 可用 |
| 自定义 CA / 证书 / 代理 GUI | 🟡 | 靠环境变量 + 请求级(proxy/insecure/client_cert) | 无"设置→证书/CA 管理器"集中 UI，无私钥单独可视化 |
| 变量类型（number/boolean/secret/掩码） | 🟡 | 全按字符串；无 secret 掩码/强类型 | 缺 secret 掩码与类型校验 |
| 变量"当前值/作用域"查看器 | 🟡 | 作用域解析在引擎内 | GUI 无逐个变量的"来源作用域 + 当前值"面板 |
| 自动生成请求头管理（Host/Content-Length 等） | 🟡 | reqwest 自动加，但 GUI 不可见/不可禁 | 无"隐藏自动头/预设头"管理 |
| 导入：Insomnia / OpenAPI 3.1 / WSDL / RAML | ❌ | 支持 postman/openapi2·3/curl/har | 缺 Insomnia、3.1 细节、WSDL/RAML |
| 导出：OpenAPI 等更多格式 | 🟡 | 仅集合 v2.1 + 环境 | 不能导出 OpenAPI |
| Postman Vault / 密钥保管 | ❌ | 无 | 未复刻（可用本地明文变量替代） |
| 历史高级（请求体对比/分组/固定） | 🟡 | 历史搜索 + 存为请求 + 重放 | 无固定/分组/体对比 |

### P3 — 锦上添花

| 功能 | 状态 | 说明 |
|---|---|---|
| 扩展 HTTP 方法下拉（COPY/LINK/PURGE/LOCK…） | 🟡 | 引擎任意方法可用；GUI 下拉有限 |
| 动态变量冷门项（$randomLorem*、$isoTimestamp、$randomBankAccount、$randomAvatar、$randomImageUrl 等） | 🟡 | 已 ~60 种，冷门项可再补 |
| 代码生成更多语言/片段（Kotlin/Dart/Elixir/OCaml…） | 🟡 | 已有 13 种 |
| WS 压缩/扩展、消息分帧可视化 | ❌ | 无 |
| 请求体 raw 语言细分（JSON/XML/HTML/JS/Text） | ✅ | 已支持(raw_lang) |

### 🚫 按设计不做（云端 / 登录 / 付费 / 会员 —— 用户明确要求无条件可用）

| Postman 功能 | 处置 |
|---|---|
| 账号登录 / 云同步 / Team Workspace / 协作权限 | 不做（本地文件即"工作区"） |
| 会员专属：云端 Mock / Cloud Monitor / API Network / 高级报表 | 不做；以**本地 Mock / 本地定时(可用系统计划任务+CLI run) / 本地 perf** 替代 |
| 计费 / 用量限额 / 功能解锁 | 一律无限制 |
| Git 版本控制集成（Postman 内置） | 不做内置；用户可直接用 git 管 `~/.postman-rs` 下的 json |

---

## 3. 建议补齐优先级（下一轮若要继续做）

1. **P0（最该先补）**：响应体"另存为文件/下载"按钮；GUI 全局变量编辑器；Cookie 管理器增删改；把 `pm.*` 从正则子集升级为可用 JS 引擎（如 `boa_engine`/`rquickjs`）或至少补齐 `pm.sendRequest/pm.variables.get/pm.response.headers.get/pm.iterationData`；快捷键（Ctrl+Enter 发送、Ctrl+S 保存、Ctrl+N 新标签、Ctrl+W 关标签）。
2. **P1**：请求/文件夹描述文档字段+展示；Postman Console 日志面板；集合/文件夹级脚本与变量继承的 GUI 语义；SSE/MQTT GUI 工作台；WS 交互式常驻会话；响应 Visualizer。
3. **P2**：gRPC、Socket.IO、MQTT TLS、NTLM、OpenAPI 导出、变量类型/掩码。
4. **P3**：方法下拉扩展、冷门动态变量、更多 codegen 语言、请求 diff。

---

## 4. 验收基线（当前）

- 单元/集成测试：`lib` 26 + `gui` 2 + `integration` 3 全通过；`cargo check --all-targets` 零告警。
- release 产物已刷新到 `dist\`：`pmrs-gui-windows-x64.exe`、`pmrs-windows-x64.exe` 及 zip。
- GUI 可正常启动并抓图验证（完整 DPI：先 `SetProcessDPIAware` 再 `PrintWindow(hwnd,hdc,2)`，真实窗口约 2002×1376）。

> 说明：本文件为**第四轮两遍核对**审计结论，未改动任何源码；如需按下表实施，请另行确认目标范围。
