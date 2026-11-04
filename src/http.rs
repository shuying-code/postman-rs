//! Execute a Postman `Request` using `reqwest`, without any account or quota.
use crate::model::{Auth, Body, Request}; // 引入依赖
use crate::storage; // 引入依赖
use crate::variables::VarContext; // 引入依赖
use anyhow::{anyhow, Context, Result}; // 引入依赖
use base64::Engine; // 引入依赖
use reqwest::cookie::CookieStore; // 引入依赖
use reqwest::header::HeaderValue; // 引入依赖
use std::collections::HashMap; // 引入依赖
use std::path::PathBuf; // 引入依赖
use std::sync::{Arc, Mutex, OnceLock}; // 引入依赖
use std::time::Instant; // 引入依赖

#[derive(Debug, Clone)] // 语句
pub struct ResponseRecord { // 结构体 ResponseRecord
    pub status: String, // 字段 status
    pub code: u16, // 字段 code
    pub headers: Vec<(String, String)>, // 字段 headers
    pub body: String, // 字段 body
    pub binary: bool, // 字段 binary
    pub elapsed_ms: u128, // 字段 elapsed_ms
    pub final_url: String, // 字段 final_url
    /// 计时细分（毫秒，尽力而为；部分阶段可能不可用）
    pub dns_ms: Option<u128>, // 字段 dns_ms
    pub tcp_ms: Option<u128>, // 字段 tcp_ms
    pub ttfb_ms: Option<u128>, // 字段 ttfb_ms
    pub download_ms: Option<u128>, // 字段 download_ms
} // 块结束

/// Approximate host-based cookie jar persisted to `~/.postman-rs/cookies.json`.
struct PersistJar { // 结构体 PersistJar
    inner: Mutex<HashMap<String, Vec<String>>>, // 字段 inner
    path: PathBuf, // 字段 path
} // 块结束

impl PersistJar { // 实现
    fn load(path: PathBuf) -> Self { // 函数 load
        let map: HashMap<String, Vec<String>> = std::fs::read_to_string(&path) // 变量 map
            .ok() // 语句
            .and_then(|t| serde_json::from_str(&t).ok()) // 语句
            .unwrap_or_default(); // 语句
        PersistJar { // 语句
            inner: Mutex::new(map), // 字段 inner
            path, // 语句
        } // 块结束
    } // 块结束
    fn save(&self) { // 函数 save
        if let Ok(map) = self.inner.lock() { // 条件分支
            if let Ok(t) = serde_json::to_string_pretty(&*map) { // 条件分支
                let _ = std::fs::write(&self.path, t); // 变量 _
            } // 块结束
        } // 块结束
    } // 块结束
} // 块结束

impl CookieStore for PersistJar { // 实现
    fn set_cookies(&self, cookie_headers: &mut dyn Iterator<Item = &HeaderValue>, url: &url::Url) { // 函数 set_cookies
        let host = url.host_str().unwrap_or("").to_string(); // 变量 host
        { // 语句
            let mut map = self.inner.lock().unwrap(); // 变量 map
            let entry = map.entry(host).or_default(); // 变量 entry
            for hv in cookie_headers { // 循环
                if let Ok(s) = hv.to_str() { // 条件分支
                    let nv = s.split(';').next().unwrap_or("").trim().to_string(); // 变量 nv
                    if nv.is_empty() { // 条件分支
                        continue; // 语句
                    } // 块结束
                    let name = nv.split('=').next().unwrap_or("").to_string(); // 变量 name
                    entry.retain(|c| c.split('=').next() != Some(name.as_str())); // 语句
                    entry.push(nv); // 语句
                } // 块结束
            } // 块结束
        } // 块结束
        self.save(); // 语句
    } // 块结束
    fn cookies(&self, url: &url::Url) -> Option<HeaderValue> { // 函数 cookies
        let host = url.host_str().unwrap_or(""); // 变量 host
        let map = self.inner.lock().unwrap(); // 变量 map
        if let Some(list) = map.get(host) { // 条件分支
            if !list.is_empty() { // 条件分支
                if let Ok(hv) = HeaderValue::from_str(&list.join("; ")) { // 条件分支
                    return Some(hv); // 返回
                } // 块结束
            } // 块结束
        } // 块结束
        None // 语句
    } // 块结束
} // 块结束

fn cookie_jar() -> Arc<PersistJar> { // 函数 cookie_jar
    static JAR: OnceLock<Arc<PersistJar>> = OnceLock::new(); // 常量 JAR
    JAR.get_or_init(|| Arc::new(PersistJar::load(storage::home().join("cookies.json")))) // 语句
        .clone() // 语句
} // 块结束

/// List all persisted cookies as `(host, name, value)` (sorted).
pub fn cookies_dump() -> Vec<(String, String, String)> { // 函数 cookies_dump
    let jar = cookie_jar(); // 变量 jar
    let map = jar.inner.lock().unwrap(); // 变量 map
    let mut out = Vec::new(); // 变量 out
    for (host, list) in map.iter() { // 循环
        for nv in list { // 循环
            let name = nv.split('=').next().unwrap_or("").to_string(); // 变量 name
            let value = nv.splitn(2, '=').nth(1).unwrap_or("").to_string(); // 变量 value
            out.push((host.clone(), name, value)); // 语句
        } // 块结束
    } // 块结束
    out.sort(); // 语句
    out // 语句
} // 块结束

/// Add or replace a cookie (`name=value`) for a host and persist immediately.
pub fn cookie_set(host: &str, name: &str, value: &str) { // 函数 cookie_set
    if host.trim().is_empty() || name.trim().is_empty() { // 条件分支
        return; // 返回
    } // 块结束
    let jar = cookie_jar(); // 变量 jar
    { // 语句
        let mut map = jar.inner.lock().unwrap(); // 变量 map
        let entry = map.entry(host.trim().to_string()).or_default(); // 变量 entry
        entry.retain(|c| c.split('=').next() != Some(name)); // 语句
        entry.push(format!("{}={}", name, value)); // 语句
    } // 块结束
    jar.save(); // 语句
} // 块结束

