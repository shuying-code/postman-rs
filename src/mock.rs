//! Local mock server: replays saved example responses from a collection.
use crate::model::{Collection, Item}; // 引入依赖
use anyhow::{Context, Result}; // 引入依赖
use std::collections::HashMap; // 引入依赖
use std::io::{BufRead, BufReader, Write}; // 引入依赖
use std::net::{TcpListener, TcpStream}; // 引入依赖
use std::sync::Arc; // 引入依赖
use std::time::Duration; // 引入依赖
type Route = (u16, Vec<(String, String)>, String); // 语句
fn url_path(raw: &str) -> String { // 函数 url_path
    if let Ok(u) = url::Url::parse(raw) { // 条件分支
        return u.path().to_string(); // 返回
    } // 块结束
    let rest = raw.splitn(2, "://").nth(1).unwrap_or(raw); // 变量 rest
    let after_host = rest.split_once('/').map(|(_, p)| format!("/{}", p)).unwrap_or("/".into()); // 变量 after_host
    after_host.split('?').next().unwrap_or("/").to_string() // 语句
} // 块结束
fn walk(items: &[Item], routes: &mut HashMap<String, Route>) { // 函数 walk
    for it in items { // 循环
        if let Some(req) = &it.request { // 条件分支
            if let Some(ex) = it.response.first() { // 条件分支
                let code = ex.code.unwrap_or(200); // 变量 code
                let mut headers = Vec::new(); // 变量 headers
                for h in &ex.header { // 循环
                    headers.push((h.key.clone(), h.value.clone())); // 语句
                } // 块结束
                let body = ex.body.clone().unwrap_or_default(); // 变量 body
                let key = format!("{} {}", req.method.to_uppercase(), url_path(&req.url.raw)); // 变量 key
                routes.insert(key, (code, headers, body)); // 语句
            } // 块结束
        } // 块结束
        if let Some(children) = &it.item { // 条件分支
            walk(children, routes); // 调用 walk
        } // 块结束
    } // 块结束
} // 块结束
fn reason(code: u16) -> &'static str { // 函数 reason
    match code { // 分支匹配
        200 => "OK", // 匹配分支
        201 => "Created", // 匹配分支
        202 => "Accepted", // 匹配分支
        204 => "No Content", // 匹配分支
        301 => "Moved Permanently", // 匹配分支
        302 => "Found", // 匹配分支
        400 => "Bad Request", // 匹配分支
        401 => "Unauthorized", // 匹配分支
        403 => "Forbidden", // 匹配分支
        404 => "Not Found", // 匹配分支
        500 => "Internal Server Error", // 匹配分支
        _ => "OK", // 匹配分支
    } // 块结束
} // 块结束
fn handle(mut stream: TcpStream, routes: Arc<HashMap<String, Route>>, delay_ms: u64) { // 函数 handle
    let mut reader = BufReader::new(stream.try_clone().expect("clone stream")); // 变量 reader
    let mut first = String::new(); // 变量 first
    if reader.read_line(&mut first).is_err() { // 条件分支
        return; // 返回
    } // 块结束
    let mut parts = first.split_whitespace(); // 变量 parts
    let method = parts.next().unwrap_or("GET").to_uppercase(); // 变量 method
    let target = parts.next().unwrap_or("/").to_string(); // 变量 target
    let path = target.split('?').next().unwrap_or("/").to_string(); // 变量 path
    if delay_ms > 0 { // 条件分支
        std::thread::sleep(Duration::from_millis(delay_ms)); // 调用 sleep
    } // 块结束
    let key = format!("{} {}", method, path); // 变量 key
    let (code, headers, body) = match routes.get(&key) { // 调用 let
        Some(r) => r.clone(), // 匹配分支
        None => ( // 匹配分支
            404, // 语句
            vec![("Content-Type".to_string(), "application/json".to_string())], // 语句
            format!("{{\"error\":\"no mock for {}\"}}", key), // 语句
        ), // 语句
    }; // 块结束
    let mut resp = format!("HTTP/1.1 {} {}\r\n", code, reason(code)); // 变量 resp
    let mut has_ct = false; // 变量 has_ct
    for (k, v) in &headers { // 循环
        if k.eq_ignore_ascii_case("content-type") { // 条件分支
            has_ct = true; // 语句
        } // 块结束
        resp.push_str(&format!("{}: {}\r\n", k, v)); // 语句
    } // 块结束
    if !has_ct { // 条件分支
        resp.push_str("Content-Type: application/json\r\n"); // 语句
    } // 块结束
    resp.push_str(&format!("Content-Length: {}\r\n", body.as_bytes().len())); // 语句
    resp.push_str("Connection: close\r\n\r\n"); // 语句
    resp.push_str(&body); // 语句
    let _ = stream.write_all(resp.as_bytes()); // 变量 _
    let _ = stream.flush(); // 变量 _
} // 块结束
pub fn serve(collection_path: &str, port: u16, delay_ms: u64) -> Result<()> { // 函数 serve
    let text = std::fs::read_to_string(collection_path).context("read collection")?; // 变量 text
    let c: Collection = serde_json::from_str(&text).context("parse collection")?; // 变量 c
    let mut routes = HashMap::new(); // 变量 routes
    walk(&c.item, &mut routes); // 调用 walk
    let routes = Arc::new(routes); // 变量 routes
    let listener = TcpListener::bind(("0.0.0.0", port)).with_context(|| format!("bind port {}", port))?; // 变量 listener
    println!( // 语句
        "mock server listening on http://127.0.0.1:{}  ({} routes from '{}')", // 语句
        port, // 语句
        routes.len(), // 语句
        c.info.name // 语句
    ); // 块结束
    for r in routes.keys() { // 循环
        println!("  {}", r); // 语句
    } // 块结束
    for stream in listener.incoming() { // 循环
        if let Ok(s) = stream { // 条件分支
            let routes = routes.clone(); // 变量 routes
            std::thread::spawn(move || handle(s, routes, delay_ms)); // 调用 spawn
        } // 块结束
    } // 块结束
    Ok(()) // 调用 Ok
} // 块结束
