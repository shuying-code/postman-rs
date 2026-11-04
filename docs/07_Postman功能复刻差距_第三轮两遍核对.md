# Postman 功能复刻差距（第三轮 · 两遍核对）
> 对象：`F:\codex工作区\postman-rs`（已含 P0–P3 修补、8 套主题、UI 重设计）
> 方法：第一遍＝源码事实盘点（逐模块看实际实现）；第二遍＝对照 Postman 功能面逐项比对。
> 日期：2026-10-31。图例：✅已实现 / 🟡部分 / ❌未复刻(本地可补) / 🚫按设计不做(云端/登录/付费)

---

## 0. 一句话结论
本地 HTTP 核心（发送 / 集合 / 环境 / 全局变量 / 历史 / Runner / 导入导出 / 代码生成 / 声明式断言 / 多协议 CLI / Mock / 性能测试 / 8 主题 GUI）已具备。
**仍缺的主要是**：Postman 的**完整 JS 脚本引擎（pm.*）**、**结构化 URL/路径变量**、**更多鉴权类型**、**GUI 的集合树增删改 / 丰富 Body 编辑器 / 各类工作台**、**gRPC 与 Socket.IO**、**响应计时细分**、**OpenAPI(Swagger2)/v1·v2.0 导入**、**codegen 的 form/file 体**。

---

## 1. 第一遍：源码事实盘点（按模块）