/// Delete a specific cookie by `(host, name)`; empty name removes all for the host.
pub fn cookie_delete(host: &str, name: &str) { // 函数 cookie_delete
    let jar = cookie_jar(); // 变量 jar
    { // 语句
        let mut map = jar.inner.lock().unwrap(); // 变量 map
        if name.is_empty() { // 条件分支
            map.remove(host); // 语句
        } else if let Some(entry) = map.get_mut(host) { // 条件分支
            entry.retain(|c| c.split('=').next() != Some(name)); // 语句
            if entry.is_empty() { // 条件分支
                map.remove(host); // 语句
            } // 块结束
        } // 块结束
    } // 块结束
    jar.save(); // 语句
} // 块结束

/// Remove every persisted cookie.
pub fn cookie_clear() { // 函数 cookie_clear
    let jar = cookie_jar(); // 变量 jar
    { // 语句
        let mut map = jar.inner.lock().unwrap(); // 变量 map
        map.clear(); // 语句
    } // 块结束
    jar.save(); // 语句
} // 块结束

fn env_u64(name: &str) -> Option<u64> { // 函数 env_u64
    std::env::var(name).ok().and_then(|s| s.trim().parse().ok()) // 调用 var
} // 块结束

/// Per-request network settings (GUI/CLI can override the process-global env defaults).
#[derive(Debug, Clone, Default)] // 语句
pub struct NetConfig { // 结构体 NetConfig
    pub timeout_secs: u64, // 字段 timeout_secs
    pub connect_timeout_secs: Option<u64>, // 字段 connect_timeout_secs
    /// Some(false) disables redirects; Some(true)/None keeps redirect following.
    pub follow_redirects: Option<bool>, // 字段 follow_redirects
    pub max_redirects: Option<u64>, // 字段 max_redirects
    pub proxy: Option<String>, // 字段 proxy
    pub insecure: bool, // 字段 insecure
    pub client_cert: Option<String>, // 字段 client_cert
    pub client_key: Option<String>, // 字段 client_key
} // 块结束

impl NetConfig { // 实现
    pub fn from_env(timeout_secs: u64) -> Self { // 函数 from_env
        NetConfig { // 语句
            timeout_secs, // 语句
            connect_timeout_secs: env_u64("PMRS_CONNECT_TIMEOUT"), // 字段 connect_timeout_secs
            follow_redirects: None, // 字段 follow_redirects
            max_redirects: env_u64("PMRS_MAX_REDIRECTS"), // 字段 max_redirects
            proxy: std::env::var("PMRS_PROXY").ok(), // 字段 proxy
            insecure: std::env::var("PMRS_INSECURE").ok().as_deref() == Some("1"), // 字段 insecure
            client_cert: std::env::var("PMRS_CLIENT_CERT").ok(), // 字段 client_cert
            client_key: std::env::var("PMRS_CLIENT_KEY").ok(), // 字段 client_key
        } // 块结束
    } // 块结束
} // 块结束

fn build_client_cfg(cfg: &NetConfig) -> Result<reqwest::blocking::Client> { // 函数 build_client_cfg
    let mut b = reqwest::blocking::Client::builder() // 变量 b
        .timeout(std::time::Duration::from_secs(cfg.timeout_secs)) // 语句
        .connect_timeout(std::time::Duration::from_secs( // 语句
            cfg.connect_timeout_secs.unwrap_or(cfg.timeout_secs), // 语句
        )) // 语句
        .user_agent("postman-rs/0.1 (+local, account-free)") // 语句
        .cookie_provider(cookie_jar()); // 语句
    match cfg.follow_redirects { // 分支匹配
        Some(false) => b = b.redirect(reqwest::redirect::Policy::none()), // 匹配分支
        _ => { // 匹配分支
            b = b.redirect(reqwest::redirect::Policy::limited( // 语句
                cfg.max_redirects.unwrap_or(10) as usize, // 语句
            )) // 语句
        } // 块结束
    } // 块结束
    if let Some(proxy) = &cfg.proxy { // 条件分支
        if !proxy.trim().is_empty() { // 条件分支
            b = b.proxy(reqwest::Proxy::all(proxy.trim()).context("invalid proxy")?); // 语句
        } // 块结束
    } // 块结束
    if cfg.insecure { // 条件分支
        b = b.danger_accept_invalid_certs(true); // 语句
    } // 块结束
    if let Ok(ca) = std::env::var("PMRS_CA_BUNDLE") { // 条件分支
        if !ca.trim().is_empty() { // 条件分支
            let pem = std::fs::read(ca.trim()).context("read PMRS_CA_BUNDLE")?; // 变量 pem
            b = b.add_root_certificate(reqwest::Certificate::from_pem(&pem).context("parse CA")?); // 语句
        } // 块结束
    } // 块结束
    if let (Some(cert), Some(key)) = (&cfg.client_cert, &cfg.client_key) { // 条件分支
        if !cert.trim().is_empty() && !key.trim().is_empty() { // 条件分支
            let mut pem = std::fs::read(cert.trim()).context("read client cert")?; // 变量 pem
            pem.extend(std::fs::read(key.trim()).context("read client key")?); // 语句
            b = b.identity(reqwest::Identity::from_pem(&pem).context("parse client identity")?); // 语句
        } // 块结束
    } // 块结束
    b.build().context("failed to build HTTP client") // 语句
} // 块结束

fn build_client(timeout_secs: u64) -> Result<reqwest::blocking::Client> { // 函数 build_client
    build_client_cfg(&NetConfig::from_env(timeout_secs)) // 调用 build_client_cfg
} // 块结束

fn resolve_auth_header(auth: &Auth, ctx: &VarContext) -> Result<Option<(String, String)>> { // 函数 resolve_auth_header
    let out = match auth.kind.to_ascii_lowercase().as_str() { // 变量 out
        "" | "none" | "noauth" | "inherit" => None, // 匹配分支
        "bearer" => auth // 匹配分支
            .bearer_token() // 语句
            .map(|t| ("Authorization".to_string(), format!("Bearer {}", ctx.resolve(t)))), // 语句
        "basic" => { // 匹配分支
            let u = auth.basic_user().unwrap_or(""); // 变量 u
            let p = auth.basic_pass().unwrap_or(""); // 变量 p
            let raw = format!("{}:{}", ctx.resolve(u), ctx.resolve(p)); // 变量 raw
            let enc = base64::engine::general_purpose::STANDARD.encode(raw.as_bytes()); // 变量 enc
            Some(("Authorization".to_string(), format!("Basic {}", enc))) // 调用 Some
        } // 块结束
        "apikey" => { // 匹配分支
            if auth.apikey_in().eq_ignore_ascii_case("query") { // 条件分支
                None // handled via query string
            } else { // 语句
                let k = auth.apikey_key().unwrap_or(""); // 变量 k
                let v = auth.apikey_value().unwrap_or(""); // 变量 v
                Some((ctx.resolve(k), ctx.resolve(v))) // 调用 Some
            } // 块结束
        } // 块结束
        "oauth2" => None, // handled by oauth2_token
        "oauth1" | "awsv4" | "jwt" | "digest" | "ntlm" => None, // handled specially in execute()
        other => { // 匹配分支
            return Err(anyhow!( // 返回
                "auth type '{}' is not supported by postman-rs (supported: none/basic/bearer/apikey/oauth2/oauth1/awsv4/jwt/digest/ntlm)", // 语句
                other // 语句
            )) // 语句
        } // 块结束
    }; // 块结束
    Ok(out) // 调用 Ok
} // 块结束

