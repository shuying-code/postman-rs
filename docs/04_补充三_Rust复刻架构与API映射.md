# 补充三：Rust 复刻架构与 Postman API 映射

> 目标：用 Rust 构建跨平台（Windows/macOS/Linux）的 Postman 本地功能复刻，**无账号、无付费、无会员限制、所有功能无条件可用**。
> 项目位置：`F:\codex工作区\postman-rs`

---

## 1. 设计原则

1. **本地优先**：只复刻不依赖 Postman 云端的能力，因此天然无登录/计费。
2. **格式兼容**：直接读写 **Postman Collection v2.1** 与 **Environment** JSON，可导入真实 Postman 导出文件、也可把结果导回 Postman。
3. **CLI + 库双形态**：`pmrs` 命令行工具 + 可复用的 `postman_rs` 库。
4. **可组合**：每个子命令单一职责；输出 JSON 便于脚本化与验证。
5. **可验收**：每个核心模块有单元测试；HTTP 有集成测试；文档给出验收命令。

---

## 2. 概念映射表（Postman → postman-rs）

| Postman 概念 | Rust 结构 / 模块 | 说明 |
|---|---|---|
| Collection (`v2.1.0`) | `model::Collection` | `info` + `item[]` + `variable[]` + `auth` |
| Folder / Item | `model::Item`（含 `item` 嵌套或 `request`） | 递归树 |
| Request | `model::Request` | method/url/header/body/auth |
| Url | `model::Url`（raw 或结构化） | 支持 `{{var}}` |
| Header | `model::Header { key,value,disabled }` | 禁用即不发送 |
| Body | `model::Body { mode, raw, urlencoded, formdata, file, graphql }` | 覆盖全部 mode |
| Auth | `model::Auth { type, basic,bearer,apikey,... }` | 支持常见类型 |
| Environment | `model::Environment` | `values[]` + scope |
| Globals | `model::Environment`（scope=globals） | 复用 |
| Collection Variables | `Collection.variable` | 作用域解析 |
| History | `storage::HistoryEntry` | JSON 行存储 |
| Runner | `runner::run_collection` | 迭代 + 数据 + 断言 |
| Tests (`pm.test`) | `tests_engine::assertions` | 声明式断言 + 常见形态解析 |
| Code Generators | `codegen::generate` | curl/rust/python/js/go/node |
| Importers | `import_export::import_*` | postman/openapi/curl/har |
| Exporters | `import_export::export_collection` | v2.1 JSON |
| Mock Server | `mock::serve`（计划/可选） | 本地回放示例 |
| WebSocket | `ws`（可选实现） | 连接/收发 |

---

## 3. 项目结构

```
postman-rs/
├─ Cargo.toml
├─ README.md                 # 使用说明 + 无条件原则 + 验收
├─ src/
│  ├─ lib.rs                 # 库入口，导出模块
│  ├─ main.rs                # CLI 入口（clap）
│  ├─ cli.rs                 # 子命令定义与调度
│  ├─ model.rs               # Postman v2.1 数据模型（serde）
│  ├─ variables.rs           # 变量作用域解析 + 动态变量
│  ├─ auth.rs                # 鉴权应用到请求
│  ├─ http.rs                # reqwest 执行请求，产出 Response
│  ├─ storage.rs             # 本地工作区：collections/environments/history
│  ├─ import_export.rs       # 导入(postman/openapi/curl/har) 与 导出
│  ├─ codegen.rs             # 多语言代码生成
│  ├─ tests_engine.rs        # 断言引擎
│  └─ runner.rs              # Collection Runner
├─ examples/
│  ├─ demo_collection.json   # 示例集合（Postman v2.1）
│  └─ demo_environment.json  # 示例环境
└─ tests/
   └─ integration.rs         # 启动本地 HTTP 服务器做端到端测试
```

---

## 4. 数据模型（关键定义，Postman v2.1 兼容）

