use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use postman_rs::codegen;
use postman_rs::http::{self, ResponseRecord};
use postman_rs::import_export::{self, Imported};
use postman_rs::model::*;
use postman_rs::runner::{self, RunOptions};
use postman_rs::{mock, mqtt, perf, sse, ws};
use postman_rs::storage::{self, HistoryEntry};
use postman_rs::variables::VarContext;
use std::collections::HashMap;

#[derive(Parser)]
#[command(
    name = "pmrs",
    version,
    about = "postman-rs — local, account-free, payment-free Postman-style API client"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Send a single HTTP request.
    Send(SendArgs),
    /// Run a saved request from a collection (by name).
    Request(RequestArgs),
    /// Collection operations.
    Collection {
        #[command(subcommand)]
        cmd: CollectionCmd,
    },
    /// Environment operations.
    Env {
        #[command(subcommand)]
        cmd: EnvCmd,
    },
    /// History operations.
    History {
        #[command(subcommand)]
        cmd: HistoryCmd,
    },
    /// Import a collection/environment (postman|openapi|curl|har|auto).
    Import(ImportArgs),
    /// Export a collection file (re-normalized to v2.1).
    Export(ExportArgs),
    /// Generate client code for a request.
    Codegen(CodegenArgs),
    /// Run a collection (Collection Runner).
    Run(RunArgs),
    /// Global variables.
    Globals {
        #[command(subcommand)]
        cmd: GlobalsCmd,
    },
    /// WebSocket client.
    Ws(WsArgs),
    /// Server-Sent Events client.
    Sse(SseArgs),
    /// MQTT client (plain TCP).
    Mqtt(MqttArgs),
    /// Send a GraphQL query over HTTP.
    Graphql(GraphqlArgs),
    /// Run a local mock server replaying saved examples.
    Mock(MockArgs),
    /// Performance/load test.
    Perf(PerfArgs),
}

#[derive(clap::Args)]
struct SendArgs {
    /// Request URL (supports {{vars}}).
    url: String,
    #[arg(short = 'm', long, default_value = "GET")]
    method: String,
    /// Header, repeatable: -H "Key: Value" or -H "Key=Value".
    #[arg(short = 'H', long = "header")]
    header: Vec<String>,
    /// Body data.
    #[arg(short = 'd', long)]
    data: Option<String>,
    /// Body type: raw|json|urlencoded|form|file|graphql.
    #[arg(long, default_value = "raw")]
    body_type: String,
    /// Environment JSON file.
    #[arg(short = 'e', long)]
    env: Option<String>,
    /// Collection JSON file (for variables).
    #[arg(short = 'c', long)]
    collection: Option<String>,
    /// Extra variable k=v, repeatable.
    #[arg(long = "var")]
    vars: Vec<String>,
    /// Append this request to local history.
    #[arg(long)]
    save_history: bool,
    #[arg(long, default_value_t = 30)]
    timeout: u64,
}

#[derive(clap::Args)]
struct RequestArgs {
    /// Collection JSON file.
    collection: String,
    /// Request/item name (fuzzy contains match).
    name: String,
    #[arg(short = 'e', long)]
    env: Option<String>,
    #[arg(long = "var")]
    vars: Vec<String>,
    #[arg(long)]
    save_history: bool,
    #[arg(long, default_value_t = 30)]
    timeout: u64,
}

#[derive(Subcommand)]
enum CollectionCmd {
    List,
    Show { path: String },
    New { name: String },
    AddFolder {
        collection: String,
        name: String,
    },
    AddRequest {
        collection: String,
        name: String,
        #[arg(short = 'm', long, default_value = "GET")]
        method: String,
        #[arg(short = 'u', long)]
        url: String,
        #[arg(long)]
        folder: Option<String>,
    },
    Rm {
        collection: String,
        name: String,
    },
    Rename {
        collection: String,
        old: String,
        new: String,
    },
    Move {
        collection: String,
        name: String,
        /// 目标文件夹（留空=移动到集合根）
        #[arg(long)]
        folder: Option<String>,
    },
    Duplicate {
        collection: String,
        name: String,
        #[arg(short = 'n', long)]
        new_name: Option<String>,
    },
    Export {
        collection: String,
        #[arg(short = 'o', long)]
        output: Option<String>,
    },
}