// ---------------------------------------------------------------------------
// 额外鉴权：OAuth1 / AWS SigV4 / JWT / Digest（NTLM 明确不支持）
// ---------------------------------------------------------------------------
fn hex_lower(b: &[u8]) -> String { // 函数 hex_lower
    const H: &[u8; 16] = b"0123456789abcdef"; // 常量 H
    let mut s = String::with_capacity(b.len() * 2); // 变量 s
    for &x in b { // 循环
        s.push(H[(x >> 4) as usize] as char); // 语句
        s.push(H[(x & 0xf) as usize] as char); // 语句
    } // 块结束
    s // 语句
} // 块结束

fn hmac_sha1(key: &[u8], msg: &[u8]) -> Vec<u8> { // 函数 hmac_sha1
    use hmac::{Hmac, Mac}; // 引入依赖
    let mut m = Hmac::<sha1::Sha1>::new_from_slice(key).expect("hmac key"); // 变量 m
    m.update(msg); // 语句
    m.finalize().into_bytes().to_vec() // 语句
} // 块结束

fn hmac_sha256(key: &[u8], msg: &[u8]) -> Vec<u8> { // 函数 hmac_sha256
    use hmac::{Hmac, Mac}; // 引入依赖
    let mut m = Hmac::<sha2::Sha256>::new_from_slice(key).expect("hmac key"); // 变量 m
    m.update(msg); // 语句
    m.finalize().into_bytes().to_vec() // 语句
} // 块结束

fn sha256_bytes(b: &[u8]) -> Vec<u8> { // 函数 sha256_bytes
    use sha2::Digest; // 引入依赖
    let mut h = sha2::Sha256::new(); // 变量 h
    h.update(b); // 语句
    h.finalize().to_vec() // 语句
} // 块结束

/// RFC3986 percent-encoding (unreserved kept).
fn pct(s: &str) -> String { // 函数 pct
    let mut o = String::new(); // 变量 o
    for b in s.bytes() { // 循环
        let c = b as char; // 变量 c
        if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '~') { // 条件分支
            o.push(c); // 语句
        } else { // 语句
            o.push_str(&format!("%{:02X}", b)); // 语句
        } // 块结束
    } // 块结束
    o // 语句
} // 块结束

fn extra_params(auth: &Auth, which: &str) -> HashMap<String, String> { // 函数 extra_params
    let mut m = HashMap::new(); // 变量 m
    if let Some(serde_json::Value::Array(arr)) = auth.extra.get(which) { // 条件分支
        for it in arr { // 循环
            if let (Some(k), Some(v)) = ( // 条件分支
                it.get("key").and_then(|x| x.as_str()), // 语句
                it.get("value"), // 语句
            ) { // 语句
                let val = v // 变量 val
                    .as_str() // 语句
                    .map(|s| s.to_string()) // 语句
                    .unwrap_or_else(|| v.to_string()); // 语句
                m.insert(k.to_string(), val); // 语句
            } // 块结束
        } // 块结束
    } // 块结束
    m // 语句
} // 块结束

fn body_bytes_for_sign(body: &Option<Body>, ctx: &VarContext) -> Vec<u8> { // 函数 body_bytes_for_sign
    let Some(b) = body else { return Vec::new() }; // 变量 Some
    match b.mode.as_str() { // 分支匹配
        "raw" => ctx.resolve(b.raw.as_deref().unwrap_or("")).into_bytes(), // 匹配分支
        "urlencoded" => { // 匹配分支
            let pairs: Vec<String> = b // 变量 pairs
                .urlencoded // 语句
                .iter() // 语句
                .filter(|kv| !kv.disabled) // 语句
                .map(|kv| format!("{}={}", kv.key, kv.value)) // 语句
                .collect(); // 语句
            pairs.join("&").into_bytes() // 语句
        } // 块结束
        "graphql" => { // 匹配分支
            let g = b.graphql.clone().unwrap_or_default(); // 变量 g
            format!("{{\"query\":{},\"variables\":{}}}", g.query, if g.variables.trim().is_empty() { "{}".into() } else { g.variables }).into_bytes() // 语句
        } // 块结束
        _ => Vec::new(), // 匹配分支
    } // 块结束
} // 块结束

