//! Minimal MQTT 3.1.1 client over **plain TCP**: connect, subscribe, publish,
//! receive.
//!
//! NOTE ON TLS: this module deliberately does not implement MQTT-over-TLS
//! (`mqtts://` / port 8883). Doing so cleanly would require pulling in a TLS
//! stack for the raw TCP stream; to keep the dependency footprint small we stay
//! on plain-text TCP. Username/password authentication, configurable client id,
//! subscription QoS and QoS-0/1 publishing *are* supported.
use anyhow::{anyhow, Context, Result}; // 引入依赖
use std::io::{Read, Write}; // 引入依赖
use std::net::TcpStream; // 引入依赖
use std::time::Duration; // 引入依赖

fn enc_str(s: &str) -> Vec<u8> { // 函数 enc_str
    let b = s.as_bytes(); // 变量 b
    let mut v = Vec::with_capacity(2 + b.len()); // 变量 v
    v.push((b.len() >> 8) as u8); // 语句
    v.push((b.len() & 0xff) as u8); // 语句
    v.extend_from_slice(b); // 语句
    v // 语句
} // 块结束

fn enc_len(mut len: usize) -> Vec<u8> { // 函数 enc_len
    let mut out = Vec::new(); // 变量 out
    loop { // 循环
        let mut byte = (len % 128) as u8; // 变量 byte
        len /= 128; // 语句
        if len > 0 { // 条件分支
            byte |= 0x80; // 语句
        } // 块结束
        out.push(byte); // 语句
        if len == 0 { // 条件分支
            break; // 语句
        } // 块结束
    } // 块结束
    out // 语句
} // 块结束

fn packet(first: u8, body: &[u8]) -> Vec<u8> { // 函数 packet
    let mut v = vec![first]; // 变量 v
    v.extend(enc_len(body.len())); // 语句
    v.extend_from_slice(body); // 语句
    v // 语句
} // 块结束

/// Build a CONNECT packet body.
///
/// Connect flags (bit7 = username, bit6 = password, bit1 = clean session) and
/// the payload order (client id, then username, then password) follow the
/// MQTT 3.1.1 spec.
fn connect_body( // 函数 connect_body
    client_id: &str, // 字段 client_id
    username: Option<&str>, // 字段 username
    password: Option<&str>, // 字段 password
) -> Vec<u8> { // 语句
    let mut flags = 0x02u8; // clean session
    if username.is_some() { // 条件分支
        flags |= 0x80; // 语句
    } // 块结束
    if password.is_some() { // 条件分支
        flags |= 0x40; // 语句
    } // 块结束
    let mut body = enc_str("MQTT"); // 变量 body
    body.push(4); // protocol level (3.1.1)
    body.push(flags); // 语句
    body.extend_from_slice(&[0x00, 0x3C]); // keepalive 60s
    body.extend(enc_str(client_id)); // 语句
    if let Some(u) = username { // 条件分支
        body.extend(enc_str(u)); // 语句
    } // 块结束
    if let Some(p) = password { // 条件分支
        body.extend(enc_str(p)); // 语句
    } // 块结束
    body // 语句
} // 块结束

fn subscribe_body(packet_id: u16, topic: &str, qos: u8) -> Vec<u8> { // 函数 subscribe_body
    let mut body = Vec::new(); // 变量 body
    body.push((packet_id >> 8) as u8); // 语句
    body.push((packet_id & 0xff) as u8); // 语句
    body.extend(enc_str(topic)); // 语句
    body.push(qos & 0x03); // requested QoS
    body // 语句
} // 块结束

fn publish_body(topic: &str, payload: &str, packet_id: Option<u16>) -> Vec<u8> { // 函数 publish_body
    let mut body = enc_str(topic); // 变量 body
    if let Some(pid) = packet_id { // 条件分支
        body.push((pid >> 8) as u8); // 语句
        body.push((pid & 0xff) as u8); // 语句
    } // 块结束
    body.extend_from_slice(payload.as_bytes()); // 语句
    body // 语句
} // 块结束

fn read_packet(stream: &mut TcpStream) -> Result<(u8, Vec<u8>)> { // 函数 read_packet
    let mut first = [0u8; 1]; // 变量 first
    stream.read_exact(&mut first).context("read mqtt first byte")?; // 语句
    let mut len = 0usize; // 变量 len
    let mut mult = 1usize; // 变量 mult
    loop { // 循环
        let mut b = [0u8; 1]; // 变量 b
        stream.read_exact(&mut b)?; // 语句
        len += (b[0] & 0x7f) as usize * mult; // 语句
        if b[0] & 0x80 == 0 { // 条件分支
            break; // 语句
        } // 块结束
        mult *= 128; // 语句
        if mult > 128 * 128 * 128 * 128 { // 条件分支
            return Err(anyhow!("malformed mqtt remaining length")); // 返回
        } // 块结束
    } // 块结束
    let mut body = vec![0u8; len]; // 变量 body
    if len > 0 { // 条件分支
        stream.read_exact(&mut body)?; // 语句
    } // 块结束
    Ok((first[0], body)) // 调用 Ok
} // 块结束

/// Full configuration for an MQTT session.
pub struct MqttConfig<'a> { // 结构体 MqttConfig
    pub host: &'a str, // 字段 host
    pub port: u16, // 字段 port
    pub client_id: Option<&'a str>, // 字段 client_id
    pub username: Option<&'a str>, // 字段 username
    pub password: Option<&'a str>, // 字段 password
    pub topic: &'a str, // 字段 topic
    pub publish: Option<(&'a str, &'a str, u8)>, // topic, payload, QoS (0/1)
    pub qos: u8, // subscription QoS (0/1)
    pub listen: usize, // 字段 listen
    pub timeout_secs: u64, // 字段 timeout_secs
} // 块结束

