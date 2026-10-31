//! Minimal WebSocket client (ws:// and wss://) built on `tungstenite`.
//!
//! Supports custom request headers, a sub-protocol, and binary frames. The
//! event log is produced by [`run_collect`] (used by the GUI) and streamed to
//! stdout by [`run`] (used by the CLI).
use anyhow::{Context, Result};
use std::time::{Duration, Instant};
use tungstenite::client::IntoClientRequest;
use tungstenite::http::header::{HeaderName, HeaderValue};
use tungstenite::Message;

pub struct WsConfig {
    pub url: String,
    pub messages: Vec<String>,
    pub listen: usize,
    pub wait_ms: u64,
    /// Custom request headers sent with the upgrade request.
    pub headers: Vec<(String, String)>,
    /// Value for the `Sec-WebSocket-Protocol` header (sub-protocol).
    pub subprotocol: Option<String>,
    /// When true, `messages` are sent as binary frames instead of text.
    pub binary: bool,
}

/// Connect, exchange messages, and return a log of events as strings.
pub fn run_collect(cfg: &WsConfig) -> Result<Vec<String>> {
    let mut events: Vec<String> = Vec::new();

    let mut req = cfg
        .url
        .as_str()
        .into_client_request()
        .context("invalid websocket url")?;
    for (k, v) in &cfg.headers {
        let name: HeaderName = k
            .parse()
            .with_context(|| format!("invalid header name: {}", k))?;
        let value: HeaderValue = v
            .parse()
            .with_context(|| format!("invalid header value for {}", k))?;
        req.headers_mut().insert(name, value);
    }
    if let Some(p) = &cfg.subprotocol {
        let value: HeaderValue = p
            .parse()
            .with_context(|| format!("invalid subprotocol: {}", p))?;
        req.headers_mut().insert(
            HeaderName::from_static("sec-websocket-protocol"),
            value,
        );
    }

    let (mut socket, resp) = tungstenite::connect(req).context("websocket connect failed")?;
    events.push(format!(
        "connected: {} (status {})",
        cfg.url,
        resp.status()
    ));

    for m in &cfg.messages {
        let msg = if cfg.binary {
            Message::Binary(m.as_bytes().to_vec())
        } else {
            Message::Text(m.clone())
        };
        socket
            .send(msg)
            .with_context(|| format!("send failed: {}", m))?;
        events.push(format!(">> {}", m));
    }

    if cfg.listen == 0 {
        let _ = socket.close(None);
        return Ok(events);
    }

    let deadline = Instant::now() + Duration::from_millis(cfg.wait_ms);
    let mut got = 0usize;
    while got < cfg.listen && Instant::now() < deadline {
        match socket.read() {
            Ok(Message::Text(t)) => {
                events.push(format!("<< {}", t));
                got += 1;
            }
            Ok(Message::Binary(b)) => {
                events.push(format!("<< [binary {} bytes]", b.len()));
                got += 1;
            }
            Ok(Message::Ping(p)) => {
                let _ = socket.send(Message::Pong(p));
            }
            Ok(Message::Pong(_)) | Ok(Message::Frame(_)) => {}
            Ok(Message::Close(_)) => {
                events.push("<< closed by peer".to_string());
                break;
            }
            Err(e) => {
                events.push(format!("<< read error: {}", e));
                break;
            }
        }
    }
    let _ = socket.close(None);
    Ok(events)
}

pub fn run(cfg: &WsConfig) -> Result<()> {
    for line in run_collect(cfg)? {
        println!("{}", line);
    }
    Ok(())
}