```rust
pub struct Collection {
    pub info: CollectionInfo,          // name, _postman_id, schema, description
    pub item: Vec<Item>,
    pub variable: Vec<Variable>,
    pub auth: Option<Auth>,
    pub event: Vec<Event>,
}
pub struct Item {
    pub name: String,
    pub item: Option<Vec<Item>>,       // 文件夹
    pub request: Option<Request>,
    pub event: Vec<Event>,
}
pub struct Request {
    pub method: String,
    pub url: UrlSpec,
    pub header: Vec<Header>,
    pub body: Option<Body>,
    pub auth: Option<Auth>,
    // 扩展：声明式断言（非 Postman 原生，可选）
    #[serde(default)] pub tests: Vec<TestAssertion>,
}
pub struct UrlSpec { pub raw: String }  // 也接受结构化对象
pub struct Header { pub key: String, pub value: String, #[serde(default)] pub disabled: bool }
pub struct Body {
    pub mode: String,                  // raw/urlencoded/formdata/file/graphql
    pub raw: Option<String>,
    pub urlencoded: Vec<KV>,
    pub formdata: Vec<KV>,
    pub file: Option<FileRef>,
    pub graphql: Option<GraphQL>,
}
pub enum Auth { None, Bearer{token:String}, Basic{username,password},
                ApiKey{key,value,in_}, /* ... */ Inherit }
pub struct Environment { pub id: Option<String>, pub name: String, pub values: Vec<Variable>,
                         #[serde(rename="_postman_variable_scope")] pub scope: Option<String> }
```

解析容错：`url` 既可为字符串也可为对象（自定义 `Deserialize`）；字段用 `#[serde(default)]` 容忍缺失。

---

## 5. 变量解析

- 语法 `{{name}}`，递归解析（最多 N 层，防环）。
- 作用域优先级：`--var` 临时 > `iterationData` > `environment` > `collection` > `globals`。
- 动态变量（与 Postman 同名）：`$randomInt`、`$randomUUID`、`$guid`、`$timestamp`、`$isoTimestamp`、`$randomAlphaNumeric`、`$randomBoolean`、`$randomFirstName`、`$randomLastName`、`$randomEmail`、`$randomPassword`、`$randomCity`、`$randomCountry`、`$randomPhoneNumber`、`$randomIP`、`$randomColor`、`$randomHexColor`、`$randomAbbreviation`。

---

## 6. HTTP 执行

- 基于 `reqwest`（blocking，rustls TLS），支持所有方法。
- Body 映射：`raw`→bytes+Content-Type；`urlencoded`→表单；`formdata`→multipart（含文件）；`file`→二进制流；`graphql`→JSON `{query,variables}`。
- Header 过滤 `disabled`；变量先解析再发送。
- 鉴权：`basic`/`bearer`/`apikey(header|query)` 自动注入；其余类型给出明确「未实现」错误（不静默）。
- 重定向策略、超时、可选代理可配置。
- 产出 `ResponseRecord{ status, code, headers, body, elapsed_ms }` 用于展示/断言/历史。

---

## 7. 本地工作区存储

```
~/.postman-rs/            (可用 PMRS_HOME 覆盖)
├─ collections/*.json      # 集合（v2.1）
├─ environments/*.json     # 环境
└─ history.jsonl           # 历史（每行一条）
```
纯文件、可纳入 git、可手工编辑；无数据库、无云端。

---

## 8. 导入 / 导出

| 命令 | 支持 |
|---|---|
| `import --format postman` | Collection v2.1 / v2.0 / v1 |
| `import --format openapi` | OpenAPI 3.x / Swagger 2.0（路径→请求，基础鉴权/参数推断） |
| `import --format curl` | `curl` 命令转请求 |
| `import --format har` | HAR `log.entries` → 请求 |
| `export` | 输出 v2.1 Collection / Environment |

---

## 9. 代码生成

`codegen --lang <x>` 支持（与 Postman 命名对齐）：

| lang 值 | 生成 |
|---|---|
| `curl` | `curl` 命令 |
| `rust` | `reqwest` 客户端代码 |
| `python` | `requests` 代码 |
| `js` | `fetch` 代码 |
| `go` | `net/http` 代码 |
| `nodejs` | `axios` 代码 |