fn oauth1_header(auth: &Auth, method: &str, url: &str, ctx: &VarContext) -> Option<String> { // 函数 oauth1_header
    let p = extra_params(auth, "oauth1"); // 变量 p
    let get = |k: &str| p.get(k).map(|s| ctx.resolve(s)).unwrap_or_default(); // 变量 get
    let consumer_key = get("consumerKey"); // 变量 consumer_key
    if consumer_key.is_empty() { // 条件分支
        return None; // 返回
    } // 块结束
    let consumer_secret = get("consumerSecret"); // 变量 consumer_secret
    let token = get("token"); // 变量 token
    let token_secret = get("tokenSecret"); // 变量 token_secret
    let nonce = uuid::Uuid::new_v4().simple().to_string(); // 变量 nonce
    let ts = chrono::Utc::now().timestamp().to_string(); // 变量 ts
    let parsed = url::Url::parse(url).ok(); // 变量 parsed
    let base_url = parsed // 变量 base_url
        .as_ref() // 语句
        .map(|u| format!("{}://{}{}", u.scheme(), u.host_str().unwrap_or(""), u.path())) // 语句
        .unwrap_or_else(|| url.split('?').next().unwrap_or(url).to_string()); // 语句
    let mut params: Vec<(String, String)> = Vec::new(); // 变量 params
    if let Some(u) = &parsed { // 条件分支
        for (k, v) in u.query_pairs() { // 循环
            params.push((k.to_string(), v.to_string())); // 语句
        } // 块结束
    } // 块结束
    params.push(("oauth_consumer_key".into(), consumer_key)); // 语句
    params.push(("oauth_nonce".into(), nonce.clone())); // 语句
    params.push(("oauth_signature_method".into(), "HMAC-SHA1".into())); // 语句
    params.push(("oauth_timestamp".into(), ts.clone())); // 语句
    params.push(("oauth_version".into(), "1.0".into())); // 语句
    if !token.is_empty() { // 条件分支
        params.push(("oauth_token".into(), token.clone())); // 语句
    } // 块结束
    let mut encoded: Vec<(String, String)> = params // 变量 encoded
        .iter() // 语句
        .map(|(k, v)| (pct(k), pct(v))) // 语句
        .collect(); // 语句
    encoded.sort(); // 语句
    let param_str = encoded // 变量 param_str
        .iter() // 语句
        .map(|(k, v)| format!("{}={}", k, v)) // 语句
        .collect::<Vec<_>>() // 语句
        .join("&"); // 语句
    let base = format!( // 变量 base
        "{}&{}&{}", // 语句
        method.to_uppercase(), // 语句
        pct(&base_url), // 调用 pct
        pct(&param_str) // 调用 pct
    ); // 块结束
    let key = format!("{}&{}", pct(&consumer_secret), pct(&token_secret)); // 变量 key
    let sig = base64::engine::general_purpose::STANDARD.encode(hmac_sha1(key.as_bytes(), base.as_bytes())); // 变量 sig
    let mut header_params = vec![ // 变量 header_params
        format!("oauth_consumer_key=\"{}\"", pct(&p.get("consumerKey").cloned().unwrap_or_default())), // 语句
        format!("oauth_nonce=\"{}\"", pct(&nonce)), // 语句
        format!("oauth_signature=\"{}\"", pct(&sig)), // 语句
        "oauth_signature_method=\"HMAC-SHA1\"".to_string(), // 语句
        format!("oauth_timestamp=\"{}\"", ts), // 语句
        "oauth_version=\"1.0\"".to_string(), // 语句
    ]; // 语句
    if !token.is_empty() { // 条件分支
        header_params.push(format!("oauth_token=\"{}\"", pct(&token))); // 语句
    } // 块结束
    Some(format!("OAuth {}", header_params.join(", "))) // 调用 Some
} // 块结束

fn jwt_bearer(auth: &Auth, ctx: &VarContext) -> Option<String> { // 函数 jwt_bearer
    let p = extra_params(auth, "jwt"); // 变量 p
    let token = p.get("token").map(|s| ctx.resolve(s)).unwrap_or_default(); // 变量 token
    if !token.is_empty() { // 条件分支
        return Some(token); // 返回
    } // 块结束
    let alg = p.get("algorithm").cloned().unwrap_or_else(|| "HS256".into()); // 变量 alg
    let secret = p.get("secret").map(|s| ctx.resolve(s)).unwrap_or_default(); // 变量 secret
    let payload = p // 变量 payload
        .get("payload") // 语句
        .map(|s| ctx.resolve(s)) // 语句
        .filter(|s| !s.trim().is_empty()) // 语句
        .unwrap_or_else(|| "{}".into()); // 语句
    let header = serde_json::json!({"alg": alg, "typ": "JWT"}).to_string(); // 变量 header
    let hp = format!( // 变量 hp
        "{}.{}", // 语句
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(header.as_bytes()), // 字段 base64
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(payload.as_bytes()) // 语句
    ); // 块结束
    let sig = match alg.as_str() { // 变量 sig
        "HS384" => { // 匹配分支
            use hmac::{Hmac, Mac}; // 引入依赖
            let mut m = Hmac::<sha2::Sha384>::new_from_slice(secret.as_bytes()).ok()?; // 变量 m
            m.update(hp.as_bytes()); // 语句
            m.finalize().into_bytes().to_vec() // 语句
        } // 块结束
        "HS512" => { // 匹配分支
            use hmac::{Hmac, Mac}; // 引入依赖
            let mut m = Hmac::<sha2::Sha512>::new_from_slice(secret.as_bytes()).ok()?; // 变量 m
            m.update(hp.as_bytes()); // 语句
            m.finalize().into_bytes().to_vec() // 语句
        } // 块结束
        _ => hmac_sha256(secret.as_bytes(), hp.as_bytes()), // 匹配分支
    }; // 块结束
    let jwt = format!( // 变量 jwt
        "{}.{}", // 语句
        hp, // 语句
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(sig) // 语句
    ); // 块结束
    Some(jwt) // 调用 Some
} // 块结束