#[derive(Subcommand)]
enum EnvCmd {
    List,
    Show { path: String },
    New { name: String },
    Set {
        path: String,
        key: String,
        value: String,
    },
    Export {
        path: String,
        #[arg(short = 'o', long)]
        output: Option<String>,
    },
    Delete {
        path: String,
        key: String,
    },
}

#[derive(Subcommand)]
enum GlobalsCmd {
    List,
    Show,
    Set { key: String, value: String },
    Clear,
}

#[derive(Subcommand)]
enum HistoryCmd {
    List {
        #[arg(long, default_value_t = 50)]
        limit: usize,
        /// 仅显示 URL 或方法包含该关键字的记录
        #[arg(long)]
        search: Option<String>,
    },
    Clear,
}

#[derive(clap::Args)]
struct ImportArgs {
    file: String,
    #[arg(long, default_value = "auto")]
    format: String,
    /// Do not save to local workspace (just validate/print summary).
    #[arg(long)]
    no_save: bool,
}

#[derive(clap::Args)]
struct ExportArgs {
    file: String,
    #[arg(short = 'o', long)]
    output: Option<String>,
}

#[derive(clap::Args)]
struct CodegenArgs {
    #[arg(short = 'l', long)]
    lang: String,
    #[arg(short = 'c', long)]
    collection: Option<String>,
    #[arg(short = 'i', long)]
    item: Option<String>,
    #[arg(short = 'u', long)]
    url: Option<String>,
    #[arg(short = 'm', long, default_value = "GET")]
    method: String,
}

#[derive(clap::Args)]
struct RunArgs {
    collection: String,
    #[arg(short = 'e', long)]
    env: Option<String>,
    #[arg(short = 'i', long, default_value_t = 1)]
    iterations: usize,
    /// Data file (CSV or JSON array).
    #[arg(short = 'd', long)]
    data: Option<String>,
    #[arg(long, default_value_t = 0)]
    delay: u64,
    #[arg(long, default_value_t = 30)]
    timeout: u64,
    /// Run only requests under this folder name.
    #[arg(long)]
    folder: Option<String>,
    /// Stop on first failing request.
    #[arg(long)]
    bail: bool,
    /// Write a JSON run report to this path.
    #[arg(long)]
    report: Option<String>,
}

#[derive(clap::Args)]
struct WsArgs {
    url: String,
    /// Message to send after connect (repeatable).
    #[arg(long = "send")]
    send: Vec<String>,
    /// Number of messages to receive before exiting.
    #[arg(long, default_value_t = 1)]
    listen: usize,
    /// Maximum wait time for messages (ms).
    #[arg(long, default_value_t = 5000)]
    wait_ms: u64,
    /// Custom request header, "K: V" (repeatable).
    #[arg(short = 'H', long = "header")]
    header: Vec<String>,
    /// Value for the Sec-WebSocket-Protocol header.
    #[arg(long)]
    subprotocol: Option<String>,
    /// Send messages as binary frames instead of text.
    #[arg(long, default_value_t = false)]
    binary: bool,
}

#[derive(clap::Args)]
struct SseArgs {
    url: String,
    #[arg(short = 'H', long = "header")]
    header: Vec<String>,
    /// Stop after N data events (0 = until stream ends).
    #[arg(long, default_value_t = 0)]
    count: usize,
    #[arg(long, default_value_t = 30)]
    timeout: u64,
}

#[derive(clap::Args)]
struct MqttArgs {
    host: String,
    #[arg(short = 'p', long, default_value_t = 1883)]
    port: u16,
    /// Topic to subscribe to.
    topic: String,
    /// Topic to publish to (optional).
    #[arg(long)]
    publish: Option<String>,
    /// Payload for --publish.
    #[arg(long)]
    message: Option<String>,
    /// Number of messages to receive before exiting.
    #[arg(long, default_value_t = 1)]
    listen: usize,
    #[arg(long, default_value_t = 10)]
    timeout: u64,
    /// Username for MQTT CONNECT.
    #[arg(long)]
    username: Option<String>,
    /// Password for MQTT CONNECT.
    #[arg(long)]
    password: Option<String>,
    /// Publish QoS (0 or 1).
    #[arg(long, default_value_t = 0)]
    qos: u8,
    /// Client id for MQTT CONNECT.
    #[arg(long = "client-id")]
    client_id: Option<String>,
}

