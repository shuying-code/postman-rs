use anyhow::{anyhow, Result}; // 引入依赖
use clap::{Parser, Subcommand}; // 引入依赖
use postman_rs::codegen; // 引入依赖
use postman_rs::http::{self, ResponseRecord}; // 引入依赖
use postman_rs::import_export::{self, Imported}; // 引入依赖
use postman_rs::model::*; // 引入依赖
use postman_rs::runner::{self, RunOptions}; // 引入依赖
use postman_rs::{mock, mqtt, perf, sse, ws}; // 引入依赖
use postman_rs::storage::{self, HistoryEntry}; // 引入依赖
use postman_rs::variables::VarContext; // 引入依赖
use std::collections::HashMap; // 引入依赖

#[derive(Parser)] // 语句
#[command( // 语句
    name = "pmrs", // 语句
    version, // 语句
    about = "postman-rs — local, account-free, payment-free Postman-style API client" // 语句
)] // 语句
struct Cli { // 结构体 Cli
    #[command(subcommand)] // 语句
    cmd: Cmd, // 字段 cmd
} // 块结束

#[derive(Subcommand)] // 语句
enum Cmd { // 枚举 Cmd
    /// Send a single HTTP request.
    Send(SendArgs), // 调用 Send
    /// Run a saved request from a collection (by name).
    Request(RequestArgs), // 调用 Request
    /// Collection operations.
    Collection { // 语句
        #[command(subcommand)] // 语句
        cmd: CollectionCmd, // 字段 cmd
    }, // 块结束
    /// Environment operations.
    Env { // 语句
        #[command(subcommand)] // 语句
        cmd: EnvCmd, // 字段 cmd
    }, // 块结束
    /// History operations.
    History { // 语句
        #[command(subcommand)] // 语句
        cmd: HistoryCmd, // 字段 cmd
    }, // 块结束
    /// Import a collection/environment (postman|openapi|curl|har|auto).
    Import(ImportArgs), // 调用 Import
    /// Export a collection file (re-normalized to v2.1).
    Export(ExportArgs), // 调用 Export
    /// Generate client code for a request.
    Codegen(CodegenArgs), // 调用 Codegen
    /// Run a collection (Collection Runner).
    Run(RunArgs), // 调用 Run
    /// Global variables.
    Globals { // 语句
        #[command(subcommand)] // 语句
        cmd: GlobalsCmd, // 字段 cmd
    }, // 块结束
    /// WebSocket client.
    Ws(WsArgs), // 调用 Ws
    /// Server-Sent Events client.
    Sse(SseArgs), // 调用 Sse
    /// MQTT client (plain TCP).
    Mqtt(MqttArgs), // 调用 Mqtt
    /// Send a GraphQL query over HTTP.
    Graphql(GraphqlArgs), // 调用 Graphql
    /// Run a local mock server replaying saved examples.
    Mock(MockArgs), // 调用 Mock
    /// Performance/load test.
    Perf(PerfArgs), // 调用 Perf
} // 块结束

#[derive(clap::Args)] // 语句
struct SendArgs { // 结构体 SendArgs
    /// Request URL (supports {{vars}}).
    url: String, // 字段 url
    #[arg(short = 'm', long, default_value = "GET")] // 语句
    method: String, // 字段 method
    /// Header, repeatable: -H "Key: Value" or -H "Key=Value".
    #[arg(short = 'H', long = "header")] // 语句
    header: Vec<String>, // 字段 header
    /// Body data.
    #[arg(short = 'd', long)] // 语句
    data: Option<String>, // 字段 data
    /// Body type: raw|json|urlencoded|form|file|graphql.
    #[arg(long, default_value = "raw")] // 语句
    body_type: String, // 字段 body_type
    /// Environment JSON file.
    #[arg(short = 'e', long)] // 语句
    env: Option<String>, // 字段 env
    /// Collection JSON file (for variables).
    #[arg(short = 'c', long)] // 语句
    collection: Option<String>, // 字段 collection
    /// Extra variable k=v, repeatable.
    #[arg(long = "var")] // 语句
    vars: Vec<String>, // 字段 vars
    /// Append this request to local history.
    #[arg(long)] // 语句
    save_history: bool, // 字段 save_history
    #[arg(long, default_value_t = 30)] // 语句
    timeout: u64, // 字段 timeout
} // 块结束

#[derive(clap::Args)] // 语句
struct RequestArgs { // 结构体 RequestArgs
    /// Collection JSON file.
    collection: String, // 字段 collection
    /// Request/item name (fuzzy contains match).
    name: String, // 字段 name
    #[arg(short = 'e', long)] // 语句
    env: Option<String>, // 字段 env
    #[arg(long = "var")] // 语句
    vars: Vec<String>, // 字段 vars
    #[arg(long)] // 语句
    save_history: bool, // 字段 save_history
    #[arg(long, default_value_t = 30)] // 语句
    timeout: u64, // 字段 timeout
} // 块结束

#[derive(Subcommand)] // 语句
enum CollectionCmd { // 枚举 CollectionCmd
    List, // 语句
    Show { path: String }, // 语句
    New { name: String }, // 语句
    AddFolder { // 语句
        collection: String, // 字段 collection
        name: String, // 字段 name
    }, // 块结束
    AddRequest { // 语句
        collection: String, // 字段 collection
        name: String, // 字段 name
        #[arg(short = 'm', long, default_value = "GET")] // 语句
        method: String, // 字段 method
        #[arg(short = 'u', long)] // 语句
        url: String, // 字段 url
        #[arg(long)] // 语句
        folder: Option<String>, // 字段 folder
    }, // 块结束
    Rm { // 语句
        collection: String, // 字段 collection
        name: String, // 字段 name
    }, // 块结束
    Rename { // 语句
        collection: String, // 字段 collection
        old: String, // 字段 old
        new: String, // 字段 new
    }, // 块结束
    Move { // 语句
        collection: String, // 字段 collection
        name: String, // 字段 name
        /// 目标文件夹（留空=移动到集合根）
        #[arg(long)] // 语句
        folder: Option<String>, // 字段 folder
    }, // 块结束
    Duplicate { // 语句
        collection: String, // 字段 collection
        name: String, // 字段 name
        #[arg(short = 'n', long)] // 语句
        new_name: Option<String>, // 字段 new_name
    }, // 块结束
    Export { // 语句
        collection: String, // 字段 collection
        #[arg(short = 'o', long)] // 语句
        output: Option<String>, // 字段 output
    }, // 块结束
} // 块结束