fn awsv4_headers( // 函数 awsv4_headers
    auth: &Auth, // 字段 auth
    method: &str, // 字段 method
    url: &str, // 字段 url
    body: &[u8], // 字段 body
    ctx: &VarContext, // 字段 ctx
) -> Option<Vec<(String, String)>> { // 语句
    let p = extra_params(auth, "awsv4"); // 变量 p
    let ak = p.get("accessKey").map(|s| ctx.resolve(s))?; // 变量 ak
    let sk = p.get("secretKey").map(|s| ctx.resolve(s))?; // 变量 sk
    if ak.is_empty() || sk.is_empty() { // 条件分支
        return None; // 返回
    } // 块结束
    let region = p.get("region").cloned().unwrap_or_else(|| "us-east-1".into()); // 变量 region
    let service = p.get("service").cloned().unwrap_or_else(|| "execute-api".into()); // 变量 service
    let session = p.get("sessionToken").map(|s| ctx.resolve(s)); // 变量 session
    let u = url::Url::parse(url).ok()?; // 变量 u
    let host = u.host_str()?.to_string(); // 变量 host
    let path = if u.path().is_empty() { "/".to_string() } else { u.path().to_string() }; // 变量 path
    let mut query: Vec<(String, String)> = u // 变量 query
        .query_pairs() // 语句
        .map(|(k, v)| (k.to_string(), v.to_string())) // 语句
        .collect(); // 语句
    query.sort(); // 语句
    let canonical_query = query // 变量 canonical_query
        .iter() // 语句
        .map(|(k, v)| format!("{}={}", pct(k), pct(v))) // 语句
        .collect::<Vec<_>>() // 语句
        .join("&"); // 语句
    let now = chrono::Utc::now(); // 变量 now
    let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string(); // 变量 amz_date
    let date = now.format("%Y%m%d").to_string(); // 变量 date
    let payload_hash = hex_lower(&sha256_bytes(body)); // 变量 payload_hash
    let mut canon_headers = format!( // 变量 canon_headers
        "host:{}\nx-amz-content-sha256:{}\nx-amz-date:{}\n", // 语句
        host, payload_hash, amz_date // 语句
    ); // 块结束
    let mut signed = "host;x-amz-content-sha256;x-amz-date".to_string(); // 变量 signed
    if let Some(t) = &session { // 条件分支
        if !t.is_empty() { // 条件分支
            canon_headers.push_str(&format!("x-amz-security-token:{}\n", t)); // 语句
            signed.push_str(";x-amz-security-token"); // 语句
        } // 块结束
    } // 块结束
    let canonical_request = format!( // 变量 canonical_request
        "{}\n{}\n{}\n{}\n{}\n{}", // 语句
        method.to_uppercase(), // 语句
        path, // 语句
        canonical_query, // 语句
        canon_headers, // 语句
        signed, // 语句
        payload_hash // 语句
    ); // 块结束
    let scope = format!("{}/{}/{}/aws4_request", date, region, service); // 变量 scope
    let string_to_sign = format!( // 变量 string_to_sign
        "AWS4-HMAC-SHA256\n{}\n{}\n{}", // 语句
        amz_date, // 语句
        scope, // 语句
        hex_lower(&sha256_bytes(canonical_request.as_bytes())) // 调用 hex_lower
    ); // 块结束
    let k_date = hmac_sha256(format!("AWS4{}", sk).as_bytes(), date.as_bytes()); // 变量 k_date
    let k_region = hmac_sha256(&k_date, region.as_bytes()); // 变量 k_region
    let k_service = hmac_sha256(&k_region, service.as_bytes()); // 变量 k_service
    let k_signing = hmac_sha256(&k_service, b"aws4_request"); // 变量 k_signing
    let signature = hex_lower(&hmac_sha256(&k_signing, string_to_sign.as_bytes())); // 变量 signature
    let authz = format!( // 变量 authz
        "AWS4-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}", // 语句
        ak, scope, signed, signature // 语句
    ); // 块结束
    let mut headers = vec![ // 变量 headers
        ("Host".to_string(), host), // 语句
        ("x-amz-date".to_string(), amz_date), // 语句
        ("x-amz-content-sha256".to_string(), payload_hash), // 语句
        ("Authorization".to_string(), authz), // 语句
    ]; // 语句
    if let Some(t) = session { // 条件分支
        if !t.is_empty() { // 条件分支
            headers.push(("x-amz-security-token".to_string(), t)); // 语句
        } // 块结束
    } // 块结束
    Some(headers) // 调用 Some
} // 块结束

fn digest_authorization( // 函数 digest_authorization
    auth: &Auth, // 字段 auth
    method: &str, // 字段 method
    url: &str, // 字段 url
    body: &[u8], // 字段 body
    challenge: &str, // 字段 challenge
) -> Option<String> { // 语句
    use md5::{Digest, Md5}; // 引入依赖
    let p = extra_params(auth, "digest"); // 变量 p
    let user = p.get("username").cloned().unwrap_or_default(); // 变量 user
    let pass = p.get("password").cloned().unwrap_or_default(); // 变量 pass
    let mut realm = String::new(); // 变量 realm
    let mut nonce = String::new(); // 变量 nonce
    let mut qop = String::new(); // 变量 qop
    let mut opaque = String::new(); // 变量 opaque
    let cleaned = challenge.trim().trim_start_matches("Digest "); // 变量 cleaned
    for part in cleaned.split(',') { // 循环
        if let Some((k, v)) = part.split_once('=') { // 条件分支
            let key = k.trim().trim_start_matches("Digest ").trim(); // 变量 key
            let val = v.trim().trim_matches('"'); // 变量 val
            match key.to_ascii_lowercase().as_str() { // 分支匹配
                "realm" => realm = val.to_string(), // 匹配分支
                "nonce" => nonce = val.to_string(), // 匹配分支
                "qop" => qop = val.to_string(), // 匹配分支
                "opaque" => opaque = val.to_string(), // 匹配分支
                _ => {} // 匹配分支
            } // 块结束
        } // 块结束
    } // 块结束
    let uri = url::Url::parse(url) // 变量 uri
        .map(|u| { // 语句
            let mut p = u.path().to_string(); // 变量 p
            if p.is_empty() { // 条件分支
                p = "/".to_string(); // 语句
            } // 块结束
            if let Some(q) = u.query() { // 条件分支
                p.push('?'); // 语句
                p.push_str(q); // 语句
            } // 块结束
            p // 语句
        }) // 语句
        .unwrap_or_else(|_| "/".to_string()); // 语句
    let ha1 = hex_lower(&Md5::digest(format!("{}:{}:{}", user, realm, pass).as_bytes())); // 变量 ha1
    let ha2 = hex_lower(&Md5::digest(format!("{}:{}", method.to_uppercase(), uri).as_bytes())); // 变量 ha2
    let cnonce = uuid::Uuid::new_v4().simple().to_string(); // 变量 cnonce
    let nc = "00000001"; // 变量 nc
    let q = qop.to_ascii_lowercase(); // 变量 q
    let response = if q.contains("auth-int") { // 变量 response
        let ha3 = hex_lower(&Md5::digest(body)); // 变量 ha3
        hex_lower(&Md5::digest( // 调用 hex_lower
            format!("{}:{}:{}:{}:auth-int:{}", ha1, nonce, nc, cnonce, ha3).as_bytes(), // 语句
        )) // 语句
    } else if !qop.is_empty() { // 条件分支
        hex_lower(&Md5::digest( // 调用 hex_lower
            format!("{}:{}:{}:{}:auth:{}", ha1, nonce, nc, cnonce, ha2).as_bytes(), // 语句
        )) // 语句
    } else { // 语句
        hex_lower(&Md5::digest(format!("{}:{}:{}", ha1, nonce, ha2).as_bytes())) // 调用 hex_lower
    }; // 块结束
    let mut parts = vec![ // 变量 parts
        format!("username=\"{}\"", user), // 语句
        format!("realm=\"{}\"", realm), // 语句
        format!("nonce=\"{}\"", nonce), // 语句
        format!("uri=\"{}\"", uri), // 语句
        format!("response=\"{}\"", response), // 语句
    ]; // 语句
    if !qop.is_empty() { // 条件分支
        parts.push("qop=auth".to_string()); // 语句
        parts.push(format!("nc={}", nc)); // 语句
        parts.push(format!("cnonce=\"{}\"", cnonce)); // 语句
    } // 块结束
    if !opaque.is_empty() { // 条件分支
        parts.push(format!("opaque=\"{}\"", opaque)); // 语句
    } // 块结束
    Some(format!("Digest {}", parts.join(", "))) // 调用 Some
} // 块结束

