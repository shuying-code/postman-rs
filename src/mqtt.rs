//! Minimal MQTT 3.1.1 client over **plain TCP**: connect, subscribe, publish,
//! receive.
//!
//! NOTE ON TLS: this module deliberately does not implement MQTT-over-TLS
//! (`mqtts://` / port 8883). Doing so cleanly would require pulling in a TLS
//! stack for the raw TCP stream; to keep the dependency footprint small we stay
//! on plain-text TCP. Username/password authentication, configurable client id,
//! subscription QoS and QoS-0/1 publishing *are* supported.
use anyhow::{anyhow, Context, Result};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

fn enc_str(s: &str) -> Vec<u8> {
    let b = s.as_bytes();
    let mut v = Vec::with_capacity(2 + b.len());
    v.push((b.len() >> 8) as u8);
    v.push((b.len() & 0xff) as u8);
    v.extend_from_slice(b);
    v
}

fn enc_len(mut len: usize) -> Vec<u8> {
    let mut out = Vec::new();
    loop {
        let mut byte = (len % 128) as u8;
        len /= 128;
        if len > 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if len == 0 {
            break;
        }
    }
    out
}

fn packet(first: u8, body: &[u8]) -> Vec<u8> {
    let mut v = vec![first];
    v.extend(enc_len(body.len()));
    v.extend_from_slice(body);
    v
}

/// Build a CONNECT packet body.
///
/// Connect flags (bit7 = username, bit6 = password, bit1 = clean session) and
/// the payload order (client id, then username, then password) follow the
/// MQTT 3.1.1 spec.
fn connect_body(
    client_id: &str,
    username: Option<&str>,
    password: Option<&str>,
) -> Vec<u8> {
    let mut flags = 0x02u8; // clean session
    if username.is_some() {
        flags |= 0x80;
    }
    if password.is_some() {
        flags |= 0x40;
    }
    let mut body = enc_str("MQTT");
    body.push(4); // protocol level (3.1.1)
    body.push(flags);
    body.extend_from_slice(&[0x00, 0x3C]); // keepalive 60s
    body.extend(enc_str(client_id));
    if let Some(u) = username {
        body.extend(enc_str(u));
    }
    if let Some(p) = password {
        body.extend(enc_str(p));
    }
    body
}

fn subscribe_body(packet_id: u16, topic: &str, qos: u8) -> Vec<u8> {
    let mut body = Vec::new();
    body.push((packet_id >> 8) as u8);
    body.push((packet_id & 0xff) as u8);
    body.extend(enc_str(topic));
    body.push(qos & 0x03); // requested QoS
    body
}

fn publish_body(topic: &str, payload: &str, packet_id: Option<u16>) -> Vec<u8> {
    let mut body = enc_str(topic);
    if let Some(pid) = packet_id {
        body.push((pid >> 8) as u8);
        body.push((pid & 0xff) as u8);
    }
    body.extend_from_slice(payload.as_bytes());
    body
}

fn read_packet(stream: &mut TcpStream) -> Result<(u8, Vec<u8>)> {
    let mut first = [0u8; 1];
    stream.read_exact(&mut first).context("read mqtt first byte")?;
    let mut len = 0usize;
    let mut mult = 1usize;
    loop {
        let mut b = [0u8; 1];
        stream.read_exact(&mut b)?;
        len += (b[0] & 0x7f) as usize * mult;
        if b[0] & 0x80 == 0 {
            break;
        }
        mult *= 128;
        if mult > 128 * 128 * 128 * 128 {
            return Err(anyhow!("malformed mqtt remaining length"));
        }
    }
    let mut body = vec![0u8; len];
    if len > 0 {
        stream.read_exact(&mut body)?;
    }
    Ok((first[0], body))
}

/// Full configuration for an MQTT session.
pub struct MqttConfig<'a> {
    pub host: &'a str,
    pub port: u16,
    pub client_id: Option<&'a str>,
    pub username: Option<&'a str>,
    pub password: Option<&'a str>,
    pub topic: &'a str,
    pub publish: Option<(&'a str, &'a str, u8)>, // topic, payload, QoS (0/1)
    pub qos: u8, // subscription QoS (0/1)
    pub listen: usize,
    pub timeout_secs: u64,
}