#[derive(Subcommand)] // 语句
enum EnvCmd { // 枚举 EnvCmd
    List, // 语句
    Show { path: String }, // 语句
    New { name: String }, // 语句
    Set { // 语句
        path: String, // 字段 path
        key: String, // 字段 key
        value: String, // 字段 value
    }, // 块结束
    Export { // 语句
        path: String, // 字段 path
        #[arg(short = 'o', long)] // 语句
        output: Option<String>, // 字段 output
    }, // 块结束
    Delete { // 语句
        path: String, // 字段 path
        key: String, // 字段 key
    }, // 块结束
} // 块结束

#[derive(Subcommand)] // 语句
enum GlobalsCmd { // 枚举 GlobalsCmd
    List, // 语句
    Show, // 语句
    Set { key: String, value: String }, // 语句
    Clear, // 语句
} // 块结束

#[derive(Subcommand)] // 语句
enum HistoryCmd { // 枚举 HistoryCmd
    List { // 语句
        #[arg(long, default_value_t = 50)] // 语句
        limit: usize, // 字段 limit
        /// 仅显示 URL 或方法包含该关键字的记录
        #[arg(long)] // 语句
        search: Option<String>, // 字段 search
    }, // 块结束
    Clear, // 语句
} // 块结束

#[derive(clap::Args)] // 语句
struct ImportArgs { // 结构体 ImportArgs
    file: String, // 字段 file
    #[arg(long, default_value = "auto")] // 语句
    format: String, // 字段 format
    /// Do not save to local workspace (just validate/print summary).
    #[arg(long)] // 语句
    no_save: bool, // 字段 no_save
} // 块结束

#[derive(clap::Args)] // 语句
struct ExportArgs { // 结构体 ExportArgs
    file: String, // 字段 file
    #[arg(short = 'o', long)] // 语句
    output: Option<String>, // 字段 output
} // 块结束

#[derive(clap::Args)] // 语句
struct CodegenArgs { // 结构体 CodegenArgs
    #[arg(short = 'l', long)] // 语句
    lang: String, // 字段 lang
    #[arg(short = 'c', long)] // 语句
    collection: Option<String>, // 字段 collection
    #[arg(short = 'i', long)] // 语句
    item: Option<String>, // 字段 item
    #[arg(short = 'u', long)] // 语句
    url: Option<String>, // 字段 url
    #[arg(short = 'm', long, default_value = "GET")] // 语句
    method: String, // 字段 method
} // 块结束

#[derive(clap::Args)] // 语句
struct RunArgs { // 结构体 RunArgs
    collection: String, // 字段 collection
    #[arg(short = 'e', long)] // 语句
    env: Option<String>, // 字段 env
    #[arg(short = 'i', long, default_value_t = 1)] // 语句
    iterations: usize, // 字段 iterations
    /// Data file (CSV or JSON array).
    #[arg(short = 'd', long)] // 语句
    data: Option<String>, // 字段 data
    #[arg(long, default_value_t = 0)] // 语句
    delay: u64, // 字段 delay
    #[arg(long, default_value_t = 30)] // 语句
    timeout: u64, // 字段 timeout
    /// Run only requests under this folder name.
    #[arg(long)] // 语句
    folder: Option<String>, // 字段 folder
    /// Stop on first failing request.
    #[arg(long)] // 语句
    bail: bool, // 字段 bail
    /// Write a JSON run report to this path.
    #[arg(long)] // 语句
    report: Option<String>, // 字段 report
} // 块结束

#[derive(clap::Args)] // 语句
struct WsArgs { // 结构体 WsArgs
    url: String, // 字段 url
    /// Message to send after connect (repeatable).
    #[arg(long = "send")] // 语句
    send: Vec<String>, // 字段 send
    /// Number of messages to receive before exiting.
    #[arg(long, default_value_t = 1)] // 语句
    listen: usize, // 字段 listen
    /// Maximum wait time for messages (ms).
    #[arg(long, default_value_t = 5000)] // 语句
    wait_ms: u64, // 字段 wait_ms
    /// Custom request header, "K: V" (repeatable).
    #[arg(short = 'H', long = "header")] // 语句
    header: Vec<String>, // 字段 header
    /// Value for the Sec-WebSocket-Protocol header.
    #[arg(long)] // 语句
    subprotocol: Option<String>, // 字段 subprotocol
    /// Send messages as binary frames instead of text.
    #[arg(long, default_value_t = false)] // 语句
    binary: bool, // 字段 binary
} // 块结束

#[derive(clap::Args)] // 语句
struct SseArgs { // 结构体 SseArgs
    url: String, // 字段 url
    #[arg(short = 'H', long = "header")] // 语句
    header: Vec<String>, // 字段 header
    /// Stop after N data events (0 = until stream ends).
    #[arg(long, default_value_t = 0)] // 语句
    count: usize, // 字段 count
    #[arg(long, default_value_t = 30)] // 语句
    timeout: u64, // 字段 timeout
} // 块结束

#[derive(clap::Args)] // 语句
struct MqttArgs { // 结构体 MqttArgs
    host: String, // 字段 host
    #[arg(short = 'p', long, default_value_t = 1883)] // 语句
    port: u16, // 字段 port
    /// Topic to subscribe to.
    topic: String, // 字段 topic
    /// Topic to publish to (optional).
    #[arg(long)] // 语句
    publish: Option<String>, // 字段 publish
    /// Payload for --publish.
    #[arg(long)] // 语句
    message: Option<String>, // 字段 message
    /// Number of messages to receive before exiting.
    #[arg(long, default_value_t = 1)] // 语句
    listen: usize, // 字段 listen
    #[arg(long, default_value_t = 10)] // 语句
    timeout: u64, // 字段 timeout
    /// Username for MQTT CONNECT.
    #[arg(long)] // 语句
    username: Option<String>, // 字段 username
    /// Password for MQTT CONNECT.
    #[arg(long)] // 语句
    password: Option<String>, // 字段 password
    /// Publish QoS (0 or 1).
    #[arg(long, default_value_t = 0)] // 语句
    qos: u8, // 字段 qos
    /// Client id for MQTT CONNECT.
    #[arg(long = "client-id")] // 语句
    client_id: Option<String>, // 字段 client_id
} // 块结束