#[derive(clap::Args)]
struct GraphqlArgs {
    url: String,
    #[arg(short = 'q', long)]
    query: String,
    #[arg(long)]
    variables: Option<String>,
    #[arg(short = 'e', long)]
    env: Option<String>,
    #[arg(long, default_value_t = 30)]
    timeout: u64,
}

#[derive(clap::Args)]
struct MockArgs {
    collection: String,
    #[arg(long, default_value_t = 8080)]
    port: u16,
    #[arg(long, default_value_t = 0)]
    delay: u64,
}

#[derive(clap::Args)]
struct PerfArgs {
    #[arg(short = 'c', long)]
    collection: Option<String>,
    #[arg(short = 'i', long)]
    item: Option<String>,
    #[arg(short = 'u', long)]
    url: Option<String>,
    #[arg(short = 'm', long, default_value = "GET")]
    method: String,
    #[arg(long, default_value_t = 100)]
    iterations: usize,
    #[arg(long, default_value_t = 10)]
    concurrency: usize,
    #[arg(short = 'e', long)]
    env: Option<String>,
    #[arg(long, default_value_t = 30)]
    timeout: u64,
}

fn load_collection(path: &str) -> Result<Collection> {
    let text = std::fs::read_to_string(path)?;
    let c: Collection = serde_json::from_str(&text)?;
    Ok(c)
}

fn load_env(path: &str) -> Result<Environment> {
    let text = std::fs::read_to_string(path)?;
    let e: Environment = serde_json::from_str(&text)?;
    Ok(e)
}

fn parse_headers(list: &[String]) -> Vec<Header> {
    list.iter()
        .filter_map(|h| {
            let (k, v) = if let Some((k, v)) = h.split_once(':') {
                (k, v)
            } else if let Some((k, v)) = h.split_once('=') {
                (k, v)
            } else {
                return None;
            };
            Some(Header {
                key: k.trim().to_string(),
                value: v.trim().to_string(),
                ..Default::default()
            })
        })
        .collect()
}