/// Run a full MQTT session and return the event log as strings.
pub fn run_advanced(cfg: &MqttConfig) -> Result<Vec<String>> {
    let mut events: Vec<String> = Vec::new();

    let mut stream = TcpStream::connect(format!("{}:{}", cfg.host, cfg.port))
        .with_context(|| format!("connect {}:{}", cfg.host, cfg.port))?;
    stream.set_read_timeout(Some(Duration::from_secs(cfg.timeout_secs)))?;

    let cid_owned;
    let client_id = match cfg.client_id {
        Some(c) => c,
        None => {
            cid_owned = format!("pmrs-{}", std::process::id());
            cid_owned.as_str()
        }
    };

    stream.write_all(&packet(
        0x10,
        &connect_body(client_id, cfg.username, cfg.password),
    ))?;
    let (t, _body) = read_packet(&mut stream)?;
    if t >> 4 != 2 {
        return Err(anyhow!("expected CONNACK, got packet type {}", t >> 4));
    }
    events.push(format!("connected: {}:{}", cfg.host, cfg.port));

    // Subscribe (packet id 1).
    stream.write_all(&packet(0x82, &subscribe_body(1, cfg.topic, cfg.qos)))?;
    let (t, _) = read_packet(&mut stream)?;
    if t >> 4 != 9 {
        return Err(anyhow!("expected SUBACK, got {}", t >> 4));
    }
    events.push(format!("subscribed: {}", cfg.topic));

    // Publish (packet id 2 when QoS 1).
    if let Some((pt, msg, pqos)) = cfg.publish {
        if pqos >= 1 {
            let pid: u16 = 2;
            stream.write_all(&packet(0x32, &publish_body(pt, msg, Some(pid))))?;
            // Wait for PUBACK (type 9).
            match read_packet(&mut stream) {
                Ok((first, _)) if first >> 4 == 9 => {}
                Ok((first, _)) => {
                    events.push(format!("<< expected PUBACK, got type {}", first >> 4));
                }
                Err(e) => events.push(format!("<< PUBACK read error: {}", e)),
            }
            events.push(format!("published (qos1) -> {} : {}", pt, msg));
        } else {
            stream.write_all(&packet(0x30, &publish_body(pt, msg, None)))?;
            events.push(format!("published -> {} : {}", pt, msg));
        }
    }

    // Receive loop. listen == 0 means "until the peer closes / read times out".
    let mut got = 0usize;
    loop {
        if cfg.listen > 0 && got >= cfg.listen {
            break;
        }
        match read_packet(&mut stream) {
            Ok((first, body)) => {
                let kind = first >> 4;
                if kind == 3 && body.len() >= 2 {
                    let tlen = ((body[0] as usize) << 8) | body[1] as usize;
                    if body.len() >= 2 + tlen {
                        let topic = String::from_utf8_lossy(&body[2..2 + tlen]).to_string();
                        let payload = String::from_utf8_lossy(&body[2 + tlen..]).to_string();
                        events.push(format!("<< {} : {}", topic, payload));
                        got += 1;
                    }
                }
            }
            Err(e) => {
                events.push(format!("<< read error/idle: {}", e));
                break;
            }
        }
    }

    let _ = stream.write_all(&[0xE0, 0x00]);
    Ok(events)
}

/// Backwards-compatible entry point used by the CLI. Prints events to stdout.
pub fn run(
    host: &str,
    port: u16,
    topic: &str,
    publish: Option<(&str, &str)>,
    listen: usize,
    timeout_secs: u64,
) -> Result<()> {
    let cfg = MqttConfig {
        host,
        port,
        client_id: None,
        username: None,
        password: None,
        topic,
        publish: publish.map(|(t, m)| (t, m, 0u8)),
        qos: 0,
        listen,
        timeout_secs,
    };
    for line in run_advanced(&cfg)? {
        println!("{}", line);
    }
    Ok(())
}