#[derive(clap::Args)] // 语句
struct GraphqlArgs { // 结构体 GraphqlArgs
    url: String, // 字段 url
    #[arg(short = 'q', long)] // 语句
    query: String, // 字段 query
    #[arg(long)] // 语句
    variables: Option<String>, // 字段 variables
    #[arg(short = 'e', long)] // 语句
    env: Option<String>, // 字段 env
    #[arg(long, default_value_t = 30)] // 语句
    timeout: u64, // 字段 timeout
} // 块结束

#[derive(clap::Args)] // 语句
struct MockArgs { // 结构体 MockArgs
    collection: String, // 字段 collection
    #[arg(long, default_value_t = 8080)] // 语句
    port: u16, // 字段 port
    #[arg(long, default_value_t = 0)] // 语句
    delay: u64, // 字段 delay
} // 块结束

#[derive(clap::Args)] // 语句
struct PerfArgs { // 结构体 PerfArgs
    #[arg(short = 'c', long)] // 语句
    collection: Option<String>, // 字段 collection
    #[arg(short = 'i', long)] // 语句
    item: Option<String>, // 字段 item
    #[arg(short = 'u', long)] // 语句
    url: Option<String>, // 字段 url
    #[arg(short = 'm', long, default_value = "GET")] // 语句
    method: String, // 字段 method
    #[arg(long, default_value_t = 100)] // 语句
    iterations: usize, // 字段 iterations
    #[arg(long, default_value_t = 10)] // 语句
    concurrency: usize, // 字段 concurrency
    #[arg(short = 'e', long)] // 语句
    env: Option<String>, // 字段 env
    #[arg(long, default_value_t = 30)] // 语句
    timeout: u64, // 字段 timeout
} // 块结束

fn load_collection(path: &str) -> Result<Collection> { // 函数 load_collection
    let text = std::fs::read_to_string(path)?; // 变量 text
    let c: Collection = serde_json::from_str(&text)?; // 变量 c
    Ok(c) // 调用 Ok
} // 块结束

fn load_env(path: &str) -> Result<Environment> { // 函数 load_env
    let text = std::fs::read_to_string(path)?; // 变量 text
    let e: Environment = serde_json::from_str(&text)?; // 变量 e
    Ok(e) // 调用 Ok
} // 块结束

fn parse_headers(list: &[String]) -> Vec<Header> { // 函数 parse_headers
    list.iter() // 语句
        .filter_map(|h| { // 语句
            let (k, v) = if let Some((k, v)) = h.split_once(':') { // 调用 let
                (k, v) // 语句
            } else if let Some((k, v)) = h.split_once('=') { // 条件分支
                (k, v) // 语句
            } else { // 语句
                return None; // 返回
            }; // 块结束
            Some(Header { // 调用 Some
                key: k.trim().to_string(), // 字段 key
                value: v.trim().to_string(), // 字段 value
                ..Default::default() // 语句
            }) // 语句
        }) // 语句
        .collect() // 语句
} // 块结束

fn parse_vars(list: &[String]) -> HashMap<String, String> { // 函数 parse_vars
    let mut m = HashMap::new(); // 变量 m
    for kv in list { // 循环
        if let Some((k, v)) = kv.split_once('=') { // 条件分支
            m.insert(k.trim().to_string(), v.trim().to_string()); // 语句
        } // 块结束
    } // 块结束
    m // 语句
} // 块结束

fn build_ctx(collection: Option<&Collection>, env: Option<&Environment>, vars: &[String]) -> VarContext { // 函数 build_ctx
    let mut ctx = VarContext::new(); // 变量 ctx
    let coll_scope = collection // 变量 coll_scope
        .map(|c| { // 语句
            c.variable // 语句
                .iter() // 语句
                .filter(|v| v.is_enabled()) // 语句
                .map(|v| (v.key.clone(), value_str(&v.value))) // 语句
                .collect::<HashMap<_, _>>() // 语句
        }) // 语句
        .unwrap_or_default(); // 语句
    let env_scope = env // 变量 env_scope
        .map(|e| { // 语句
            e.values // 语句
                .iter() // 语句
                .filter(|v| v.is_enabled()) // 语句
                .map(|v| (v.key.clone(), value_str(&v.value))) // 语句
                .collect::<HashMap<_, _>>() // 语句
        }) // 语句
        .unwrap_or_default(); // 语句
    // priority: cli vars > env > collection
    ctx.scopes.push(parse_vars(vars)); // 语句
    if !env_scope.is_empty() { // 条件分支
        ctx.scopes.push(env_scope); // 语句
    } // 块结束
    if !coll_scope.is_empty() { // 条件分支
        ctx.scopes.push(coll_scope); // 语句
    } // 块结束
    // globals are the lowest-priority scope
    if let Ok(g) = storage::load_globals() { // 条件分支
        let gs: HashMap<String, String> = g // 变量 gs
            .values // 语句
            .iter() // 语句
            .filter(|v| v.is_enabled()) // 语句
            .map(|v| (v.key.clone(), value_str(&v.value))) // 语句
            .collect(); // 语句
        if !gs.is_empty() { // 条件分支
            ctx.scopes.push(gs); // 语句
        } // 块结束
    } // 块结束
    ctx // 语句
} // 块结束

/// Save an imported collection; if one with the same name already exists, merge into it
/// instead of overwriting (prevents silent data loss).
fn save_or_merge_collection(c: Collection) -> Result<std::path::PathBuf> { // 函数 save_or_merge_collection
    let name = c.info.name.clone(); // 变量 name
    for p in storage::list_collections().unwrap_or_default() { // 循环
        if let Ok(text) = std::fs::read_to_string(&p) { // 条件分支
            if let Ok(mut exist) = serde_json::from_str::<Collection>(&text) { // 条件分支
                if exist.info.name == name { // 条件分支
                    let before = exist.item.len(); // 变量 before
                    import_export::merge_collections(&mut exist, c); // 调用 merge_collections
                    let saved = storage::save_collection(&exist)?; // 变量 saved
                    println!( // 语句
                        "merged into existing collection '{}' (+{} items -> {})", // 语句
                        name, // 语句
                        exist.item.len() - before, // 语句
                        saved.display() // 语句
                    ); // 块结束
                    return Ok(saved); // 返回
                } // 块结束
            } // 块结束
        } // 块结束
    } // 块结束
    storage::save_collection(&c) // 调用 save_collection
} // 块结束

