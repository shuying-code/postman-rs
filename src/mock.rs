//! Local mock server: replays saved example responses from a collection.
use crate::model::{Collection, Item};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::time::Duration;
type Route = (u16, Vec<(String, String)>, String);
fn url_path(raw: &str) -> String {
    if let Ok(u) = url::Url::parse(raw) {
        return u.path().to_string();
    }
    let rest = raw.splitn(2, "://").nth(1).unwrap_or(raw);
    let after_host = rest.split_once('/').map(|(_, p)| format!("/{}", p)).unwrap_or("/".into());
    after_host.split('?').next().unwrap_or("/").to_string()
}
fn walk(items: &[Item], routes: &mut HashMap<String, Route>) {
    for it in items {
        if let Some(req) = &it.request {
            if let Some(ex) = it.response.first() {
                let code = ex.code.unwrap_or(200);
                let mut headers = Vec::new();
                for h in &ex.header {
                    headers.push((h.key.clone(), h.value.clone()));
                }
                let body = ex.body.clone().unwrap_or_default();
                let key = format!("{} {}", req.method.to_uppercase(), url_path(&req.url.raw));
                routes.insert(key, (code, headers, body));
            }
        }
        if let Some(children) = &it.item {
            walk(children, routes);
        }
    }
}
fn reason(code: u16) -> &'static str {
    match code {
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "OK",
    }
}
fn handle(mut stream: TcpStream, routes: Arc<HashMap<String, Route>>, delay_ms: u64) {
    let mut reader = BufReader::new(stream.try_clone().expect("clone stream"));
    let mut first = String::new();
    if reader.read_line(&mut first).is_err() {
        return;
    }
    let mut parts = first.split_whitespace();
    let method = parts.next().unwrap_or("GET").to_uppercase();
    let target = parts.next().unwrap_or("/").to_string();
    let path = target.split('?').next().unwrap_or("/").to_string();
    if delay_ms > 0 {
        std::thread::sleep(Duration::from_millis(delay_ms));
    }
    let key = format!("{} {}", method, path);
    let (code, headers, body) = match routes.get(&key) {
        Some(r) => r.clone(),
        None => (
            404,
            vec![("Content-Type".to_string(), "application/json".to_string())],
            format!("{{\"error\":\"no mock for {}\"}}", key),
        ),
    };
    let mut resp = format!("HTTP/1.1 {} {}\r\n", code, reason(code));
    let mut has_ct = false;
    for (k, v) in &headers {
        if k.eq_ignore_ascii_case("content-type") {
            has_ct = true;
        }
        resp.push_str(&format!("{}: {}\r\n", k, v));
    }
    if !has_ct {
        resp.push_str("Content-Type: application/json\r\n");
    }
    resp.push_str(&format!("Content-Length: {}\r\n", body.as_bytes().len()));
    resp.push_str("Connection: close\r\n\r\n");
    resp.push_str(&body);
    let _ = stream.write_all(resp.as_bytes());
    let _ = stream.flush();
}
pub fn serve(collection_path: &str, port: u16, delay_ms: u64) -> Result<()> {
    let text = std::fs::read_to_string(collection_path).context("read collection")?;
    let c: Collection = serde_json::from_str(&text).context("parse collection")?;
    let mut routes = HashMap::new();
    walk(&c.item, &mut routes);
    let routes = Arc::new(routes);
    let listener = TcpListener::bind(("0.0.0.0", port)).with_context(|| format!("bind port {}", port))?;
    println!(
        "mock server listening on http://127.0.0.1:{}  ({} routes from '{}')",
        port,
        routes.len(),
        c.info.name
    );
    for r in routes.keys() {
        println!("  {}", r);
    }
    for stream in listener.incoming() {
        if let Ok(s) = stream {
            let routes = routes.clone();
            std::thread::spawn(move || handle(s, routes, delay_ms));
        }
    }
    Ok(())
}