/// Acquire an OAuth2 access token (client_credentials / password / refresh_token).
fn oauth2_token(auth: &Auth, ctx: &VarContext, timeout_secs: u64) -> Result<(String, String)> { // 函数 oauth2_token
    let p = auth.oauth2_params(); // 变量 p
    let prefix = p.get("headerPrefix").cloned().unwrap_or_else(|| "Bearer".into()); // 变量 prefix
    let token_name = p // 变量 token_name
        .get("tokenName") // 语句
        .cloned() // 语句
        .unwrap_or_else(|| "access_token".into()); // 语句
    if let Some(t) = p.get("accessToken").or_else(|| p.get("token")) { // 条件分支
        if !t.trim().is_empty() { // 条件分支
            return Ok(( // 返回
                "Authorization".to_string(), // 语句
                format!("{} {}", prefix, ctx.resolve(t)), // 语句
            )); // 语句
        } // 块结束
    } // 块结束
    let grant = p // 变量 grant
        .get("grant_type") // 语句
        .cloned() // 语句
        .unwrap_or_else(|| "client_credentials".into()); // 语句
    let token_url = p // 变量 token_url
        .get("accessTokenUrl") // 语句
        .or_else(|| p.get("tokenUrl")) // 语句
        .cloned() // 语句
        .unwrap_or_default(); // 语句
    if token_url.trim().is_empty() { // 条件分支
        return Err(anyhow!("oauth2: missing accessTokenUrl")); // 返回
    } // 块结束
    if grant == "authorization_code" { // 条件分支
        return Err(anyhow!( // 返回
            "oauth2 authorization_code (interactive browser flow) not supported headless; provide accessToken" // 语句
        )); // 语句
    } // 块结束
    let client = build_client(timeout_secs)?; // 变量 client
    let mut form: Vec<(String, String)> = vec![("grant_type".into(), grant.clone())]; // 变量 form
    let put = |k: &str, form: &mut Vec<(String, String)>| { // 变量 put
        if let Some(v) = p.get(k) { // 条件分支
            if !v.is_empty() { // 条件分支
                form.push((k.to_string(), ctx.resolve(v))); // 语句
            } // 块结束
        } // 块结束
    }; // 块结束
    match grant.as_str() { // 分支匹配
        "client_credentials" | "password" => { // 匹配分支
            put("client_id", &mut form); // 调用 put
            put("client_secret", &mut form); // 调用 put
            put("scope", &mut form); // 调用 put
            if grant == "password" { // 条件分支
                put("username", &mut form); // 调用 put
                put("password", &mut form); // 调用 put
            } // 块结束
        } // 块结束
        "refresh_token" => { // 匹配分支
            put("client_id", &mut form); // 调用 put
            put("client_secret", &mut form); // 调用 put
            put("refresh_token", &mut form); // 调用 put
        } // 块结束
        other => return Err(anyhow!("oauth2 grant '{}' not supported", other)), // 匹配分支
    } // 块结束
    let resp = client // 变量 resp
        .post(ctx.resolve(&token_url)) // 语句
        .form(&form) // 语句
        .send() // 语句
        .context("oauth2 token request failed")?; // 语句
    let status = resp.status(); // 变量 status
    let text = resp.text().unwrap_or_default(); // 变量 text
    let v: serde_json::Value = serde_json::from_str(&text).unwrap_or(serde_json::Value::Null); // 变量 v
    let token = v // 变量 token
        .get(&token_name) // 语句
        .and_then(|x| x.as_str()) // 语句
        .or_else(|| v.get("access_token").and_then(|x| x.as_str())) // 语句
        .or_else(|| v.get("id_token").and_then(|x| x.as_str())) // 语句
        .ok_or_else(|| anyhow!("oauth2: no token in response (status {}): {}", status, text))?; // 语句
    Ok(( // 调用 Ok
        "Authorization".to_string(), // 语句
        format!("{} {}", prefix, token), // 语句
    )) // 语句
} // 块结束

/// Append `key=value` (URL-encoded) to a URL's query string.
fn append_query(url: &str, key: &str, value: &str) -> String { // 函数 append_query
    let sep = if url.contains('?') { '&' } else { '?' }; // 变量 sep
    format!( // 语句
        "{}{}{}={}", // 语句
        url, // 语句
        sep, // 语句
        urlencoding(key), // 调用 urlencoding
        urlencoding(value) // 调用 urlencoding
    ) // 块结束
} // 块结束

fn urlencoding(s: &str) -> String { // 函数 urlencoding
    let mut out = String::new(); // 变量 out
    for b in s.bytes() { // 循环
        match b { // 分支匹配
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => { // 匹配分支
                out.push(b as char) // 语句
            } // 块结束
            b' ' => out.push_str("%20"), // 匹配分支
            other => out.push_str(&format!("%{:02X}", other)), // 匹配分支
        } // 块结束
    } // 块结束
    out // 语句
} // 块结束

pub fn execute(req: &Request, ctx: &VarContext, timeout_secs: u64) -> Result<ResponseRecord> { // 函数 execute
    execute_cfg(req, ctx, &NetConfig::from_env(timeout_secs)) // 调用 execute_cfg
} // 块结束