fn value_str(v: &serde_json::Value) -> String { // 函数 value_str
    match v { // 分支匹配
        serde_json::Value::String(s) => s.clone(), // 匹配分支
        serde_json::Value::Null => String::new(), // 匹配分支
        other => other.to_string(), // 匹配分支
    } // 块结束
} // 块结束

fn build_body(args: &SendArgs) -> Option<Body> { // 函数 build_body
    let data = args.data.clone()?; // 变量 data
    match args.body_type.as_str() { // 分支匹配
        "json" => Some(Body { // 匹配分支
            mode: "raw".into(), // 字段 mode
            raw: Some(data), // 字段 raw
            options: Some(serde_json::json!({"raw":{"language":"json"}})), // 字段 options
            ..Default::default() // 语句
        }), // 语句
        "raw" => Some(Body { // 匹配分支
            mode: "raw".into(), // 字段 mode
            raw: Some(data), // 字段 raw
            options: Some(serde_json::json!({"raw":{"language":"text"}})), // 字段 options
            ..Default::default() // 语句
        }), // 语句
        "urlencoded" => { // 匹配分支
            let pairs = data // 变量 pairs
                .split('&') // 语句
                .filter_map(|kv| kv.split_once('=')) // 语句
                .map(|(k, v)| KV { // 语句
                    key: k.to_string(), // 字段 key
                    value: v.to_string(), // 字段 value
                    ..Default::default() // 语句
                }) // 语句
                .collect(); // 语句
            Some(Body { // 调用 Some
                mode: "urlencoded".into(), // 字段 mode
                urlencoded: pairs, // 字段 urlencoded
                ..Default::default() // 语句
            }) // 语句
        } // 块结束
        "form" => { // 匹配分支
            let pairs = data // 变量 pairs
                .split('&') // 语句
                .filter_map(|kv| kv.split_once('=')) // 语句
                .map(|(k, v)| KV { // 语句
                    key: k.to_string(), // 字段 key
                    value: v.to_string(), // 字段 value
                    ..Default::default() // 语句
                }) // 语句
                .collect(); // 语句
            Some(Body { // 调用 Some
                mode: "formdata".into(), // 字段 mode
                formdata: pairs, // 字段 formdata
                ..Default::default() // 语句
            }) // 语句
        } // 块结束
        "file" => Some(Body { // 匹配分支
            mode: "file".into(), // 字段 mode
            file: Some(FileRef { // 语句
                src: Some(data), // 字段 src
                content: None, // 字段 content
            }), // 语句
            ..Default::default() // 语句
        }), // 语句
        "graphql" => Some(Body { // 匹配分支
            mode: "graphql".into(), // 字段 mode
            graphql: Some(GraphQL { // 语句
                query: data, // 字段 query
                variables: String::new(), // 字段 variables
            }), // 语句
            ..Default::default() // 语句
        }), // 语句
        other => { // 匹配分支
            eprintln!("warning: unknown body-type '{}', using raw", other); // 语句
            Some(Body { // 调用 Some
                mode: "raw".into(), // 字段 mode
                raw: Some(data), // 字段 raw
                ..Default::default() // 语句
            }) // 语句
        } // 块结束
    } // 块结束
} // 块结束

fn print_response(rec: &ResponseRecord) { // 函数 print_response
    println!("{} {}  ({}ms)", rec.code, rec.status, rec.elapsed_ms); // 语句
    println!("{}", rec.final_url); // 语句
    println!("----- headers -----"); // 语句
    for (k, v) in &rec.headers { // 循环
        println!("{}: {}", k, v); // 语句
    } // 块结束
    println!("----- body -----"); // 语句
    println!("{}", rec.body); // 语句
} // 块结束

fn find_request(items: &[Item], name: &str) -> Option<(String, Request, Vec<Event>)> { // 函数 find_request
    for item in items { // 循环
        if let Some(req) = &item.request { // 条件分支
            if item.name.to_lowercase().contains(&name.to_lowercase()) { // 条件分支
                return Some((item.name.clone(), req.clone(), item.event.clone())); // 返回
            } // 块结束
        } // 块结束
        if let Some(children) = &item.item { // 条件分支
            if let Some(found) = find_request(children, name) { // 条件分支
                return Some(found); // 返回
            } // 块结束
        } // 块结束
    } // 块结束
    None // 语句
} // 块结束

fn find_mut<'a>(items: &'a mut [Item], name: &str) -> Option<&'a mut Item> { // 函数 find_mut
    for it in items.iter_mut() { // 循环
        if it.name == name { // 条件分支
            return Some(it); // 返回
        } // 块结束
        if let Some(children) = it.item.as_mut() { // 条件分支
            if let Some(f) = find_mut(children, name) { // 条件分支
                return Some(f); // 返回
            } // 块结束
        } // 块结束
    } // 块结束
    None // 语句
} // 块结束
fn remove_rec(items: &mut Vec<Item>, name: &str) -> bool { // 函数 remove_rec
    if let Some(pos) = items.iter().position(|x| x.name == name) { // 条件分支
        items.remove(pos); // 语句
        return true; // 返回
    } // 块结束
    for it in items.iter_mut() { // 循环
        if let Some(children) = it.item.as_mut() { // 条件分支
            if remove_rec(children, name) { // 条件分支
                return true; // 返回
            } // 块结束
        } // 块结束
    } // 块结束
    false // 语句
} // 块结束
fn rename_rec(items: &mut Vec<Item>, old: &str, new: &str) -> bool { // 函数 rename_rec
    for it in items.iter_mut() { // 循环
        if it.name == old { // 条件分支
            it.name = new.to_string(); // 语句
            return true; // 返回
        } // 块结束
        if let Some(children) = it.item.as_mut() { // 条件分支
            if rename_rec(children, old, new) { // 条件分支
                return true; // 返回
            } // 块结束
        } // 块结束
    } // 块结束
    false // 语句
} // 块结束

