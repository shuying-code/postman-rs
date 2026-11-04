//! Minimal WebSocket client (ws:// and wss://) built on `tungstenite`.
//!
//! Supports custom request headers, a sub-protocol, and binary frames. The
//! event log is produced by [`run_collect`] (used by the GUI) and streamed to
//! stdout by [`run`] (used by the CLI).
use anyhow::{Context, Result}; // 引入依赖
use std::time::{Duration, Instant}; // 引入依赖
use tungstenite::client::IntoClientRequest; // 引入依赖
use tungstenite::http::header::{HeaderName, HeaderValue}; // 引入依赖
use tungstenite::Message; // 引入依赖

pub struct WsConfig { // 结构体 WsConfig
    pub url: String, // 字段 url
    pub messages: Vec<String>, // 字段 messages
    pub listen: usize, // 字段 listen
    pub wait_ms: u64, // 字段 wait_ms
    /// Custom request headers sent with the upgrade request.
    pub headers: Vec<(String, String)>, // 字段 headers
    /// Value for the `Sec-WebSocket-Protocol` header (sub-protocol).
    pub subprotocol: Option<String>, // 字段 subprotocol
    /// When true, `messages` are sent as binary frames instead of text.
    pub binary: bool, // 字段 binary
} // 块结束

/// Connect, exchange messages, and return a log of events as strings.
pub fn run_collect(cfg: &WsConfig) -> Result<Vec<String>> { // 函数 run_collect
    let mut events: Vec<String> = Vec::new(); // 变量 events

    let mut req = cfg // 变量 req
        .url // 语句
        .as_str() // 语句
        .into_client_request() // 语句
        .context("invalid websocket url")?; // 语句
    for (k, v) in &cfg.headers { // 循环
        let name: HeaderName = k // 变量 name
            .parse() // 语句
            .with_context(|| format!("invalid header name: {}", k))?; // 语句
        let value: HeaderValue = v // 变量 value
            .parse() // 语句
            .with_context(|| format!("invalid header value for {}", k))?; // 语句
        req.headers_mut().insert(name, value); // 语句
    } // 块结束
    if let Some(p) = &cfg.subprotocol { // 条件分支
        let value: HeaderValue = p // 变量 value
            .parse() // 语句
            .with_context(|| format!("invalid subprotocol: {}", p))?; // 语句
        req.headers_mut().insert( // 语句
            HeaderName::from_static("sec-websocket-protocol"), // 字段 HeaderName
            value, // 语句
        ); // 块结束
    } // 块结束

    let (mut socket, resp) = tungstenite::connect(req).context("websocket connect failed")?; // 调用 let
    events.push(format!( // 语句
        "connected: {} (status {})", // 语句
        cfg.url, // 语句
        resp.status() // 语句
    )); // 语句

    for m in &cfg.messages { // 循环
        let msg = if cfg.binary { // 变量 msg
            Message::Binary(m.as_bytes().to_vec()) // 调用 Binary
        } else { // 语句
            Message::Text(m.clone()) // 调用 Text
        }; // 块结束
        socket // 语句
            .send(msg) // 语句
            .with_context(|| format!("send failed: {}", m))?; // 语句
        events.push(format!(">> {}", m)); // 语句
    } // 块结束

    if cfg.listen == 0 { // 条件分支
        let _ = socket.close(None); // 变量 _
        return Ok(events); // 返回
    } // 块结束

    let deadline = Instant::now() + Duration::from_millis(cfg.wait_ms); // 变量 deadline
    let mut got = 0usize; // 变量 got
    while got < cfg.listen && Instant::now() < deadline { // 循环
        match socket.read() { // 分支匹配
            Ok(Message::Text(t)) => { // 匹配分支
                events.push(format!("<< {}", t)); // 语句
                got += 1; // 语句
            } // 块结束
            Ok(Message::Binary(b)) => { // 匹配分支
                events.push(format!("<< [binary {} bytes]", b.len())); // 语句
                got += 1; // 语句
            } // 块结束
            Ok(Message::Ping(p)) => { // 匹配分支
                let _ = socket.send(Message::Pong(p)); // 变量 _
            } // 块结束
            Ok(Message::Pong(_)) | Ok(Message::Frame(_)) => {} // 匹配分支
            Ok(Message::Close(_)) => { // 匹配分支
                events.push("<< closed by peer".to_string()); // 语句
                break; // 语句
            } // 块结束
            Err(e) => { // 匹配分支
                events.push(format!("<< read error: {}", e)); // 语句
                break; // 语句
            } // 块结束
        } // 块结束
    } // 块结束
    let _ = socket.close(None); // 变量 _
    Ok(events) // 调用 Ok
} // 块结束

pub fn run(cfg: &WsConfig) -> Result<()> { // 函数 run
    for line in run_collect(cfg)? { // 循环
        println!("{}", line); // 语句
    } // 块结束
    Ok(()) // 调用 Ok
} // 块结束