/// Execute with explicit per-request network settings.
pub fn execute_cfg(req: &Request, ctx: &VarContext, cfg: &NetConfig) -> Result<ResponseRecord> { // 函数 execute_cfg
    let client = build_client_cfg(cfg)?; // 变量 client

    let method_str = ctx.resolve(&req.method).to_uppercase(); // 变量 method_str
    let method = reqwest::Method::from_bytes(method_str.as_bytes()) // 变量 method
        .map_err(|_| anyhow!("invalid HTTP method: {}", method_str))?; // 语句

    let mut url = ctx.resolve(&req.url.effective_url()); // 变量 url
    if url.trim().is_empty() { // 条件分支
        return Err(anyhow!("request URL is empty")); // 返回
    } // 块结束

    // Auth (query-based api key must be added to URL first).
    let mut headers: Vec<(String, String)> = Vec::new(); // 变量 headers
    if let Some(auth) = &req.auth { // 条件分支
        let kind = auth.kind.to_ascii_lowercase(); // 变量 kind
        if kind == "oauth2" { // 条件分支
            let (k, v) = oauth2_token(auth, ctx, cfg.timeout_secs)?; // 调用 let
            headers.push((k, v)); // 语句
        } else if kind == "apikey" && auth.apikey_in().eq_ignore_ascii_case("query") { // 条件分支
            let k = ctx.resolve(auth.apikey_key().unwrap_or("")); // 变量 k
            let v = ctx.resolve(auth.apikey_value().unwrap_or("")); // 变量 v
            if !k.is_empty() { // 条件分支
                url = append_query(&url, &k, &v); // 语句
            } // 块结束
        } else if kind == "oauth1" { // 条件分支
            if let Some(h) = oauth1_header(auth, &method_str, &url, ctx) { // 条件分支
                headers.push(("Authorization".to_string(), h)); // 语句
            } // 块结束
        } else if kind == "jwt" { // 条件分支
            if let Some(t) = jwt_bearer(auth, ctx) { // 条件分支
                headers.push(("Authorization".to_string(), format!("Bearer {}", t))); // 语句
            } // 块结束
        } else if kind == "awsv4" { // 条件分支
            let bb = body_bytes_for_sign(&req.body, ctx); // 变量 bb
            if let Some(hs) = awsv4_headers(auth, &method_str, &url, &bb, ctx) { // 条件分支
                headers.extend(hs); // 语句
            } // 块结束
        } else if kind == "ntlm" { // 条件分支
            return Err(anyhow!("NTLM auth is not supported")); // 返回
        } else if let Some(h) = resolve_auth_header(auth, ctx)? { // 条件分支
            headers.push(h); // 语句
        } // 块结束
    } // 块结束

    // User headers.
    let mut has_content_type = false; // 变量 has_content_type
    for h in &req.header { // 循环
        if h.disabled { // 条件分支
            continue; // 语句
        } // 块结束
        let k = ctx.resolve(&h.key); // 变量 k
        let v = ctx.resolve(&h.value); // 变量 v
        if k.eq_ignore_ascii_case("content-type") { // 条件分支
            has_content_type = true; // 语句
        } // 块结束
        headers.push((k, v)); // 语句
    } // 块结束

    let build_req = || -> Result<reqwest::blocking::RequestBuilder> { // 变量 build_req
        let mut b = client.request(method.clone(), &url); // 变量 b
        for (k, v) in &headers { // 循环
            b = b.header(k, v); // 语句
        } // 块结束
        if let Some(body) = &req.body { // 条件分支
            b = apply_body(b, body, ctx, has_content_type)?; // 语句
        } // 块结束
        Ok(b) // 调用 Ok
    }; // 块结束
    let builder = build_req()?; // 变量 builder

    // 计时细分：DNS / TCP 为独立探测（尽力而为，失败则为 None）。
    let mut dns_ms: Option<u128> = None; // 变量 dns_ms
    if let Ok(parsed) = url::Url::parse(&url) { // 条件分支
        if let Some(host) = parsed.host_str() { // 条件分支
            let port = parsed.port_or_known_default().unwrap_or(443); // 变量 port
            let ds = Instant::now(); // 变量 ds
            { // 语句
                use std::net::ToSocketAddrs; // 引入依赖
                let _ = (host, port).to_socket_addrs().map(|it| it.count()); // 变量 _
            } // 块结束
            dns_ms = Some(ds.elapsed().as_millis()); // 语句
        } // 块结束
    } // 块结束

    let start = Instant::now(); // 变量 start
    let mut resp = builder // 变量 resp
        .send() // 语句
        .with_context(|| format!("request to {} failed", url))?; // 语句
    // Digest 认证：先发一次，收到 401 challenge 后计算摘要再发一次。
    if resp.status().as_u16() == 401 // 条件分支
        && req // 语句
            .auth // 语句
            .as_ref() // 语句
            .map(|a| a.kind.eq_ignore_ascii_case("digest")) // 语句
            .unwrap_or(false) // 语句
    { // 语句
        if let Some(chal) = resp // 条件分支
            .headers() // 语句
            .get("www-authenticate") // 语句
            .and_then(|v| v.to_str().ok()) // 语句
            .map(|s| s.to_string()) // 语句
        { // 语句
            if chal.to_ascii_lowercase().starts_with("digest") { // 条件分支
                let bb = body_bytes_for_sign(&req.body, ctx); // 变量 bb
                if let Some(authz) = digest_authorization( // 条件分支
                    req.auth.as_ref().unwrap(), // 语句
                    &method_str, // 语句
                    &url, // 语句
                    &bb, // 语句
                    &chal, // 语句
                ) { // 语句
                    let mut b2 = client.request(method.clone(), &url); // 变量 b2
                    for (k, v) in &headers { // 循环
                        b2 = b2.header(k, v); // 语句
                    } // 块结束
                    b2 = b2.header("Authorization", authz); // 语句
                    if let Some(body) = &req.body { // 条件分支
                        b2 = apply_body(b2, body, ctx, has_content_type)?; // 语句
                    } // 块结束
                    resp = b2 // 语句
                        .send() // 语句
                        .with_context(|| format!("request to {} failed (digest)", url))?; // 语句
                } // 块结束
            } // 块结束
        } // 块结束
    } // 块结束
    let ttfb_ms = start.elapsed(); // 变量 ttfb_ms

    let code = resp.status().as_u16(); // 变量 code
    let status = resp.status().canonical_reason().unwrap_or("").to_string(); // 变量 status
    let final_url = resp.url().to_string(); // 变量 final_url
    let resp_headers: Vec<(String, String)> = resp // 变量 resp_headers
        .headers() // 语句
        .iter() // 语句
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string())) // 语句
        .collect(); // 语句
    let raw = resp.bytes().unwrap_or_default(); // 变量 raw
    let total = start.elapsed(); // 变量 total
    let download_ms = total.saturating_sub(ttfb_ms); // 变量 download_ms
    let (body_text, binary) = match String::from_utf8(raw.to_vec()) { // 调用 let
        Ok(s) => (s, false), // 匹配分支
        Err(e) => (base64::engine::general_purpose::STANDARD.encode(e.into_bytes()), true), // 匹配分支
    }; // 块结束

    Ok(ResponseRecord { // 调用 Ok
        status, // 语句
        code, // 语句
        headers: resp_headers, // 字段 headers
        body: body_text, // 字段 body
        binary, // 语句
        elapsed_ms: total.as_millis(), // 字段 elapsed_ms
        final_url, // 语句
        dns_ms, // 语句
        tcp_ms: None, // 字段 tcp_ms
        ttfb_ms: Some(ttfb_ms.as_millis()), // 字段 ttfb_ms
        download_ms: Some(download_ms.as_millis()), // 字段 download_ms
    }) // 语句
} // 块结束