fn take_rec(items: &mut Vec<Item>, name: &str) -> Option<Item> { // 函数 take_rec
    if let Some(pos) = items.iter().position(|x| x.name == name) { // 条件分支
        return Some(items.remove(pos)); // 返回
    } // 块结束
    for it in items.iter_mut() { // 循环
        if let Some(children) = it.item.as_mut() { // 条件分支
            if let Some(found) = take_rec(children, name) { // 条件分支
                return Some(found); // 返回
            } // 块结束
        } // 块结束
    } // 块结束
    None // 语句
} // 块结束

fn find_clone(items: &[Item], name: &str) -> Option<Item> { // 函数 find_clone
    for it in items { // 循环
        if it.name == name { // 条件分支
            return Some(it.clone()); // 返回
        } // 块结束
        if let Some(children) = &it.item { // 条件分支
            if let Some(found) = find_clone(children, name) { // 条件分支
                return Some(found); // 返回
            } // 块结束
        } // 块结束
    } // 块结束
    None // 语句
} // 块结束

fn build_perf_request(args: &PerfArgs) -> Result<(Request, VarContext)> { // 函数 build_perf_request
    let env = args.env.as_deref().map(load_env).transpose()?; // 变量 env
    if let (Some(cp), Some(item)) = (args.collection.as_ref(), args.item.as_ref()) { // 条件分支
        let c = load_collection(cp)?; // 变量 c
        let (_, mut req, _) = // 调用 let
            find_request(&c.item, item).ok_or_else(|| anyhow!("request not found: {}", item))?; // 调用 find_request
        if req.auth.is_none() { // 条件分支
            req.auth = c.auth.clone(); // 语句
        } // 块结束
        let ctx = build_ctx(Some(&c), env.as_ref(), &[]); // 变量 ctx
        Ok((req, ctx)) // 调用 Ok
    } else { // 语句
        let url = args // 变量 url
            .url // 语句
            .clone() // 语句
            .ok_or_else(|| anyhow!("provide --collection + --item, or --url"))?; // 语句
        let req = Request { // 变量 req
            method: args.method.clone(), // 字段 method
            url: UrlSpec::new(url), // 字段 url
            ..Default::default() // 语句
        }; // 块结束
        let ctx = build_ctx(None, env.as_ref(), &[]); // 变量 ctx
        Ok((req, ctx)) // 调用 Ok
    } // 块结束
} // 块结束