/// Run a full MQTT session and return the event log as strings.
pub fn run_advanced(cfg: &MqttConfig) -> Result<Vec<String>> { // 函数 run_advanced
    let mut events: Vec<String> = Vec::new(); // 变量 events

    let mut stream = TcpStream::connect(format!("{}:{}", cfg.host, cfg.port)) // 变量 stream
        .with_context(|| format!("connect {}:{}", cfg.host, cfg.port))?; // 语句
    stream.set_read_timeout(Some(Duration::from_secs(cfg.timeout_secs)))?; // 语句

    let cid_owned; // 变量 cid_owned
    let client_id = match cfg.client_id { // 变量 client_id
        Some(c) => c, // 匹配分支
        None => { // 匹配分支
            cid_owned = format!("pmrs-{}", std::process::id()); // 语句
            cid_owned.as_str() // 语句
        } // 块结束
    }; // 块结束

    stream.write_all(&packet( // 语句
        0x10, // 语句
        &connect_body(client_id, cfg.username, cfg.password), // 语句
    ))?; // 语句
    let (t, _body) = read_packet(&mut stream)?; // 调用 let
    if t >> 4 != 2 { // 条件分支
        return Err(anyhow!("expected CONNACK, got packet type {}", t >> 4)); // 返回
    } // 块结束
    events.push(format!("connected: {}:{}", cfg.host, cfg.port)); // 语句

    // Subscribe (packet id 1).
    stream.write_all(&packet(0x82, &subscribe_body(1, cfg.topic, cfg.qos)))?; // 语句
    let (t, _) = read_packet(&mut stream)?; // 调用 let
    if t >> 4 != 9 { // 条件分支
        return Err(anyhow!("expected SUBACK, got {}", t >> 4)); // 返回
    } // 块结束
    events.push(format!("subscribed: {}", cfg.topic)); // 语句

    // Publish (packet id 2 when QoS 1).
    if let Some((pt, msg, pqos)) = cfg.publish { // 条件分支
        if pqos >= 1 { // 条件分支
            let pid: u16 = 2; // 变量 pid
            stream.write_all(&packet(0x32, &publish_body(pt, msg, Some(pid))))?; // 语句
            // Wait for PUBACK (type 9).
            match read_packet(&mut stream) { // 分支匹配
                Ok((first, _)) if first >> 4 == 9 => {} // 匹配分支
                Ok((first, _)) => { // 匹配分支
                    events.push(format!("<< expected PUBACK, got type {}", first >> 4)); // 语句
                } // 块结束
                Err(e) => events.push(format!("<< PUBACK read error: {}", e)), // 匹配分支
            } // 块结束
            events.push(format!("published (qos1) -> {} : {}", pt, msg)); // 语句
        } else { // 语句
            stream.write_all(&packet(0x30, &publish_body(pt, msg, None)))?; // 语句
            events.push(format!("published -> {} : {}", pt, msg)); // 语句
        } // 块结束
    } // 块结束

    // Receive loop. listen == 0 means "until the peer closes / read times out".
    let mut got = 0usize; // 变量 got
    loop { // 循环
        if cfg.listen > 0 && got >= cfg.listen { // 条件分支
            break; // 语句
        } // 块结束
        match read_packet(&mut stream) { // 分支匹配
            Ok((first, body)) => { // 匹配分支
                let kind = first >> 4; // 变量 kind
                if kind == 3 && body.len() >= 2 { // 条件分支
                    let tlen = ((body[0] as usize) << 8) | body[1] as usize; // 变量 tlen
                    if body.len() >= 2 + tlen { // 条件分支
                        let topic = String::from_utf8_lossy(&body[2..2 + tlen]).to_string(); // 变量 topic
                        let payload = String::from_utf8_lossy(&body[2 + tlen..]).to_string(); // 变量 payload
                        events.push(format!("<< {} : {}", topic, payload)); // 语句
                        got += 1; // 语句
                    } // 块结束
                } // 块结束
            } // 块结束
            Err(e) => { // 匹配分支
                events.push(format!("<< read error/idle: {}", e)); // 语句
                break; // 语句
            } // 块结束
        } // 块结束
    } // 块结束

    let _ = stream.write_all(&[0xE0, 0x00]); // 变量 _
    Ok(events) // 调用 Ok
} // 块结束

/// Backwards-compatible entry point used by the CLI. Prints events to stdout.
pub fn run( // 函数 run
    host: &str, // 字段 host
    port: u16, // 字段 port
    topic: &str, // 字段 topic
    publish: Option<(&str, &str)>, // 字段 publish
    listen: usize, // 字段 listen
    timeout_secs: u64, // 字段 timeout_secs
) -> Result<()> { // 语句
    let cfg = MqttConfig { // 变量 cfg
        host, // 语句
        port, // 语句
        client_id: None, // 字段 client_id
        username: None, // 字段 username
        password: None, // 字段 password
        topic, // 语句
        publish: publish.map(|(t, m)| (t, m, 0u8)), // 字段 publish
        qos: 0, // 字段 qos
        listen, // 语句
        timeout_secs, // 语句
    }; // 块结束
    for line in run_advanced(&cfg)? { // 循环
        println!("{}", line); // 语句
    } // 块结束
    Ok(()) // 调用 Ok
} // 块结束