fn apply_body( // 函数 apply_body
    builder: reqwest::blocking::RequestBuilder, // 字段 builder
    body: &Body, // 字段 body
    ctx: &VarContext, // 字段 ctx
    has_content_type: bool, // 字段 has_content_type
) -> Result<reqwest::blocking::RequestBuilder> { // 语句
    match body.mode.as_str() { // 分支匹配
        "raw" => { // 匹配分支
            let raw = ctx.resolve(body.raw.as_deref().unwrap_or("")); // 变量 raw
            let mut b = builder.body(raw); // 变量 b
            if !has_content_type { // 条件分支
                if let Some(ct) = body.raw_content_type() { // 条件分支
                    b = b.header("Content-Type", ct); // 语句
                } // 块结束
            } // 块结束
            Ok(b) // 调用 Ok
        } // 块结束
        "urlencoded" => { // 匹配分支
            let pairs: Vec<(String, String)> = body // 变量 pairs
                .urlencoded // 语句
                .iter() // 语句
                .filter(|kv| !kv.disabled) // 语句
                .map(|kv| (ctx.resolve(&kv.key), ctx.resolve(&kv.value))) // 语句
                .collect(); // 语句
            Ok(builder.form(&pairs)) // 调用 Ok
        } // 块结束
        "formdata" => { // 匹配分支
            let mut form = reqwest::blocking::multipart::Form::new(); // 变量 form
            for kv in body.formdata.iter().filter(|kv| !kv.disabled) { // 循环
                let key = ctx.resolve(&kv.key); // 变量 key
                let is_file = kv // 变量 is_file
                    .typ // 语句
                    .as_deref() // 语句
                    .map(|t| t.eq_ignore_ascii_case("file")) // 语句
                    .unwrap_or(false); // 语句
                if is_file { // 条件分支
                    let path = kv // 变量 path
                        .src // 语句
                        .as_ref() // 语句
                        .and_then(|v| v.as_str()) // 语句
                        .map(|s| ctx.resolve(s)) // 语句
                        .unwrap_or_default(); // 语句
                    if path.is_empty() { // 条件分支
                        continue; // 语句
                    } // 块结束
                    let bytes = std::fs::read(&path) // 变量 bytes
                        .with_context(|| format!("cannot read form-data file: {}", path))?; // 语句
                    let fname = std::path::Path::new(&path) // 变量 fname
                        .file_name() // 语句
                        .map(|s| s.to_string_lossy().to_string()) // 语句
                        .unwrap_or_else(|| "file".to_string()); // 语句
                    let part = reqwest::blocking::multipart::Part::bytes(bytes).file_name(fname); // 变量 part
                    form = form.part(key, part); // 语句
                } else { // 语句
                    form = form.text(key, ctx.resolve(&kv.value)); // 语句
                } // 块结束
            } // 块结束
            Ok(builder.multipart(form)) // 调用 Ok
        } // 块结束
        "file" => { // 匹配分支
            let path = body // 变量 path
                .file // 语句
                .as_ref() // 语句
                .and_then(|f| f.src.as_ref()) // 语句
                .map(|s| ctx.resolve(s)) // 语句
                .unwrap_or_default(); // 语句
            if path.is_empty() { // 条件分支
                return Ok(builder); // 返回
            } // 块结束
            let bytes = // 变量 bytes
                std::fs::read(&path).with_context(|| format!("cannot read body file: {}", path))?; // 调用 read
            Ok(builder.body(bytes)) // 调用 Ok
        } // 块结束
        "graphql" => { // 匹配分支
            let gql = body.graphql.clone().unwrap_or_default(); // 变量 gql
            let query = ctx.resolve(&gql.query); // 变量 query
            let vars_text = ctx.resolve(&gql.variables); // 变量 vars_text
            let vars: serde_json::Value = if vars_text.trim().is_empty() { // 变量 vars
                serde_json::json!({}) // 语句
            } else { // 语句
                serde_json::from_str(&vars_text).unwrap_or(serde_json::json!({})) // 调用 from_str
            }; // 块结束
            let payload = serde_json::json!({ "query": query, "variables": vars }); // 变量 payload
            Ok(builder.json(&payload)) // 调用 Ok
        } // 块结束
        "" => Ok(builder), // 匹配分支
        other => Err(anyhow!("unsupported body mode: {}", other)), // 匹配分支
    } // 块结束
} // 块结束

/// Open a streaming HTTP response (used by SSE). Caller reads bytes/lines.
pub fn open_stream( // 函数 open_stream
    url: &str, // 字段 url
    headers: &[(String, String)], // 字段 headers
    timeout_secs: u64, // 字段 timeout_secs
) -> Result<reqwest::blocking::Response> { // 语句
    let client = build_client(timeout_secs)?; // 变量 client
    let mut rb = client.get(url); // 变量 rb
    for (k, v) in headers { // 循环
        rb = rb.header(k, v); // 语句
    } // 块结束
    rb.send().context("open stream failed") // 语句
} // 块结束

#[cfg(test)] // 语句
mod tests { // 模块 tests
    use super::*; // 引入依赖
    use crate::model::UrlSpec; // 引入依赖

    #[test] // 语句
    fn append_query_works() { // 函数 append_query_works
        assert_eq!(append_query("http://x/a", "k", "v"), "http://x/a?k=v"); // 语句
        assert_eq!(append_query("http://x/a?b=1", "k", "v"), "http://x/a?b=1&k=v"); // 语句
    } // 块结束

    #[test] // 语句
    fn empty_url_errors() { // 函数 empty_url_errors
        let req = Request { // 变量 req
            method: "GET".into(), // 字段 method
            url: UrlSpec::default(), // 字段 url
            ..Default::default() // 语句
        }; // 块结束
        let ctx = VarContext::new(); // 变量 ctx
        assert!(execute(&req, &ctx, 5).is_err()); // 语句
    } // 块结束
} // 块结束