fn main() -> Result<()> { // 函数 main
    let cli = Cli::parse(); // 变量 cli
    match cli.cmd { // 分支匹配
        Cmd::Send(args) => { // 匹配分支
            let collection = args.collection.as_deref().map(load_collection).transpose()?; // 变量 collection
            let env = args.env.as_deref().map(load_env).transpose()?; // 变量 env
            let ctx = build_ctx(collection.as_ref(), env.as_ref(), &args.vars); // 变量 ctx
            let req = Request { // 变量 req
                method: args.method.clone(), // 字段 method
                url: UrlSpec::new(&args.url), // 字段 url
                header: parse_headers(&args.header), // 字段 header
                body: build_body(&args), // 字段 body
                ..Default::default() // 语句
            }; // 块结束
            let rec = http::execute(&req, &ctx, args.timeout)?; // 变量 rec
            print_response(&rec); // 调用 print_response
            if args.save_history { // 条件分支
                storage::append_history(&HistoryEntry { // 调用 append_history
                    ts: chrono::Utc::now().to_rfc3339(), // 字段 ts
                    method: req.method.to_uppercase(), // 字段 method
                    url: ctx.resolve(&req.url.effective_url()), // 字段 url
                    status: rec.code, // 字段 status
                    elapsed_ms: rec.elapsed_ms, // 字段 elapsed_ms
                })?; // 语句
            } // 块结束
        } // 块结束
        Cmd::Request(args) => { // 匹配分支
            let collection = load_collection(&args.collection)?; // 变量 collection
            let env = args.env.as_deref().map(load_env).transpose()?; // 变量 env
            let (_, mut req, _events) = find_request(&collection.item, &args.name) // 调用 let
                .ok_or_else(|| anyhow!("request not found: {}", args.name))?; // 语句
            if let Some(a) = &collection.auth { // 条件分支
                if req.auth.is_none() { // 条件分支
                    req.auth = Some(a.clone()); // 语句
                } // 块结束
            } // 块结束
            let ctx = build_ctx(Some(&collection), env.as_ref(), &args.vars); // 变量 ctx
            let rec = http::execute(&req, &ctx, args.timeout)?; // 变量 rec
            print_response(&rec); // 调用 print_response
            if args.save_history { // 条件分支
                storage::append_history(&HistoryEntry { // 调用 append_history
                    ts: chrono::Utc::now().to_rfc3339(), // 字段 ts
                    method: req.method.to_uppercase(), // 字段 method
                    url: ctx.resolve(&req.url.effective_url()), // 字段 url
                    status: rec.code, // 字段 status
                    elapsed_ms: rec.elapsed_ms, // 字段 elapsed_ms
                })?; // 语句
            } // 块结束
        } // 块结束
        Cmd::Collection { cmd } => match cmd { // 匹配分支
            CollectionCmd::List => { // 匹配分支
                for p in storage::list_collections()? { // 循环
                    println!("{}", p.display()); // 语句
                } // 块结束
            } // 块结束
            CollectionCmd::Show { path } => { // 匹配分支
                let c = load_collection(&path)?; // 变量 c
                println!("{}", serde_json::to_string_pretty(&c)?); // 语句
            } // 块结束
            CollectionCmd::New { name } => { // 匹配分支
                let c = Collection { // 变量 c
                    info: CollectionInfo { // 语句
                        name: name.clone(), // 字段 name
                        postman_id: Some(uuid::Uuid::new_v4().to_string()), // 字段 postman_id
                        schema: Some( // 语句
                            "https://schema.getpostman.com/json/collection/v2.1.0/collection.json".into(), // 语句
                        ), // 语句
                        description: None, // 字段 description
                    }, // 块结束
                    ..Default::default() // 语句
                }; // 块结束
                let p = storage::save_collection(&c)?; // 变量 p
                println!("created collection: {}", p.display()); // 语句
            } // 块结束
            CollectionCmd::AddFolder { collection, name } => { // 匹配分支
                let mut c = load_collection(&collection)?; // 变量 c
                c.item.push(Item { // 语句
                    name: name.clone(), // 字段 name
                    item: Some(Vec::new()), // 字段 item
                    ..Default::default() // 语句
                }); // 语句
                let p = storage::save_collection(&c)?; // 变量 p
                println!("added folder '{}' -> {}", name, p.display()); // 语句
            } // 块结束
            CollectionCmd::AddRequest { // 语句
                collection, // 语句
                name, // 语句
                method, // 语句
                url, // 语句
                folder, // 语句
            } => { // 匹配分支
                let mut c = load_collection(&collection)?; // 变量 c
                let item = Item { // 变量 item
                    name: name.clone(), // 字段 name
                    request: Some(Request { // 语句
                        method, // 语句
                        url: UrlSpec::new(url), // 字段 url
                        ..Default::default() // 语句
                    }), // 语句
                    ..Default::default() // 语句
                }; // 块结束
                match folder { // 分支匹配
                    Some(fd) => { // 匹配分支
                        if let Some(f) = find_mut(&mut c.item, &fd) { // 条件分支
                            if f.item.is_none() { // 条件分支
                                f.item = Some(Vec::new()); // 语句
                            } // 块结束
                            f.item.as_mut().unwrap().push(item); // 语句
                        } else { // 语句
                            return Err(anyhow!("folder not found: {}", fd)); // 返回
                        } // 块结束
                    } // 块结束
                    None => c.item.push(item), // 匹配分支
                } // 块结束
                let p = storage::save_collection(&c)?; // 变量 p
                println!("added request '{}' -> {}", name, p.display()); // 语句
            } // 块结束
            CollectionCmd::Rm { collection, name } => { // 匹配分支
                let mut c = load_collection(&collection)?; // 变量 c
                if remove_rec(&mut c.item, &name) { // 条件分支
                    let p = storage::save_collection(&c)?; // 变量 p
                    println!("removed '{}' -> {}", name, p.display()); // 语句
                } else { // 语句
                    return Err(anyhow!("item not found: {}", name)); // 返回
                } // 块结束
            } // 块结束
            CollectionCmd::Rename { // 语句
                collection, // 语句
                old, // 语句
                new, // 语句
            } => { // 匹配分支
                let mut c = load_collection(&collection)?; // 变量 c
                if rename_rec(&mut c.item, &old, &new) { // 条件分支
                    let p = storage::save_collection(&c)?; // 变量 p
                    println!("renamed '{}' -> '{}' ({})", old, new, p.display()); // 语句
                } else { // 语句
                    return Err(anyhow!("item not found: {}", old)); // 返回
                } // 块结束
            } // 块结束
            CollectionCmd::Move { // 语句
                collection, // 语句
                name, // 语句
                folder, // 语句
            } => { // 匹配分支
                let mut c = load_collection(&collection)?; // 变量 c
                let Some(item) = take_rec(&mut c.item, &name) else { // 变量 Some
                    return Err(anyhow!("item not found: {}", name)); // 返回
                }; // 块结束
                match folder { // 分支匹配
                    Some(fd) => { // 匹配分支
                        if let Some(f) = find_mut(&mut c.item, &fd) { // 条件分支
                            if f.item.is_none() { // 条件分支
                                f.item = Some(Vec::new()); // 语句
                            } // 块结束
                            f.item.as_mut().unwrap().push(item); // 语句
                        } else { // 语句
                            return Err(anyhow!("folder not found: {}", fd)); // 返回
                        } // 块结束
                    } // 块结束
                    None => c.item.push(item), // 匹配分支
                } // 块结束
                let p = storage::save_collection(&c)?; // 变量 p
                println!("moved '{}' -> {}", name, p.display()); // 语句
            } // 块结束
            CollectionCmd::Duplicate { // 语句
                collection, // 语句
                name, // 语句
                new_name, // 语句
            } => { // 匹配分支
                let mut c = load_collection(&collection)?; // 变量 c
                let Some(mut it) = find_clone(&c.item, &name) else { // 变量 Some
                    return Err(anyhow!("item not found: {}", name)); // 返回
                }; // 块结束
                it.name = new_name.unwrap_or_else(|| format!("{} 副本", it.name)); // 语句
                let nn = it.name.clone(); // 变量 nn
                c.item.push(it); // 语句
                let p = storage::save_collection(&c)?; // 变量 p
                println!("duplicated '{}' as '{}' -> {}", name, nn, p.display()); // 语句
            } // 块结束
            CollectionCmd::Export { collection, output } => { // 匹配分支
                let mut c = load_collection(&collection)?; // 变量 c
                let text = import_export::export_collection(&mut c)?; // 变量 text
                match output { // 分支匹配
                    Some(o) => { // 匹配分支
                        std::fs::write(&o, text)?; // 调用 write
                        println!("exported collection: {}", o); // 语句
                    } // 块结束
                    None => println!("{}", text), // 匹配分支
                } // 块结束
            } // 块结束
        }, // 块结束
        Cmd::Env { cmd } => match cmd { // 匹配分支
            EnvCmd::List => { // 匹配分支
                for p in storage::list_environments()? { // 循环
                    println!("{}", p.display()); // 语句
                } // 块结束
            } // 块结束
            EnvCmd::Show { path } => { // 匹配分支
                let e = load_env(&path)?; // 变量 e
                println!("{}", serde_json::to_string_pretty(&e)?); // 语句
            } // 块结束
            EnvCmd::New { name } => { // 匹配分支
                let e = Environment { // 变量 e
                    id: Some(uuid::Uuid::new_v4().to_string()), // 字段 id
                    name, // 语句
                    values: Vec::new(), // 字段 values
                    scope: Some("environment".into()), // 字段 scope
                }; // 块结束
                let p = storage::save_environment(&e)?; // 变量 p
                println!("created environment: {}", p.display()); // 语句
            } // 块结束
            EnvCmd::Set { path, key, value } => { // 匹配分支
                let mut e = load_env(&path)?; // 变量 e
                if let Some(v) = e.values.iter_mut().find(|v| v.key == key) { // 条件分支
                    v.value = serde_json::Value::String(value); // 语句
                } else { // 语句
                    e.values.push(Variable { // 语句
                        key, // 语句
                        value: serde_json::Value::String(value), // 字段 value
                        typ: Some("default".into()), // 字段 typ
                        enabled: Some(true), // 字段 enabled
                        description: None, // 字段 description
                    }); // 语句
                } // 块结束
                std::fs::write(&path, serde_json::to_string_pretty(&e)?)?; // 调用 write
                println!("updated {}", path); // 语句
            } // 块结束
            EnvCmd::Export { path, output } => { // 匹配分支
                let mut e = load_env(&path)?; // 变量 e
                let text = import_export::export_environment(&mut e)?; // 变量 text
                match output { // 分支匹配
                    Some(o) => { // 匹配分支
                        std::fs::write(&o, text)?; // 调用 write
                        println!("exported environment: {}", o); // 语句
                    } // 块结束
                    None => println!("{}", text), // 匹配分支
                } // 块结束
            } // 块结束
            EnvCmd::Delete { path, key } => { // 匹配分支
                let mut e = load_env(&path)?; // 变量 e
                let before = e.values.len(); // 变量 before
                e.values.retain(|v| v.key != key); // 语句
                if e.values.len() == before { // 条件分支
                    return Err(anyhow!("key not found: {}", key)); // 返回
                } // 块结束
                std::fs::write(&path, serde_json::to_string_pretty(&e)?)?; // 调用 write
                println!("deleted key '{}' from {}", key, path); // 语句
            } // 块结束
        }, // 块结束
        Cmd::History { cmd } => match cmd { // 匹配分支
            HistoryCmd::List { limit, search } => { // 匹配分支
                let h = storage::read_history()?; // 变量 h
                let q = search.unwrap_or_default().to_lowercase(); // 变量 q
                for e in h // 循环
                    .iter() // 语句
                    .rev() // 语句
                    .filter(|e| { // 语句
                        q.is_empty() // 语句
                            || e.url.to_lowercase().contains(&q) // 语句
                            || e.method.to_lowercase().contains(&q) // 语句
                    }) // 语句
                    .take(limit) // 语句
                { // 语句
                    println!("{}  {} {} -> {}", e.ts, e.method, e.url, e.status); // 语句
                } // 块结束
            } // 块结束
            HistoryCmd::Clear => { // 匹配分支
                storage::clear_history()?; // 调用 clear_history
                println!("history cleared"); // 语句
            } // 块结束
        }, // 块结束
        Cmd::Import(args) => { // 匹配分支
            // For cURL, accept either a file path or an inline command string.
            let imported = if args.format.eq_ignore_ascii_case("curl") // 变量 imported
                && !std::path::Path::new(&args.file).exists() // 语句
            { // 语句
                Imported::Collection(Box::new(import_export::import_curl(&args.file))) // 调用 Collection
            } else { // 语句
                import_export::import_file(&args.file, &args.format)? // 调用 import_file
            }; // 块结束
            match imported { // 分支匹配
                Imported::Collection(mut c) => { // 匹配分支
                    println!( // 语句
                        "imported collection '{}' with {} top-level items", // 语句
                        c.info.name, // 语句
                        c.item.len() // 语句
                    ); // 块结束
                    if !args.no_save { // 条件分支
                        let p = save_or_merge_collection(*c)?; // 变量 p
                        println!("saved: {}", p.display()); // 语句
                    } else { // 语句
                        let _ = import_export::export_collection(&mut c); // 变量 _
                    } // 块结束
                } // 块结束
                Imported::Environment(e) => { // 匹配分支
                    println!("imported environment '{}' with {} values", e.name, e.values.len()); // 语句
                    if !args.no_save { // 条件分支
                        let p = storage::save_environment(&e)?; // 变量 p
                        println!("saved: {}", p.display()); // 语句
                    } // 块结束
                } // 块结束
            } // 块结束
        } // 块结束
        Cmd::Export(args) => { // 匹配分支
            let mut c = load_collection(&args.file)?; // 变量 c
            let out = import_export::export_collection(&mut c)?; // 变量 out
            match args.output { // 分支匹配
                Some(o) => { // 匹配分支
                    std::fs::write(&o, out)?; // 调用 write
                    println!("exported: {}", o); // 语句
                } // 块结束
                None => println!("{}", out), // 匹配分支
            } // 块结束
        } // 块结束
        Cmd::Codegen(args) => { // 匹配分支
            let req = if let (Some(cp), Some(item)) = (args.collection.as_ref(), args.item.as_ref()) { // 变量 req
                let c = load_collection(cp)?; // 变量 c
                find_request(&c.item, item) // 调用 find_request
                    .map(|(_, r, _)| r) // 语句
                    .ok_or_else(|| anyhow!("request not found: {}", item))? // 语句
            } else { // 语句
                let url = args.url.clone().ok_or_else(|| { // 变量 url
                    anyhow!("provide --collection + --item, or --url for direct codegen") // 语句
                })?; // 语句
                Request { // 语句
                    method: args.method.clone(), // 字段 method
                    url: UrlSpec::new(url), // 字段 url
                    ..Default::default() // 语句
                } // 块结束
            }; // 块结束
            println!("{}", codegen::generate(&req, &args.lang)?); // 语句
        } // 块结束
        Cmd::Run(args) => { // 匹配分支
            let collection = load_collection(&args.collection)?; // 变量 collection
            let env = args.env.as_deref().map(load_env).transpose()?; // 变量 env
            let data_rows = match args.data.as_deref() { // 变量 data_rows
                Some(d) => runner::load_data_file(d)?, // 匹配分支
                None => Vec::new(), // 匹配分支
            }; // 块结束
            let summary = runner::run_collection_opts( // 变量 summary
                &collection, // 语句
                env.as_ref(), // 语句
                args.iterations, // 语句
                data_rows, // 语句
                args.delay, // 语句
                args.timeout, // 语句
                RunOptions { // 语句
                    folder: args.folder.clone(), // 字段 folder
                    bail: args.bail, // 字段 bail
                    report: args.report.clone(), // 字段 report
                }, // 块结束
            ); // 块结束
            for r in &summary.results { // 循环
                let status = match r.code { // 变量 status
                    Some(c) => c.to_string(), // 匹配分支
                    None => "ERR".to_string(), // 匹配分支
                }; // 块结束
                println!( // 语句
                    "[{}] {} {} {} ({}ms) tests: {}/{}", // 语句
                    status, // 语句
                    r.method, // 语句
                    r.url, // 语句
                    if r.passed { "PASS" } else { "FAIL" }, // 条件分支
                    r.elapsed_ms, // 语句
                    r.tests.iter().filter(|t| t.passed).count(), // 语句
                    r.tests.len() // 语句
                ); // 块结束
                if let Some(err) = &r.error { // 条件分支
                    println!("    error: {}", err); // 语句
                } // 块结束
                for t in &r.tests { // 循环
                    if !t.passed { // 条件分支
                        println!("    FAILED: {} ({})", t.name, t.detail); // 语句
                    } // 块结束
                } // 块结束
            } // 块结束
            println!( // 语句
                "\nSummary: total={} passed={} failed={}", // 语句
                summary.total, summary.passed, summary.failed // 语句
            ); // 块结束
            if summary.failed > 0 { // 条件分支
                std::process::exit(1); // 调用 exit
            } // 块结束
        } // 块结束
        Cmd::Globals { cmd } => match cmd { // 匹配分支
            GlobalsCmd::List | GlobalsCmd::Show => { // 匹配分支
                let g = storage::load_globals()?; // 变量 g
                for v in &g.values { // 循环
                    println!("{}={}", v.key, value_str(&v.value)); // 语句
                } // 块结束
            } // 块结束
            GlobalsCmd::Set { key, value } => { // 匹配分支
                let mut g = storage::load_globals()?; // 变量 g
                if let Some(v) = g.values.iter_mut().find(|v| v.key == key) { // 条件分支
                    v.value = serde_json::Value::String(value); // 语句
                } else { // 语句
                    g.values.push(Variable { // 语句
                        key, // 语句
                        value: serde_json::Value::String(value), // 字段 value
                        typ: Some("default".into()), // 字段 typ
                        enabled: Some(true), // 字段 enabled
                        description: None, // 字段 description
                    }); // 语句
                } // 块结束
                let p = storage::save_globals(&g)?; // 变量 p
                println!("updated globals: {}", p.display()); // 语句
            } // 块结束
            GlobalsCmd::Clear => { // 匹配分支
                let mut g = storage::load_globals()?; // 变量 g
                g.values.clear(); // 语句
                let p = storage::save_globals(&g)?; // 变量 p
                println!("cleared globals: {}", p.display()); // 语句
            } // 块结束
        }, // 块结束
        Cmd::Ws(args) => { // 匹配分支
            let headers: Vec<(String, String)> = parse_headers(&args.header) // 变量 headers
                .into_iter() // 语句
                .map(|h| (h.key, h.value)) // 语句
                .collect(); // 语句
            let cfg = ws::WsConfig { // 变量 cfg
                url: args.url, // 字段 url
                messages: args.send, // 字段 messages
                listen: args.listen, // 字段 listen
                wait_ms: args.wait_ms, // 字段 wait_ms
                headers, // 语句
                subprotocol: args.subprotocol, // 字段 subprotocol
                binary: args.binary, // 字段 binary
            }; // 块结束
            for line in ws::run_collect(&cfg)? { // 循环
                println!("{}", line); // 语句
            } // 块结束
        } // 块结束
        Cmd::Sse(args) => { // 匹配分支
            let headers: Vec<(String, String)> = parse_headers(&args.header) // 变量 headers
                .into_iter() // 语句
                .map(|h| (h.key, h.value)) // 语句
                .collect(); // 语句
            sse::run(&args.url, &headers, args.count, args.timeout)?; // 调用 run
        } // 块结束
        Cmd::Mqtt(args) => { // 匹配分支
            let publish = match (args.publish, args.message) { // 变量 publish
                (Some(t), Some(m)) => Some((t, m)), // 匹配分支
                (Some(t), None) => Some((t, String::new())), // 匹配分支
                _ => None, // 匹配分支
            }; // 块结束
            let pub_ref = publish // 变量 pub_ref
                .as_ref() // 语句
                .map(|(t, m)| (t.as_str(), m.as_str(), args.qos)); // 语句
            let cfg = mqtt::MqttConfig { // 变量 cfg
                host: &args.host, // 字段 host
                port: args.port, // 字段 port
                client_id: args.client_id.as_deref(), // 字段 client_id
                username: args.username.as_deref(), // 字段 username
                password: args.password.as_deref(), // 字段 password
                topic: &args.topic, // 字段 topic
                publish: pub_ref, // 字段 publish
                qos: args.qos, // 字段 qos
                listen: args.listen, // 字段 listen
                timeout_secs: args.timeout, // 字段 timeout_secs
            }; // 块结束
            for line in mqtt::run_advanced(&cfg)? { // 循环
                println!("{}", line); // 语句
            } // 块结束
        } // 块结束
        Cmd::Graphql(args) => { // 匹配分支
            let env = args.env.as_deref().map(load_env).transpose()?; // 变量 env
            let ctx = build_ctx(None, env.as_ref(), &[]); // 变量 ctx
            let req = Request { // 变量 req
                method: "POST".into(), // 字段 method
                url: UrlSpec::new(args.url), // 字段 url
                body: Some(Body { // 语句
                    mode: "graphql".into(), // 字段 mode
                    graphql: Some(GraphQL { // 语句
                        query: args.query, // 字段 query
                        variables: args.variables.unwrap_or_default(), // 字段 variables
                    }), // 语句
                    ..Default::default() // 语句
                }), // 语句
                ..Default::default() // 语句
            }; // 块结束
            let rec = http::execute(&req, &ctx, args.timeout)?; // 变量 rec
            print_response(&rec); // 调用 print_response
        } // 块结束
        Cmd::Mock(args) => { // 匹配分支
            mock::serve(&args.collection, args.port, args.delay)?; // 调用 serve
        } // 块结束
        Cmd::Perf(args) => { // 匹配分支
            let (req, ctx) = build_perf_request(&args)?; // 调用 let
            let report = perf::run(&req, &ctx, args.iterations, args.concurrency, args.timeout); // 变量 report
            println!("iterations: {}  concurrency: {}", report.total, args.concurrency); // 语句
            println!("ok: {}  err: {}  rps: {:.1}", report.ok, report.err, report.rps); // 语句
            println!( // 语句
                "latency ms  mean: {:.1}  p50: {}  p90: {}  p99: {}  max: {}", // 语句
                report.mean_ms, report.p50, report.p90, report.p99, report.max_ms // 语句
            ); // 块结束
        } // 块结束
    } // 块结束
    Ok(()) // 调用 Ok
} // 块结束