fn parse_vars(list: &[String]) -> HashMap<String, String> {
    let mut m = HashMap::new();
    for kv in list {
        if let Some((k, v)) = kv.split_once('=') {
            m.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    m
}

fn build_ctx(collection: Option<&Collection>, env: Option<&Environment>, vars: &[String]) -> VarContext {
    let mut ctx = VarContext::new();
    let coll_scope = collection
        .map(|c| {
            c.variable
                .iter()
                .filter(|v| v.is_enabled())
                .map(|v| (v.key.clone(), value_str(&v.value)))
                .collect::<HashMap<_, _>>()
        })
        .unwrap_or_default();
    let env_scope = env
        .map(|e| {
            e.values
                .iter()
                .filter(|v| v.is_enabled())
                .map(|v| (v.key.clone(), value_str(&v.value)))
                .collect::<HashMap<_, _>>()
        })
        .unwrap_or_default();
    // priority: cli vars > env > collection
    ctx.scopes.push(parse_vars(vars));
    if !env_scope.is_empty() {
        ctx.scopes.push(env_scope);
    }
    if !coll_scope.is_empty() {
        ctx.scopes.push(coll_scope);
    }
    // globals are the lowest-priority scope
    if let Ok(g) = storage::load_globals() {
        let gs: HashMap<String, String> = g
            .values
            .iter()
            .filter(|v| v.is_enabled())
            .map(|v| (v.key.clone(), value_str(&v.value)))
            .collect();
        if !gs.is_empty() {
            ctx.scopes.push(gs);
        }
    }
    ctx
}

/// Save an imported collection; if one with the same name already exists, merge into it
/// instead of overwriting (prevents silent data loss).
fn save_or_merge_collection(c: Collection) -> Result<std::path::PathBuf> {
    let name = c.info.name.clone();
    for p in storage::list_collections().unwrap_or_default() {
        if let Ok(text) = std::fs::read_to_string(&p) {
            if let Ok(mut exist) = serde_json::from_str::<Collection>(&text) {
                if exist.info.name == name {
                    let before = exist.item.len();
                    import_export::merge_collections(&mut exist, c);
                    let saved = storage::save_collection(&exist)?;
                    println!(
                        "merged into existing collection '{}' (+{} items -> {})",
                        name,
                        exist.item.len() - before,
                        saved.display()
                    );
                    return Ok(saved);
                }
            }
        }
    }
    storage::save_collection(&c)
}

fn value_str(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn build_body(args: &SendArgs) -> Option<Body> {
    let data = args.data.clone()?;
    match args.body_type.as_str() {
        "json" => Some(Body {
            mode: "raw".into(),
            raw: Some(data),
            options: Some(serde_json::json!({"raw":{"language":"json"}})),
            ..Default::default()
        }),
        "raw" => Some(Body {
            mode: "raw".into(),
            raw: Some(data),
            options: Some(serde_json::json!({"raw":{"language":"text"}})),
            ..Default::default()
        }),
        "urlencoded" => {
            let pairs = data
                .split('&')
                .filter_map(|kv| kv.split_once('='))
                .map(|(k, v)| KV {
                    key: k.to_string(),
                    value: v.to_string(),
                    ..Default::default()
                })
                .collect();
            Some(Body {
                mode: "urlencoded".into(),
                urlencoded: pairs,
                ..Default::default()
            })
        }
        "form" => {
            let pairs = data
                .split('&')
                .filter_map(|kv| kv.split_once('='))
                .map(|(k, v)| KV {
                    key: k.to_string(),
                    value: v.to_string(),
                    ..Default::default()
                })
                .collect();
            Some(Body {
                mode: "formdata".into(),
                formdata: pairs,
                ..Default::default()
            })
        }
        "file" => Some(Body {
            mode: "file".into(),
            file: Some(FileRef {
                src: Some(data),
                content: None,
            }),
            ..Default::default()
        }),
        "graphql" => Some(Body {
            mode: "graphql".into(),
            graphql: Some(GraphQL {
                query: data,
                variables: String::new(),
            }),
            ..Default::default()
        }),
        other => {
            eprintln!("warning: unknown body-type '{}', using raw", other);
            Some(Body {
                mode: "raw".into(),
                raw: Some(data),
                ..Default::default()
            })
        }
    }
}

fn print_response(rec: &ResponseRecord) {
    println!("{} {}  ({}ms)", rec.code, rec.status, rec.elapsed_ms);
    println!("{}", rec.final_url);
    println!("----- headers -----");
    for (k, v) in &rec.headers {
        println!("{}: {}", k, v);
    }
    println!("----- body -----");
    println!("{}", rec.body);
}

fn find_request(items: &[Item], name: &str) -> Option<(String, Request, Vec<Event>)> {
    for item in items {
        if let Some(req) = &item.request {
            if item.name.to_lowercase().contains(&name.to_lowercase()) {
                return Some((item.name.clone(), req.clone(), item.event.clone()));
            }
        }
        if let Some(children) = &item.item {
            if let Some(found) = find_request(children, name) {
                return Some(found);
            }
        }
    }
    None
}

fn find_mut<'a>(items: &'a mut [Item], name: &str) -> Option<&'a mut Item> {
    for it in items.iter_mut() {
        if it.name == name {
            return Some(it);
        }
        if let Some(children) = it.item.as_mut() {
            if let Some(f) = find_mut(children, name) {
                return Some(f);
            }
        }
    }
    None
}
fn remove_rec(items: &mut Vec<Item>, name: &str) -> bool {
    if let Some(pos) = items.iter().position(|x| x.name == name) {
        items.remove(pos);
        return true;
    }
    for it in items.iter_mut() {
        if let Some(children) = it.item.as_mut() {
            if remove_rec(children, name) {
                return true;
            }
        }
    }
    false
}
fn rename_rec(items: &mut Vec<Item>, old: &str, new: &str) -> bool {
    for it in items.iter_mut() {
        if it.name == old {
            it.name = new.to_string();
            return true;
        }
        if let Some(children) = it.item.as_mut() {
            if rename_rec(children, old, new) {
                return true;
            }
        }
    }
    false
}

fn take_rec(items: &mut Vec<Item>, name: &str) -> Option<Item> {
    if let Some(pos) = items.iter().position(|x| x.name == name) {
        return Some(items.remove(pos));
    }
    for it in items.iter_mut() {
        if let Some(children) = it.item.as_mut() {
            if let Some(found) = take_rec(children, name) {
                return Some(found);
            }
        }
    }
    None
}

fn find_clone(items: &[Item], name: &str) -> Option<Item> {
    for it in items {
        if it.name == name {
            return Some(it.clone());
        }
        if let Some(children) = &it.item {
            if let Some(found) = find_clone(children, name) {
                return Some(found);
            }
        }
    }
    None
}

fn build_perf_request(args: &PerfArgs) -> Result<(Request, VarContext)> {
    let env = args.env.as_deref().map(load_env).transpose()?;
    if let (Some(cp), Some(item)) = (args.collection.as_ref(), args.item.as_ref()) {
        let c = load_collection(cp)?;
        let (_, mut req, _) =
            find_request(&c.item, item).ok_or_else(|| anyhow!("request not found: {}", item))?;
        if req.auth.is_none() {
            req.auth = c.auth.clone();
        }
        let ctx = build_ctx(Some(&c), env.as_ref(), &[]);
        Ok((req, ctx))
    } else {
        let url = args
            .url
            .clone()
            .ok_or_else(|| anyhow!("provide --collection + --item, or --url"))?;
        let req = Request {
            method: args.method.clone(),
            url: UrlSpec::new(url),
            ..Default::default()
        };
        let ctx = build_ctx(None, env.as_ref(), &[]);
        Ok((req, ctx))
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Send(args) => {
            let collection = args.collection.as_deref().map(load_collection).transpose()?;
            let env = args.env.as_deref().map(load_env).transpose()?;
            let ctx = build_ctx(collection.as_ref(), env.as_ref(), &args.vars);
            let req = Request {
                method: args.method.clone(),
                url: UrlSpec::new(&args.url),
                header: parse_headers(&args.header),
                body: build_body(&args),
                ..Default::default()
            };
            let rec = http::execute(&req, &ctx, args.timeout)?;
            print_response(&rec);
            if args.save_history {
                storage::append_history(&HistoryEntry {
                    ts: chrono::Utc::now().to_rfc3339(),
                    method: req.method.to_uppercase(),
                    url: ctx.resolve(&req.url.effective_url()),
                    status: rec.code,
                    elapsed_ms: rec.elapsed_ms,
                })?;
            }
        }
        Cmd::Request(args) => {
            let collection = load_collection(&args.collection)?;
            let env = args.env.as_deref().map(load_env).transpose()?;
            let (_, mut req, _events) = find_request(&collection.item, &args.name)
                .ok_or_else(|| anyhow!("request not found: {}", args.name))?;
            if let Some(a) = &collection.auth {
                if req.auth.is_none() {
                    req.auth = Some(a.clone());
                }
            }
            let ctx = build_ctx(Some(&collection), env.as_ref(), &args.vars);
            let rec = http::execute(&req, &ctx, args.timeout)?;
            print_response(&rec);
            if args.save_history {
                storage::append_history(&HistoryEntry {
                    ts: chrono::Utc::now().to_rfc3339(),
                    method: req.method.to_uppercase(),
                    url: ctx.resolve(&req.url.effective_url()),
                    status: rec.code,
                    elapsed_ms: rec.elapsed_ms,
                })?;
            }
        }
        Cmd::Collection { cmd } => match cmd {
            CollectionCmd::List => {
                for p in storage::list_collections()? {
                    println!("{}", p.display());
                }
            }
            CollectionCmd::Show { path } => {
                let c = load_collection(&path)?;
                println!("{}", serde_json::to_string_pretty(&c)?);
            }
            CollectionCmd::New { name } => {
                let c = Collection {
                    info: CollectionInfo {
                        name: name.clone(),
                        postman_id: Some(uuid::Uuid::new_v4().to_string()),
                        schema: Some(
                            "https://schema.getpostman.com/json/collection/v2.1.0/collection.json".into(),
                        ),
                        description: None,
                    },
                    ..Default::default()
                };
                let p = storage::save_collection(&c)?;
                println!("created collection: {}", p.display());
            }
            CollectionCmd::AddFolder { collection, name } => {
                let mut c = load_collection(&collection)?;
                c.item.push(Item {
                    name: name.clone(),
                    item: Some(Vec::new()),
                    ..Default::default()
                });
                let p = storage::save_collection(&c)?;
                println!("added folder '{}' -> {}", name, p.display());
            }
            CollectionCmd::AddRequest {
                collection,
                name,
                method,
                url,
                folder,
            } => {
                let mut c = load_collection(&collection)?;
                let item = Item {
                    name: name.clone(),
                    request: Some(Request {
                        method,
                        url: UrlSpec::new(url),
                        ..Default::default()
                    }),
                    ..Default::default()
                };
                match folder {
                    Some(fd) => {
                        if let Some(f) = find_mut(&mut c.item, &fd) {
                            if f.item.is_none() {
                                f.item = Some(Vec::new());
                            }
                            f.item.as_mut().unwrap().push(item);
                        } else {
                            return Err(anyhow!("folder not found: {}", fd));
                        }
                    }
                    None => c.item.push(item),
                }
                let p = storage::save_collection(&c)?;
                println!("added request '{}' -> {}", name, p.display());
            }
            CollectionCmd::Rm { collection, name } => {
                let mut c = load_collection(&collection)?;
                if remove_rec(&mut c.item, &name) {
                    let p = storage::save_collection(&c)?;
                    println!("removed '{}' -> {}", name, p.display());
                } else {
                    return Err(anyhow!("item not found: {}", name));
                }
            }
            CollectionCmd::Rename {
                collection,
                old,
                new,
            } => {
                let mut c = load_collection(&collection)?;
                if rename_rec(&mut c.item, &old, &new) {
                    let p = storage::save_collection(&c)?;
                    println!("renamed '{}' -> '{}' ({})", old, new, p.display());
                } else {
                    return Err(anyhow!("item not found: {}", old));
                }
            }
            CollectionCmd::Move {
                collection,
                name,
                folder,
            } => {
                let mut c = load_collection(&collection)?;
                let Some(item) = take_rec(&mut c.item, &name) else {
                    return Err(anyhow!("item not found: {}", name));
                };
                match folder {
                    Some(fd) => {
                        if let Some(f) = find_mut(&mut c.item, &fd) {
                            if f.item.is_none() {
                                f.item = Some(Vec::new());
                            }
                            f.item.as_mut().unwrap().push(item);
                        } else {
                            return Err(anyhow!("folder not found: {}", fd));
                        }
                    }
                    None => c.item.push(item),
                }
                let p = storage::save_collection(&c)?;
                println!("moved '{}' -> {}", name, p.display());
            }
            CollectionCmd::Duplicate {
                collection,
                name,
                new_name,
            } => {
                let mut c = load_collection(&collection)?;
                let Some(mut it) = find_clone(&c.item, &name) else {
                    return Err(anyhow!("item not found: {}", name));
                };
                it.name = new_name.unwrap_or_else(|| format!("{} 副本", it.name));
                let nn = it.name.clone();
                c.item.push(it);
                let p = storage::save_collection(&c)?;
                println!("duplicated '{}' as '{}' -> {}", name, nn, p.display());
            }
            CollectionCmd::Export { collection, output } => {
                let mut c = load_collection(&collection)?;
                let text = import_export::export_collection(&mut c)?;
                match output {
                    Some(o) => {
                        std::fs::write(&o, text)?;
                        println!("exported collection: {}", o);
                    }
                    None => println!("{}", text),
                }
            }
        },
        Cmd::Env { cmd } => match cmd {
            EnvCmd::List => {
                for p in storage::list_environments()? {
                    println!("{}", p.display());
                }
            }
            EnvCmd::Show { path } => {
                let e = load_env(&path)?;
                println!("{}", serde_json::to_string_pretty(&e)?);
            }
            EnvCmd::New { name } => {
                let e = Environment {
                    id: Some(uuid::Uuid::new_v4().to_string()),
                    name,
                    values: Vec::new(),
                    scope: Some("environment".into()),
                };
                let p = storage::save_environment(&e)?;
                println!("created environment: {}", p.display());
            }
            EnvCmd::Set { path, key, value } => {
                let mut e = load_env(&path)?;
                if let Some(v) = e.values.iter_mut().find(|v| v.key == key) {
                    v.value = serde_json::Value::String(value);
                } else {
                    e.values.push(Variable {
                        key,
                        value: serde_json::Value::String(value),
                        typ: Some("default".into()),
                        enabled: Some(true),
                        description: None,
                    });
                }
                std::fs::write(&path, serde_json::to_string_pretty(&e)?)?;
                println!("updated {}", path);
            }
            EnvCmd::Export { path, output } => {
                let mut e = load_env(&path)?;
                let text = import_export::export_environment(&mut e)?;
                match output {
                    Some(o) => {
                        std::fs::write(&o, text)?;
                        println!("exported environment: {}", o);
                    }
                    None => println!("{}", text),
                }
            }
            EnvCmd::Delete { path, key } => {
                let mut e = load_env(&path)?;
                let before = e.values.len();
                e.values.retain(|v| v.key != key);
                if e.values.len() == before {
                    return Err(anyhow!("key not found: {}", key));
                }
                std::fs::write(&path, serde_json::to_string_pretty(&e)?)?;
                println!("deleted key '{}' from {}", key, path);
            }
        },
        Cmd::History { cmd } => match cmd {
            HistoryCmd::List { limit, search } => {
                let h = storage::read_history()?;
                let q = search.unwrap_or_default().to_lowercase();
                for e in h
                    .iter()
                    .rev()
                    .filter(|e| {
                        q.is_empty()
                            || e.url.to_lowercase().contains(&q)
                            || e.method.to_lowercase().contains(&q)
                    })
                    .take(limit)
                {
                    println!("{}  {} {} -> {}", e.ts, e.method, e.url, e.status);
                }
            }
            HistoryCmd::Clear => {
                storage::clear_history()?;
                println!("history cleared");
            }
        },
        Cmd::Import(args) => {
            // For cURL, accept either a file path or an inline command string.
            let imported = if args.format.eq_ignore_ascii_case("curl")
                && !std::path::Path::new(&args.file).exists()
            {
                Imported::Collection(Box::new(import_export::import_curl(&args.file)))
            } else {
                import_export::import_file(&args.file, &args.format)?
            };
            match imported {
                Imported::Collection(mut c) => {
                    println!(
                        "imported collection '{}' with {} top-level items",
                        c.info.name,
                        c.item.len()
                    );
                    if !args.no_save {
                        let p = save_or_merge_collection(*c)?;
                        println!("saved: {}", p.display());
                    } else {
                        let _ = import_export::export_collection(&mut c);
                    }
                }
                Imported::Environment(e) => {
                    println!("imported environment '{}' with {} values", e.name, e.values.len());
                    if !args.no_save {
                        let p = storage::save_environment(&e)?;
                        println!("saved: {}", p.display());
                    }
                }
            }
        }
        Cmd::Export(args) => {
            let mut c = load_collection(&args.file)?;
            let out = import_export::export_collection(&mut c)?;
            match args.output {
                Some(o) => {
                    std::fs::write(&o, out)?;
                    println!("exported: {}", o);
                }
                None => println!("{}", out),
            }
        }
        Cmd::Codegen(args) => {
            let req = if let (Some(cp), Some(item)) = (args.collection.as_ref(), args.item.as_ref()) {
                let c = load_collection(cp)?;
                find_request(&c.item, item)
                    .map(|(_, r, _)| r)
                    .ok_or_else(|| anyhow!("request not found: {}", item))?
            } else {
                let url = args.url.clone().ok_or_else(|| {
                    anyhow!("provide --collection + --item, or --url for direct codegen")
                })?;
                Request {
                    method: args.method.clone(),
                    url: UrlSpec::new(url),
                    ..Default::default()
                }
            };
            println!("{}", codegen::generate(&req, &args.lang)?);
        }
        Cmd::Run(args) => {
            let collection = load_collection(&args.collection)?;
            let env = args.env.as_deref().map(load_env).transpose()?;
            let data_rows = match args.data.as_deref() {
                Some(d) => runner::load_data_file(d)?,
                None => Vec::new(),
            };
            let summary = runner::run_collection_opts(
                &collection,
                env.as_ref(),
                args.iterations,
                data_rows,
                args.delay,
                args.timeout,
                RunOptions {
                    folder: args.folder.clone(),
                    bail: args.bail,
                    report: args.report.clone(),
                },
            );
            for r in &summary.results {
                let status = match r.code {
                    Some(c) => c.to_string(),
                    None => "ERR".to_string(),
                };
                println!(
                    "[{}] {} {} {} ({}ms) tests: {}/{}",
                    status,
                    r.method,
                    r.url,
                    if r.passed { "PASS" } else { "FAIL" },
                    r.elapsed_ms,
                    r.tests.iter().filter(|t| t.passed).count(),
                    r.tests.len()
                );
                if let Some(err) = &r.error {
                    println!("    error: {}", err);
                }
                for t in &r.tests {
                    if !t.passed {
                        println!("    FAILED: {} ({})", t.name, t.detail);
                    }
                }
            }
            println!(
                "\nSummary: total={} passed={} failed={}",
                summary.total, summary.passed, summary.failed
            );
            if summary.failed > 0 {
                std::process::exit(1);
            }
        }
        Cmd::Globals { cmd } => match cmd {
            GlobalsCmd::List | GlobalsCmd::Show => {
                let g = storage::load_globals()?;
                for v in &g.values {
                    println!("{}={}", v.key, value_str(&v.value));
                }
            }
            GlobalsCmd::Set { key, value } => {
                let mut g = storage::load_globals()?;
                if let Some(v) = g.values.iter_mut().find(|v| v.key == key) {
                    v.value = serde_json::Value::String(value);
                } else {
                    g.values.push(Variable {
                        key,
                        value: serde_json::Value::String(value),
                        typ: Some("default".into()),
                        enabled: Some(true),
                        description: None,
                    });
                }
                let p = storage::save_globals(&g)?;
                println!("updated globals: {}", p.display());
            }
            GlobalsCmd::Clear => {
                let mut g = storage::load_globals()?;
                g.values.clear();
                let p = storage::save_globals(&g)?;
                println!("cleared globals: {}", p.display());
            }
        },
        Cmd::Ws(args) => {
            let headers: Vec<(String, String)> = parse_headers(&args.header)
                .into_iter()
                .map(|h| (h.key, h.value))
                .collect();
            let cfg = ws::WsConfig {
                url: args.url,
                messages: args.send,
                listen: args.listen,
                wait_ms: args.wait_ms,
                headers,
                subprotocol: args.subprotocol,
                binary: args.binary,
            };
            for line in ws::run_collect(&cfg)? {
                println!("{}", line);
            }
        }
        Cmd::Sse(args) => {
            let headers: Vec<(String, String)> = parse_headers(&args.header)
                .into_iter()
                .map(|h| (h.key, h.value))
                .collect();
            sse::run(&args.url, &headers, args.count, args.timeout)?;
        }
        Cmd::Mqtt(args) => {
            let publish = match (args.publish, args.message) {
                (Some(t), Some(m)) => Some((t, m)),
                (Some(t), None) => Some((t, String::new())),
                _ => None,
            };
            let pub_ref = publish
                .as_ref()
                .map(|(t, m)| (t.as_str(), m.as_str(), args.qos));
            let cfg = mqtt::MqttConfig {
                host: &args.host,
                port: args.port,
                client_id: args.client_id.as_deref(),
                username: args.username.as_deref(),
                password: args.password.as_deref(),
                topic: &args.topic,
                publish: pub_ref,
                qos: args.qos,
                listen: args.listen,
                timeout_secs: args.timeout,
            };
            for line in mqtt::run_advanced(&cfg)? {
                println!("{}", line);
            }
        }
        Cmd::Graphql(args) => {
            let env = args.env.as_deref().map(load_env).transpose()?;
            let ctx = build_ctx(None, env.as_ref(), &[]);
            let req = Request {
                method: "POST".into(),
                url: UrlSpec::new(args.url),
                body: Some(Body {
                    mode: "graphql".into(),
                    graphql: Some(GraphQL {
                        query: args.query,
                        variables: args.variables.unwrap_or_default(),
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            };
            let rec = http::execute(&req, &ctx, args.timeout)?;
            print_response(&rec);
        }
        Cmd::Mock(args) => {
            mock::serve(&args.collection, args.port, args.delay)?;
        }
        Cmd::Perf(args) => {
            let (req, ctx) = build_perf_request(&args)?;
            let report = perf::run(&req, &ctx, args.iterations, args.concurrency, args.timeout);
            println!("iterations: {}  concurrency: {}", report.total, args.concurrency);
            println!("ok: {}  err: {}  rps: {:.1}", report.ok, report.err, report.rps);
            println!(
                "latency ms  mean: {:.1}  p50: {}  p90: {}  p99: {}  max: {}",
                report.mean_ms, report.p50, report.p90, report.p99, report.max_ms
            );
        }
    }
    Ok(())
}