（Postman 另有 30+ 语言变体，本项目优先实现上述高价值项，其余按同一 `Request` 模型可扩展。）

---

## 10. Collection Runner

`pmrs run <collection.json> [-e env.json] [-i N] [-d data.csv|data.json] [--delay ms]`
- 递归遍历请求；对每个请求：解析变量（含数据行）→ 发送 → 执行断言 → 记录结果。
- 输出：每请求 `status/时间/断言通过数/失败`；汇总 `total/passed/failed`；任一失败则以非零码退出（CI 友好）。
- 数据文件：CSV（首行表头）或 JSON 数组，每行一次迭代。

---

## 11. 断言引擎（替代 pm.test 的最小子集）

支持两种来源：
1. **声明式**（请求 JSON 内 `tests` 字段）：
   ```json
   "tests": [
     {"type":"status","equals":200},
     {"type":"json_path","path":"data.id","equals":"abc"},
     {"type":"body_contains","value":"success"},
     {"type":"header","key":"content-type","contains":"json"},
     {"type":"max_time_ms","value":2000}
   ]
   ```
2. **轻量解析**常见 `pm.test("...", () => pm.response.to.have.status(200))`：
   - 用正则识别 `status(<code>)`、`header("k","v")` 等简单形态进行近似断言；复杂 JS 逻辑标注「需脚本引擎（路线图）」。

> 说明：Postman 使用 Hermes 运行完整 JS。纯 Rust CLI 不内嵌 JS 引擎（避免体积与稳定性风险）；复杂脚本作为路线图（可选用 `boa`/`rquickjs` 集成）。此差异在 README 明确标注。

---

## 12. CLI 一览

```
pmrs send      发送单次请求
pmrs request   运行集合中的某个保存请求
pmrs collection  list/show/new/add
pmrs env        list/show/new/set
pmrs history    list/clear
pmrs import     导入 (postman|openapi|curl|har)
pmrs export     导出集合/环境
pmrs codegen    生成代码
pmrs run        运行集合（Runner）
pmrs ws         WebSocket 连接/收发（可选）
pmrs mock       本地 Mock 回放（可选）
```

---

## 13. 验收标准（Acceptance）

1. `cargo build --release` 成功，产出 `pmrs`（Win `.exe` / macOS/Linux 可执行）。
2. `cargo test` 全绿：
   - 变量解析（含动态变量、优先级、递归）；
   - Postman v2.1 导入/导出往返一致；
   - 各代码生成器输出非空且含关键片段；
   - 集成测试：本地起 HTTP 服务器，`send` 得到正确状态/体。
3. 端到端演示：
   - `pmrs import --format curl ...` 生成集合；
   - `pmrs run examples/demo_collection.json -e examples/demo_environment.json` 输出汇总且断言通过；
   - `pmrs codegen --lang rust` 输出可读 Rust 代码。
4. 「无条件」核验：
   - 全代码库 `grep -i "login\|token\|paywall\|subscription\|stripe\|license"` 在**业务逻辑**中无强制分支（README 例外说明）。
   - 所有命令无需任何账号/网络即可运行（除访问目标 API 本身）。

---

## 14. 与 Postman 的差异（诚实声明）

| 方面 | Postman | postman-rs |
|---|---|---|
| 完整 JS 脚本 | Hermes 全量 `pm.*` | 声明式断言 + 常见 `pm.test` 近似；完整 JS 为路线图 |
| 多协议 | WS/Socket.IO/MQTT/gRPC/GraphQL/SSE | 核心 HTTP 完整；WS 可选；其余为路线图 |
| 云端协作 | 有 | 无（按设计） |
| 计费/登录 | 有 | 无（按设计） |
| 兼容格式 | v2.1 | v2.1 兼容（双向） |
| 平台 | Win/mac/Linux | Win/mac/Linux（Rust 单二进制） |

本项目的价值在于：**把最常用的本地能力做成轻量、跨平台、无条件、可脚本化、可版本化的开源式工具**，而非复制云端商业服务。