| 模块 | 实际能力（证据） |
|---|---|
| `http.rs` | reqwest + rustls；方法任意字符串；`Auth`: none/inherit/bearer/basic/apikey/**oauth2**(client_credentials/password/refresh_token/accessToken)；body: raw/urlencoded/formdata/file/graphql；gzip/br/deflate、持久 Cookie jar、二进制→Base64；`PMRS_PROXY/PMRS_CA_BUNDLE/PMRS_CLIENT_CERT/PMRS_CLIENT_KEY/PMRS_INSECURE/PMRS_CONNECT_TIMEOUT/PMRS_MAX_REDIRECTS` |
| `model.rs` | `Collection/Item/Request/ResponseExample/Header/KV/FileRef/GraphQL/Body/Auth/Script/Event/Variable/Environment/TestAssertion`；**`UrlSpec` 仅 `raw` 字段** |
| `variables.rs` | 作用域优先（cli>iteration>env>collection>globals）+ 递归解析；动态变量 ~60 种（randomInt/UUID/timestamp/姓名/公司/职位/经纬度/货币/文件/UA/locale/MAC/IPV6/日期/银行/信用卡…） |
| `tests_engine.rs` | 支持 kind：status / json_path / json_path_exists / body_contains / body_equals / header / max_time_ms / response_time / regex；解析 `pm.test(...status(N))`、`to.have.header(...)` |
| `runner.rs` | `RunOptions{folder,bail,report}`；迭代/CSV·JSON 数据/延迟/断言统计/CI 退出码；继承 `collection.auth` + globals；执行 `req.event`+`item.event` |
| `import_export.rs` | 导入 postman(**仅 v2.1**)/环境(`values`)/OpenAPI(**仅 servers + parameters + requestBody 基础**)/curl/HAR；同名集合**合并**；导出集合 v2.1、环境 |
| `codegen.rs` | curl/http/rust/python/js/go/nodejs/java(okhttp)/csharp/php/ruby/powershell/swift；body 仅 raw/graphql（form/file 丢弃）；注入 bearer/basic/apikey |
| `ws.rs` | 基础 WebSocket 客户端（url/messages/listen/wait）；**无自定义 header、无子协议、无二进制发送/鉴权** |
| `sse.rs` | SSE 消费（url/headers/count/timeout） |
| `mqtt.rs` | MQTT 3.1.1 **纯 TCP**（host/port/topic/publish/listen）；**QoS0、无 TLS、无用户名密码** |
| `mock.rs` | 本地 Mock：按 `METHOD path` 回放保存的 examples，端口+延迟 |
| `perf.rs` | 负载测试：iterations/concurrency、p50/p90/p99、错误率、RPS |
| `main.rs` | CLI：send/request/collection(list/show/new/add-folder/add-request/rm/rename)/env(list/show/new/set/export)/history/import/export/codegen/run/globals/ws/sse/mqtt/graphql/mock/perf |
| `bin/gui.rs` | 中文 GUI：多标签、请求构建(参数/授权/请求头/请求体/测试)、响应(体/头/Cookies/测试/计时)、导入、Runner、代码生成、环境编辑、历史重放、顶部搜索、8 主题 + 渐变 + JSON 高亮 |

---

## 2. 第二遍：对照 Postman 的功能差距矩阵

### 2.1 请求构建器
- 方法：🟡 引擎任意方法；GUI 下拉仅 7 种（无 COPY/LINK/UNLINK/PURGE/LOCK 等）
- URL 结构化对象：❌ 仅 `raw`；**path 变量 `:id`、`url.variable`、query 的 disabled、hash 均不保留**
- Query 可视化：🟡 CLI/GUI 参数页可编辑，但导入的 disabled 标志丢失
- Headers：🟡 键值+启用/禁用；❌ 自动头管理/预设(Preset)/隐藏自动头
- Body：🟡 引擎支持 raw/urlencoded/formdata/file/graphql；**GUI 仅 json/text/none**（无 form-data/file/graphql/urlencoded 编辑器）
- 请求设置：🟡 重定向/代理/证书/SSL/超时走全局环境变量；❌ 不能按请求单独配置；❌ SNI

### 2.2 鉴权
- none/basic/bearer/apikey(header/query)：✅
- oauth2：🟡 仅 client_credentials/password/refresh_token/accessToken；❌ authorization_code/PKCE（明确报错，headless 不支持浏览器流程）
- oauth1 / digest / ntlm / awsv4(SigV4) / hawk / jwt / edgegrid / asap：❌
- GUI 授权页：🟡 仅 none/bearer/basic/apikey，**未暴露后端已支持的 oauth2**
- CLI `send`：🟡 无 --auth 参数（需靠 header 或集合级 auth）

### 2.3 变量
- 作用域（cli/iteration/env/collection/globals）+ 递归：✅
- Globals：✅（文件 + CLI + 最低优先级）
- 类型 secret/number/boolean：🟡 全按字符串；无 secret 掩码/强类型/系统密钥链
- 动态变量：✅ 非常全（~60 种）；少量冷门（如 `$randomImageUrl`/`$randomAvatar`/`$randomLorem*`）可再补
- 脚本侧 `pm.variables/local/temp`、变量“当前值”查看：❌/🟡

### 2.4 脚本与测试（pm.*）
- pre-request 脚本执行（改请求 / `pm.sendRequest`）：❌
- test 脚本完整 JS（Hermes/boa/rquickjs）：❌（仅声明式断言 + 少量正则解析）
- 断言 kind：✅ status/json_path/json_path_exists/body_contains/body_equals/header/max_time_ms/response_time/regex；❌ schema(chai jsonSchema)/自定义 chai/visualizer
- 集合级/文件夹级 event 执行：❌（`collection.event` 解析但不执行；Runner 用 req/item event）

### 2.5 Collection Runner
- 迭代/CSV·JSON/延迟/断言统计/CI 退出码/folder/bail/report：✅
- 继承集合级鉴权 + globals：✅
- 保存响应 / DuckDB / 结果可视化：🟡（CLI 文本 + GUI 文本结果）
- Runner 内 pre-request 脚本：❌

### 2.6 导入 / 导出
- 导入：postman v2.1 ✅ / 环境 ✅ / OpenAPI(basic) 🟡 / curl 🟡 / HAR 🟡；**v1、v2.0 ❌**；**globals ❌**
- OpenAPI：🟡 `servers` + parameters(query/path/header) + requestBody 基础；**Swagger2 的 host/basePath/schemes ❌、securitySchemes ❌**
- 导出：集合 v2.1 ✅、环境 ✅；globals/OpenAPI/v2.0/v1 ❌
- 同名合并：✅

### 2.7 代码生成
- 13 种语言 ✅；Postman 30+ 🟡（缺 C libcurl、C# RestSharp、Dart、Elixir、Kotlin、R、OCaml、Shell wget 等）
- form/urlencoded/file/graphql 体：🟡 仅 raw/graphql 生成（form/file 丢弃）

### 2.8 集合 / 工作区
- 数据结构树 ✅；CLI add-folder/add-request/rm/rename ✅
- CLI move/duplicate/导出集合 ❌；**GUI 集合树增删改/文件夹/拖拽排序 ❌**（GUI 只读+载入）
- 集合描述 Markdown 渲染 / examples 展示：🟡（字段存但不展示）
- Workspace/团队/git 同步/Fork/版本比较/公开 API Network：🚫/❌（云端·账号相关）

### 2.9 历史
- 记录/list/clear ✅；GUI 点击重放 ✅；搜索历史 🟡（顶部搜索只过滤请求）；保存为请求/集合 ❌

### 2.10 网络与连接
- 代理/CA/客户端证书/跳过校验/超时/最大重定向：✅（全局环境变量）
- Cookie 罐（持久、自动携带）：✅
- 压缩 gzip/br/deflate、二进制 Base64：✅
- **SOCKS 代理**：❌（reqwest 未开 socks feature）
- **响应计时细分**（DNS/TCP/TLS/TTFB）：❌（仅总耗时）
- SNI/按请求级证书：❌

### 2.11 多协议工作台
- WebSocket：🟡 CLI 基础（无 header/子协议/二进制/鉴权）；GUI ❌
- SSE：🟡 CLI；GUI ❌
- MQTT：🟡 CLI 纯 TCP/QoS0（无 TLS/账号）；GUI ❌
- GraphQL：🟡 CLI（query+vars over HTTP）；GUI 无独立工作台
- **gRPC**：❌
- **Socket.IO**：❌

### 2.12 Mock / 性能 / 文档
- 本地 Mock（回放 examples，端口/延迟）：✅；动态 mock 脚本/按 query·body 匹配/CORS：🟡/❌
- 性能测试（并发、P50/90/99、错误率、RPS）：✅；ramp-up/DuckDB/报告图：❌
- API 文档生成发布 / Monitors / Flows / Postbot AI / Interceptor 抓包 / Vault / Governance：🚫/❌

### 2.13 GUI / UX
- 已具备：中文、8 主题+渐变、多标签、请求构建、响应 5 标签、JSON 高亮、导入、Runner、代码生成、环境编辑、历史重放、顶部搜索 ✅
- **集合树增删改/文件夹/拖拽 ❌**
- **保存响应的 examples 展示 ❌**
- **按请求的网络设置 UI ❌**
- **pre-request/test 脚本编辑器 ❌（仅 JSON 断言编辑器）**
- **form-data/file/graphql/urlencoded 体编辑器 ❌**
- **oauth2 授权 UI ❌、globals UI ❌、Cookie 编辑（仅查看）❌**
- 取消请求按钮 ❌、标签重命名/复制请求 ❌
- WebSocket/SSE/MQTT/GraphQL/Mock/Perf 的图形工作台 ❌（仅 CLI）

---

## 3. 尚未复刻清单（按优先级）

### P0（影响正确性/数据完整性的剩余项）
1. 结构化 URL：支持 `:path` 变量、`url.variable`、query 的 enabled/disabled 与 hash 保真（当前仅 raw，导入会丢信息）。
2. OpenAPI 导入补 Swagger2（`host/basePath/schemes`）与 `securitySchemes`；补 Postman v1/v2.0 导入与 globals 导入。
3. codegen 的 form/urlencoded/file/graphql 请求体生成（当前丢弃）。

### P1（高频本地功能）
1. GUI 集合树 CRUD：新建/重命名/删除/文件夹/拖拽排序/复制；集合级/文件夹级运行选择。
2. GUI Body 编辑器补 urlencoded / form-data(file) / graphql；授权页补 OAuth2。
3. GUI 请求级网络设置（重定向、超时、代理、证书、SSL）与“取消请求”。
4. 保存响应为 example 并在 GUI 展示；导出 examples。
5. CLI：env delete、collection move/duplicate/export。

### P2（进阶）
1. 响应计时细分（DNS/TCP/TLS/TTFB）。
2. 更多鉴权：oauth1 / digest / ntlm / awsv4(SigV4) / jwt。
3. SOCKS 代理（reqwest 开 socks feature）。
4. Runner 结果可视化/保存响应/DuckDB 报告。
5. 历史搜索、保存历史为请求。

### P3（协议工作台与脚本引擎）
1. **完整 JS 脚本引擎**（boa/rquickjs）+ `pm.*`（pre-request/test、pm.sendRequest、pm.environment.set、pm.test 全量、chai/schema）。
2. gRPC、Socket.IO 支持。
3. WebSocket/SSE/MQTT/GraphQL/Mock/Perf 的**图形工作台**（当前仅 CLI）；WS 补 header/子协议/二进制，MQTT 补 TLS/账号/QoS1-2。
4. 主题跟随系统/自定义强调色。

### 🚫 按设计不做（云端/登录/付费）
云同步、团队协作、云端 Mock/Monitor、Postman Vault、Governance、Postbot AI、Public API Network、版本比较(Fork)、Interceptor 抓包上传等。**本工具全部本地、无条件可用，无登录/付费/会员限制。**

---

## 4. 复现命令
```powershell
cd F:\codex工作区\postman-rs
Get-Content .\src\lib.rs
Select-String -Path .\Cargo.toml -Pattern 'reqwest|socks|eframe'
Select-String -Path .\src\model.rs -Pattern 'pub struct|pub [a-z_]+:'
Select-String -Path .\src\http.rs -Pattern 'bearer|basic|apikey|oauth2|grant|formdata|urlencoded|PMRS_'
Select-String -Path .\src\import_export.rs -Pattern 'servers|host|basePath|schemes|security|info|values'
Select-String -Path .\src\codegen.rs -Pattern '=> gen_'
Select-String -Path .\src\tests_engine.rs -Pattern 'status|json_path|header|regex|response_time|body_'
Select-String -Path .\src\variables.rs -Pattern '"\$random'
Select-String -Path .\src\bin\gui.rs -Pattern 'body_type|auth_kind|\["json", "text", "none"\]|folder|duplicate|redirect'
```
