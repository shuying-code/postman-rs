//! Execute a Postman `Request` using `reqwest`, without any account or quota.
use crate::model::{Auth, Body, Request};
use crate::storage;
use crate::variables::VarContext;
use anyhow::{anyhow, Context, Result};
use base64::Engine;
use reqwest::cookie::CookieStore;
use reqwest::header::HeaderValue;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct ResponseRecord {
    pub status: String,
    pub code: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub binary: bool,
    pub elapsed_ms: u128,
    pub final_url: String,
    /// 计时细分（毫秒，尽力而为；部分阶段可能不可用）
    pub dns_ms: Option<u128>,
    pub tcp_ms: Option<u128>,
    pub ttfb_ms: Option<u128>,
    pub download_ms: Option<u128>,
}

/// Approximate host-based cookie jar persisted to `~/.postman-rs/cookies.json`.
struct PersistJar {
    inner: Mutex<HashMap<String, Vec<String>>>,
    path: PathBuf,
}

impl PersistJar {
    fn load(path: PathBuf) -> Self {
        let map: HashMap<String, Vec<String>> = std::fs::read_to_string(&path)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default();
        PersistJar {
            inner: Mutex::new(map),
            path,
        }
    }
    fn save(&self) {
        if let Ok(map) = self.inner.lock() {
            if let Ok(t) = serde_json::to_string_pretty(&*map) {
                let _ = std::fs::write(&self.path, t);
            }
        }
    }
}

impl CookieStore for PersistJar {
    fn set_cookies(&self, cookie_headers: &mut dyn Iterator<Item = &HeaderValue>, url: &url::Url) {
        let host = url.host_str().unwrap_or("").to_string();
        {
            let mut map = self.inner.lock().unwrap();
            let entry = map.entry(host).or_default();
            for hv in cookie_headers {
                if let Ok(s) = hv.to_str() {
                    let nv = s.split(';').next().unwrap_or("").trim().to_string();
                    if nv.is_empty() {
                        continue;
                    }
                    let name = nv.split('=').next().unwrap_or("").to_string();
                    entry.retain(|c| c.split('=').next() != Some(name.as_str()));
                    entry.push(nv);
                }
            }
        }
        self.save();
    }
    fn cookies(&self, url: &url::Url) -> Option<HeaderValue> {
        let host = url.host_str().unwrap_or("");
        let map = self.inner.lock().unwrap();
        if let Some(list) = map.get(host) {
            if !list.is_empty() {
                if let Ok(hv) = HeaderValue::from_str(&list.join("; ")) {
                    return Some(hv);
                }
            }
        }
        None
    }
}

fn cookie_jar() -> Arc<PersistJar> {
    static JAR: OnceLock<Arc<PersistJar>> = OnceLock::new();
    JAR.get_or_init(|| Arc::new(PersistJar::load(storage::home().join("cookies.json"))))
        .clone()
}

/// List all persisted cookies as `(host, name, value)` (sorted).
pub fn cookies_dump() -> Vec<(String, String, String)> {
    let jar = cookie_jar();
    let map = jar.inner.lock().unwrap();
    let mut out = Vec::new();
    for (host, list) in map.iter() {
        for nv in list {
            let name = nv.split('=').next().unwrap_or("").to_string();
            let value = nv.splitn(2, '=').nth(1).unwrap_or("").to_string();
            out.push((host.clone(), name, value));
        }
    }
    out.sort();
    out
}

/// Add or replace a cookie (`name=value`) for a host and persist immediately.
pub fn cookie_set(host: &str, name: &str, value: &str) {
    if host.trim().is_empty() || name.trim().is_empty() {
        return;
    }
    let jar = cookie_jar();
    {
        let mut map = jar.inner.lock().unwrap();
        let entry = map.entry(host.trim().to_string()).or_default();
        entry.retain(|c| c.split('=').next() != Some(name));
        entry.push(format!("{}={}", name, value));
    }
    jar.save();
}

/// Delete a specific cookie by `(host, name)`; empty name removes all for the host.
pub fn cookie_delete(host: &str, name: &str) {
    let jar = cookie_jar();
    {
        let mut map = jar.inner.lock().unwrap();
        if name.is_empty() {
            map.remove(host);
        } else if let Some(entry) = map.get_mut(host) {
            entry.retain(|c| c.split('=').next() != Some(name));
            if entry.is_empty() {
                map.remove(host);
            }
        }
    }
    jar.save();
}

/// Remove every persisted cookie.
pub fn cookie_clear() {
    let jar = cookie_jar();
    {
        let mut map = jar.inner.lock().unwrap();
        map.clear();
    }
    jar.save();
}

fn env_u64(name: &str) -> Option<u64> {
    std::env::var(name).ok().and_then(|s| s.trim().parse().ok())
}

/// Per-request network settings (GUI/CLI can override the process-global env defaults).
#[derive(Debug, Clone, Default)]
pub struct NetConfig {
    pub timeout_secs: u64,
    pub connect_timeout_secs: Option<u64>,
    /// Some(false) disables redirects; Some(true)/None keeps redirect following.
    pub follow_redirects: Option<bool>,
    pub max_redirects: Option<u64>,
    pub proxy: Option<String>,
    pub insecure: bool,
    pub client_cert: Option<String>,
    pub client_key: Option<String>,
}

impl NetConfig {
    pub fn from_env(timeout_secs: u64) -> Self {
        NetConfig {
            timeout_secs,
            connect_timeout_secs: env_u64("PMRS_CONNECT_TIMEOUT"),
            follow_redirects: None,
            max_redirects: env_u64("PMRS_MAX_REDIRECTS"),
            proxy: std::env::var("PMRS_PROXY").ok(),
            insecure: std::env::var("PMRS_INSECURE").ok().as_deref() == Some("1"),
            client_cert: std::env::var("PMRS_CLIENT_CERT").ok(),
            client_key: std::env::var("PMRS_CLIENT_KEY").ok(),
        }
    }
}

fn build_client_cfg(cfg: &NetConfig) -> Result<reqwest::blocking::Client> {
    let mut b = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(cfg.timeout_secs))
        .connect_timeout(std::time::Duration::from_secs(
            cfg.connect_timeout_secs.unwrap_or(cfg.timeout_secs),
        ))
        .user_agent("postman-rs/0.1 (+local, account-free)")
        .cookie_provider(cookie_jar());
    match cfg.follow_redirects {
        Some(false) => b = b.redirect(reqwest::redirect::Policy::none()),
        _ => {
            b = b.redirect(reqwest::redirect::Policy::limited(
                cfg.max_redirects.unwrap_or(10) as usize,
            ))
        }
    }
    if let Some(proxy) = &cfg.proxy {
        if !proxy.trim().is_empty() {
            b = b.proxy(reqwest::Proxy::all(proxy.trim()).context("invalid proxy")?);
        }
    }
    if cfg.insecure {
        b = b.danger_accept_invalid_certs(true);
    }
    if let Ok(ca) = std::env::var("PMRS_CA_BUNDLE") {
        if !ca.trim().is_empty() {
            let pem = std::fs::read(ca.trim()).context("read PMRS_CA_BUNDLE")?;
            b = b.add_root_certificate(reqwest::Certificate::from_pem(&pem).context("parse CA")?);
        }
    }
    if let (Some(cert), Some(key)) = (&cfg.client_cert, &cfg.client_key) {
        if !cert.trim().is_empty() && !key.trim().is_empty() {
            let mut pem = std::fs::read(cert.trim()).context("read client cert")?;
            pem.extend(std::fs::read(key.trim()).context("read client key")?);
            b = b.identity(reqwest::Identity::from_pem(&pem).context("parse client identity")?);
        }
    }
    b.build().context("failed to build HTTP client")
}

fn build_client(timeout_secs: u64) -> Result<reqwest::blocking::Client> {
    build_client_cfg(&NetConfig::from_env(timeout_secs))
}

fn resolve_auth_header(auth: &Auth, ctx: &VarContext) -> Result<Option<(String, String)>> {
    let out = match auth.kind.to_ascii_lowercase().as_str() {
        "" | "none" | "noauth" | "inherit" => None,
        "bearer" => auth
            .bearer_token()
            .map(|t| ("Authorization".to_string(), format!("Bearer {}", ctx.resolve(t)))),
        "basic" => {
            let u = auth.basic_user().unwrap_or("");
            let p = auth.basic_pass().unwrap_or("");
            let raw = format!("{}:{}", ctx.resolve(u), ctx.resolve(p));
            let enc = base64::engine::general_purpose::STANDARD.encode(raw.as_bytes());
            Some(("Authorization".to_string(), format!("Basic {}", enc)))
        }
        "apikey" => {
            if auth.apikey_in().eq_ignore_ascii_case("query") {
                None // handled via query string
            } else {
                let k = auth.apikey_key().unwrap_or("");
                let v = auth.apikey_value().unwrap_or("");
                Some((ctx.resolve(k), ctx.resolve(v)))
            }
        }
        "oauth2" => None, // handled by oauth2_token
        "oauth1" | "awsv4" | "jwt" | "digest" | "ntlm" => None, // handled specially in execute()
        other => {
            return Err(anyhow!(
                "auth type '{}' is not supported by postman-rs (supported: none/basic/bearer/apikey/oauth2/oauth1/awsv4/jwt/digest/ntlm)",
                other
            ))
        }
    };
    Ok(out)
}

// ---------------------------------------------------------------------------
// 额外鉴权：OAuth1 / AWS SigV4 / JWT / Digest（NTLM 明确不支持）
// ---------------------------------------------------------------------------
fn hex_lower(b: &[u8]) -> String {
    const H: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(b.len() * 2);
    for &x in b {
        s.push(H[(x >> 4) as usize] as char);
        s.push(H[(x & 0xf) as usize] as char);
    }
    s
}

fn hmac_sha1(key: &[u8], msg: &[u8]) -> Vec<u8> {
    use hmac::{Hmac, Mac};
    let mut m = Hmac::<sha1::Sha1>::new_from_slice(key).expect("hmac key");
    m.update(msg);
    m.finalize().into_bytes().to_vec()
}

fn hmac_sha256(key: &[u8], msg: &[u8]) -> Vec<u8> {
    use hmac::{Hmac, Mac};
    let mut m = Hmac::<sha2::Sha256>::new_from_slice(key).expect("hmac key");
    m.update(msg);
    m.finalize().into_bytes().to_vec()
}

fn sha256_bytes(b: &[u8]) -> Vec<u8> {
    use sha2::Digest;
    let mut h = sha2::Sha256::new();
    h.update(b);
    h.finalize().to_vec()
}

/// RFC3986 percent-encoding (unreserved kept).
fn pct(s: &str) -> String {
    let mut o = String::new();
    for b in s.bytes() {
        let c = b as char;
        if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '~') {
            o.push(c);
        } else {
            o.push_str(&format!("%{:02X}", b));
        }
    }
    o
}

fn extra_params(auth: &Auth, which: &str) -> HashMap<String, String> {
    let mut m = HashMap::new();
    if let Some(serde_json::Value::Array(arr)) = auth.extra.get(which) {
        for it in arr {
            if let (Some(k), Some(v)) = (
                it.get("key").and_then(|x| x.as_str()),
                it.get("value"),
            ) {
                let val = v
                    .as_str()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| v.to_string());
                m.insert(k.to_string(), val);
            }
        }
    }
    m
}

fn body_bytes_for_sign(body: &Option<Body>, ctx: &VarContext) -> Vec<u8> {
    let Some(b) = body else { return Vec::new() };
    match b.mode.as_str() {
        "raw" => ctx.resolve(b.raw.as_deref().unwrap_or("")).into_bytes(),
        "urlencoded" => {
            let pairs: Vec<String> = b
                .urlencoded
                .iter()
                .filter(|kv| !kv.disabled)
                .map(|kv| format!("{}={}", kv.key, kv.value))
                .collect();
            pairs.join("&").into_bytes()
        }
        "graphql" => {
            let g = b.graphql.clone().unwrap_or_default();
            format!("{{\"query\":{},\"variables\":{}}}", g.query, if g.variables.trim().is_empty() { "{}".into() } else { g.variables }).into_bytes()
        }
        _ => Vec::new(),
    }
}

fn oauth1_header(auth: &Auth, method: &str, url: &str, ctx: &VarContext) -> Option<String> {
    let p = extra_params(auth, "oauth1");
    let get = |k: &str| p.get(k).map(|s| ctx.resolve(s)).unwrap_or_default();
    let consumer_key = get("consumerKey");
    if consumer_key.is_empty() {
        return None;
    }
    let consumer_secret = get("consumerSecret");
    let token = get("token");
    let token_secret = get("tokenSecret");
    let nonce = uuid::Uuid::new_v4().simple().to_string();
    let ts = chrono::Utc::now().timestamp().to_string();
    let parsed = url::Url::parse(url).ok();
    let base_url = parsed
        .as_ref()
        .map(|u| format!("{}://{}{}", u.scheme(), u.host_str().unwrap_or(""), u.path()))
        .unwrap_or_else(|| url.split('?').next().unwrap_or(url).to_string());
    let mut params: Vec<(String, String)> = Vec::new();
    if let Some(u) = &parsed {
        for (k, v) in u.query_pairs() {
            params.push((k.to_string(), v.to_string()));
        }
    }
    params.push(("oauth_consumer_key".into(), consumer_key));
    params.push(("oauth_nonce".into(), nonce.clone()));
    params.push(("oauth_signature_method".into(), "HMAC-SHA1".into()));
    params.push(("oauth_timestamp".into(), ts.clone()));
    params.push(("oauth_version".into(), "1.0".into()));
    if !token.is_empty() {
        params.push(("oauth_token".into(), token.clone()));
    }
    let mut encoded: Vec<(String, String)> = params
        .iter()
        .map(|(k, v)| (pct(k), pct(v)))
        .collect();
    encoded.sort();
    let param_str = encoded
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&");
    let base = format!(
        "{}&{}&{}",
        method.to_uppercase(),
        pct(&base_url),
        pct(&param_str)
    );
    let key = format!("{}&{}", pct(&consumer_secret), pct(&token_secret));
    let sig = base64::engine::general_purpose::STANDARD.encode(hmac_sha1(key.as_bytes(), base.as_bytes()));
    let mut header_params = vec![
        format!("oauth_consumer_key=\"{}\"", pct(&p.get("consumerKey").cloned().unwrap_or_default())),
        format!("oauth_nonce=\"{}\"", pct(&nonce)),
        format!("oauth_signature=\"{}\"", pct(&sig)),
        "oauth_signature_method=\"HMAC-SHA1\"".to_string(),
        format!("oauth_timestamp=\"{}\"", ts),
        "oauth_version=\"1.0\"".to_string(),
    ];
    if !token.is_empty() {
        header_params.push(format!("oauth_token=\"{}\"", pct(&token)));
    }
    Some(format!("OAuth {}", header_params.join(", ")))
}

fn jwt_bearer(auth: &Auth, ctx: &VarContext) -> Option<String> {
    let p = extra_params(auth, "jwt");
    let token = p.get("token").map(|s| ctx.resolve(s)).unwrap_or_default();
    if !token.is_empty() {
        return Some(token);
    }
    let alg = p.get("algorithm").cloned().unwrap_or_else(|| "HS256".into());
    let secret = p.get("secret").map(|s| ctx.resolve(s)).unwrap_or_default();
    let payload = p
        .get("payload")
        .map(|s| ctx.resolve(s))
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "{}".into());
    let header = serde_json::json!({"alg": alg, "typ": "JWT"}).to_string();
    let hp = format!(
        "{}.{}",
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(header.as_bytes()),
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(payload.as_bytes())
    );
    let sig = match alg.as_str() {
        "HS384" => {
            use hmac::{Hmac, Mac};
            let mut m = Hmac::<sha2::Sha384>::new_from_slice(secret.as_bytes()).ok()?;
            m.update(hp.as_bytes());
            m.finalize().into_bytes().to_vec()
        }
        "HS512" => {
            use hmac::{Hmac, Mac};
            let mut m = Hmac::<sha2::Sha512>::new_from_slice(secret.as_bytes()).ok()?;
            m.update(hp.as_bytes());
            m.finalize().into_bytes().to_vec()
        }
        _ => hmac_sha256(secret.as_bytes(), hp.as_bytes()),
    };
    let jwt = format!(
        "{}.{}",
        hp,
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(sig)
    );
    Some(jwt)
}

fn awsv4_headers(
    auth: &Auth,
    method: &str,
    url: &str,
    body: &[u8],
    ctx: &VarContext,
) -> Option<Vec<(String, String)>> {
    let p = extra_params(auth, "awsv4");
    let ak = p.get("accessKey").map(|s| ctx.resolve(s))?;
    let sk = p.get("secretKey").map(|s| ctx.resolve(s))?;
    if ak.is_empty() || sk.is_empty() {
        return None;
    }
    let region = p.get("region").cloned().unwrap_or_else(|| "us-east-1".into());
    let service = p.get("service").cloned().unwrap_or_else(|| "execute-api".into());
    let session = p.get("sessionToken").map(|s| ctx.resolve(s));
    let u = url::Url::parse(url).ok()?;
    let host = u.host_str()?.to_string();
    let path = if u.path().is_empty() { "/".to_string() } else { u.path().to_string() };
    let mut query: Vec<(String, String)> = u
        .query_pairs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    query.sort();
    let canonical_query = query
        .iter()
        .map(|(k, v)| format!("{}={}", pct(k), pct(v)))
        .collect::<Vec<_>>()
        .join("&");
    let now = chrono::Utc::now();
    let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
    let date = now.format("%Y%m%d").to_string();
    let payload_hash = hex_lower(&sha256_bytes(body));
    let mut canon_headers = format!(
        "host:{}\nx-amz-content-sha256:{}\nx-amz-date:{}\n",
        host, payload_hash, amz_date
    );
    let mut signed = "host;x-amz-content-sha256;x-amz-date".to_string();
    if let Some(t) = &session {
        if !t.is_empty() {
            canon_headers.push_str(&format!("x-amz-security-token:{}\n", t));
            signed.push_str(";x-amz-security-token");
        }
    }
    let canonical_request = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        method.to_uppercase(),
        path,
        canonical_query,
        canon_headers,
        signed,
        payload_hash
    );
    let scope = format!("{}/{}/{}/aws4_request", date, region, service);
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{}\n{}\n{}",
        amz_date,
        scope,
        hex_lower(&sha256_bytes(canonical_request.as_bytes()))
    );
    let k_date = hmac_sha256(format!("AWS4{}", sk).as_bytes(), date.as_bytes());
    let k_region = hmac_sha256(&k_date, region.as_bytes());
    let k_service = hmac_sha256(&k_region, service.as_bytes());
    let k_signing = hmac_sha256(&k_service, b"aws4_request");
    let signature = hex_lower(&hmac_sha256(&k_signing, string_to_sign.as_bytes()));
    let authz = format!(
        "AWS4-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
        ak, scope, signed, signature
    );
    let mut headers = vec![
        ("Host".to_string(), host),
        ("x-amz-date".to_string(), amz_date),
        ("x-amz-content-sha256".to_string(), payload_hash),
        ("Authorization".to_string(), authz),
    ];
    if let Some(t) = session {
        if !t.is_empty() {
            headers.push(("x-amz-security-token".to_string(), t));
        }
    }
    Some(headers)
}

fn digest_authorization(
    auth: &Auth,
    method: &str,
    url: &str,
    body: &[u8],
    challenge: &str,
) -> Option<String> {
    use md5::{Digest, Md5};
    let p = extra_params(auth, "digest");
    let user = p.get("username").cloned().unwrap_or_default();
    let pass = p.get("password").cloned().unwrap_or_default();
    let mut realm = String::new();
    let mut nonce = String::new();
    let mut qop = String::new();
    let mut opaque = String::new();
    let cleaned = challenge.trim().trim_start_matches("Digest ");
    for part in cleaned.split(',') {
        if let Some((k, v)) = part.split_once('=') {
            let key = k.trim().trim_start_matches("Digest ").trim();
            let val = v.trim().trim_matches('"');
            match key.to_ascii_lowercase().as_str() {
                "realm" => realm = val.to_string(),
                "nonce" => nonce = val.to_string(),
                "qop" => qop = val.to_string(),
                "opaque" => opaque = val.to_string(),
                _ => {}
            }
        }
    }
    let uri = url::Url::parse(url)
        .map(|u| {
            let mut p = u.path().to_string();
            if p.is_empty() {
                p = "/".to_string();
            }
            if let Some(q) = u.query() {
                p.push('?');
                p.push_str(q);
            }
            p
        })
        .unwrap_or_else(|_| "/".to_string());
    let ha1 = hex_lower(&Md5::digest(format!("{}:{}:{}", user, realm, pass).as_bytes()));
    let ha2 = hex_lower(&Md5::digest(format!("{}:{}", method.to_uppercase(), uri).as_bytes()));
    let cnonce = uuid::Uuid::new_v4().simple().to_string();
    let nc = "00000001";
    let q = qop.to_ascii_lowercase();
    let response = if q.contains("auth-int") {
        let ha3 = hex_lower(&Md5::digest(body));
        hex_lower(&Md5::digest(
            format!("{}:{}:{}:{}:auth-int:{}", ha1, nonce, nc, cnonce, ha3).as_bytes(),
        ))
    } else if !qop.is_empty() {
        hex_lower(&Md5::digest(
            format!("{}:{}:{}:{}:auth:{}", ha1, nonce, nc, cnonce, ha2).as_bytes(),
        ))
    } else {
        hex_lower(&Md5::digest(format!("{}:{}:{}", ha1, nonce, ha2).as_bytes()))
    };
    let mut parts = vec![
        format!("username=\"{}\"", user),
        format!("realm=\"{}\"", realm),
        format!("nonce=\"{}\"", nonce),
        format!("uri=\"{}\"", uri),
        format!("response=\"{}\"", response),
    ];
    if !qop.is_empty() {
        parts.push("qop=auth".to_string());
        parts.push(format!("nc={}", nc));
        parts.push(format!("cnonce=\"{}\"", cnonce));
    }
    if !opaque.is_empty() {
        parts.push(format!("opaque=\"{}\"", opaque));
    }
    Some(format!("Digest {}", parts.join(", ")))
}

/// Acquire an OAuth2 access token (client_credentials / password / refresh_token).
fn oauth2_token(auth: &Auth, ctx: &VarContext, timeout_secs: u64) -> Result<(String, String)> {
    let p = auth.oauth2_params();
    let prefix = p.get("headerPrefix").cloned().unwrap_or_else(|| "Bearer".into());
    let token_name = p
        .get("tokenName")
        .cloned()
        .unwrap_or_else(|| "access_token".into());
    if let Some(t) = p.get("accessToken").or_else(|| p.get("token")) {
        if !t.trim().is_empty() {
            return Ok((
                "Authorization".to_string(),
                format!("{} {}", prefix, ctx.resolve(t)),
            ));
        }
    }
    let grant = p
        .get("grant_type")
        .cloned()
        .unwrap_or_else(|| "client_credentials".into());
    let token_url = p
        .get("accessTokenUrl")
        .or_else(|| p.get("tokenUrl"))
        .cloned()
        .unwrap_or_default();
    if token_url.trim().is_empty() {
        return Err(anyhow!("oauth2: missing accessTokenUrl"));
    }
    if grant == "authorization_code" {
        return Err(anyhow!(
            "oauth2 authorization_code (interactive browser flow) not supported headless; provide accessToken"
        ));
    }
    let client = build_client(timeout_secs)?;
    let mut form: Vec<(String, String)> = vec![("grant_type".into(), grant.clone())];
    let put = |k: &str, form: &mut Vec<(String, String)>| {
        if let Some(v) = p.get(k) {
            if !v.is_empty() {
                form.push((k.to_string(), ctx.resolve(v)));
            }
        }
    };
    match grant.as_str() {
        "client_credentials" | "password" => {
            put("client_id", &mut form);
            put("client_secret", &mut form);
            put("scope", &mut form);
            if grant == "password" {
                put("username", &mut form);
                put("password", &mut form);
            }
        }
        "refresh_token" => {
            put("client_id", &mut form);
            put("client_secret", &mut form);
            put("refresh_token", &mut form);
        }
        other => return Err(anyhow!("oauth2 grant '{}' not supported", other)),
    }
    let resp = client
        .post(ctx.resolve(&token_url))
        .form(&form)
        .send()
        .context("oauth2 token request failed")?;
    let status = resp.status();
    let text = resp.text().unwrap_or_default();
    let v: serde_json::Value = serde_json::from_str(&text).unwrap_or(serde_json::Value::Null);
    let token = v
        .get(&token_name)
        .and_then(|x| x.as_str())
        .or_else(|| v.get("access_token").and_then(|x| x.as_str()))
        .or_else(|| v.get("id_token").and_then(|x| x.as_str()))
        .ok_or_else(|| anyhow!("oauth2: no token in response (status {}): {}", status, text))?;
    Ok((
        "Authorization".to_string(),
        format!("{} {}", prefix, token),
    ))
}

/// Append `key=value` (URL-encoded) to a URL's query string.
fn append_query(url: &str, key: &str, value: &str) -> String {
    let sep = if url.contains('?') { '&' } else { '?' };
    format!(
        "{}{}{}={}",
        url,
        sep,
        urlencoding(key),
        urlencoding(value)
    )
}

fn urlencoding(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            b' ' => out.push_str("%20"),
            other => out.push_str(&format!("%{:02X}", other)),
        }
    }
    out
}

pub fn execute(req: &Request, ctx: &VarContext, timeout_secs: u64) -> Result<ResponseRecord> {
    execute_cfg(req, ctx, &NetConfig::from_env(timeout_secs))
}

/// Execute with explicit per-request network settings.
pub fn execute_cfg(req: &Request, ctx: &VarContext, cfg: &NetConfig) -> Result<ResponseRecord> {
    let client = build_client_cfg(cfg)?;

    let method_str = ctx.resolve(&req.method).to_uppercase();
    let method = reqwest::Method::from_bytes(method_str.as_bytes())
        .map_err(|_| anyhow!("invalid HTTP method: {}", method_str))?;

    let mut url = ctx.resolve(&req.url.effective_url());
    if url.trim().is_empty() {
        return Err(anyhow!("request URL is empty"));
    }

    // Auth (query-based api key must be added to URL first).
    let mut headers: Vec<(String, String)> = Vec::new();
    if let Some(auth) = &req.auth {
        let kind = auth.kind.to_ascii_lowercase();
        if kind == "oauth2" {
            let (k, v) = oauth2_token(auth, ctx, cfg.timeout_secs)?;
            headers.push((k, v));
        } else if kind == "apikey" && auth.apikey_in().eq_ignore_ascii_case("query") {
            let k = ctx.resolve(auth.apikey_key().unwrap_or(""));
            let v = ctx.resolve(auth.apikey_value().unwrap_or(""));
            if !k.is_empty() {
                url = append_query(&url, &k, &v);
            }
        } else if kind == "oauth1" {
            if let Some(h) = oauth1_header(auth, &method_str, &url, ctx) {
                headers.push(("Authorization".to_string(), h));
            }
        } else if kind == "jwt" {
            if let Some(t) = jwt_bearer(auth, ctx) {
                headers.push(("Authorization".to_string(), format!("Bearer {}", t)));
            }
        } else if kind == "awsv4" {
            let bb = body_bytes_for_sign(&req.body, ctx);
            if let Some(hs) = awsv4_headers(auth, &method_str, &url, &bb, ctx) {
                headers.extend(hs);
            }
        } else if kind == "ntlm" {
            return Err(anyhow!("NTLM auth is not supported"));
        } else if let Some(h) = resolve_auth_header(auth, ctx)? {
            headers.push(h);
        }
    }

    // User headers.
    let mut has_content_type = false;
    for h in &req.header {
        if h.disabled {
            continue;
        }
        let k = ctx.resolve(&h.key);
        let v = ctx.resolve(&h.value);
        if k.eq_ignore_ascii_case("content-type") {
            has_content_type = true;
        }
        headers.push((k, v));
    }

    let build_req = || -> Result<reqwest::blocking::RequestBuilder> {
        let mut b = client.request(method.clone(), &url);
        for (k, v) in &headers {
            b = b.header(k, v);
        }
        if let Some(body) = &req.body {
            b = apply_body(b, body, ctx, has_content_type)?;
        }
        Ok(b)
    };
    let builder = build_req()?;

    // 计时细分：DNS / TCP 为独立探测（尽力而为，失败则为 None）。
    let mut dns_ms: Option<u128> = None;
    if let Ok(parsed) = url::Url::parse(&url) {
        if let Some(host) = parsed.host_str() {
            let port = parsed.port_or_known_default().unwrap_or(443);
            let ds = Instant::now();
            {
                use std::net::ToSocketAddrs;
                let _ = (host, port).to_socket_addrs().map(|it| it.count());
            }
            dns_ms = Some(ds.elapsed().as_millis());
        }
    }

    let start = Instant::now();
    let mut resp = builder
        .send()
        .with_context(|| format!("request to {} failed", url))?;
    // Digest 认证：先发一次，收到 401 challenge 后计算摘要再发一次。
    if resp.status().as_u16() == 401
        && req
            .auth
            .as_ref()
            .map(|a| a.kind.eq_ignore_ascii_case("digest"))
            .unwrap_or(false)
    {
        if let Some(chal) = resp
            .headers()
            .get("www-authenticate")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
        {
            if chal.to_ascii_lowercase().starts_with("digest") {
                let bb = body_bytes_for_sign(&req.body, ctx);
                if let Some(authz) = digest_authorization(
                    req.auth.as_ref().unwrap(),
                    &method_str,
                    &url,
                    &bb,
                    &chal,
                ) {
                    let mut b2 = client.request(method.clone(), &url);
                    for (k, v) in &headers {
                        b2 = b2.header(k, v);
                    }
                    b2 = b2.header("Authorization", authz);
                    if let Some(body) = &req.body {
                        b2 = apply_body(b2, body, ctx, has_content_type)?;
                    }
                    resp = b2
                        .send()
                        .with_context(|| format!("request to {} failed (digest)", url))?;
                }
            }
        }
    }
    let ttfb_ms = start.elapsed();

    let code = resp.status().as_u16();
    let status = resp.status().canonical_reason().unwrap_or("").to_string();
    let final_url = resp.url().to_string();
    let resp_headers: Vec<(String, String)> = resp
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();
    let raw = resp.bytes().unwrap_or_default();
    let total = start.elapsed();
    let download_ms = total.saturating_sub(ttfb_ms);
    let (body_text, binary) = match String::from_utf8(raw.to_vec()) {
        Ok(s) => (s, false),
        Err(e) => (base64::engine::general_purpose::STANDARD.encode(e.into_bytes()), true),
    };

    Ok(ResponseRecord {
        status,
        code,
        headers: resp_headers,
        body: body_text,
        binary,
        elapsed_ms: total.as_millis(),
        final_url,
        dns_ms,
        tcp_ms: None,
        ttfb_ms: Some(ttfb_ms.as_millis()),
        download_ms: Some(download_ms.as_millis()),
    })
}

fn apply_body(
    builder: reqwest::blocking::RequestBuilder,
    body: &Body,
    ctx: &VarContext,
    has_content_type: bool,
) -> Result<reqwest::blocking::RequestBuilder> {
    match body.mode.as_str() {
        "raw" => {
            let raw = ctx.resolve(body.raw.as_deref().unwrap_or(""));
            let mut b = builder.body(raw);
            if !has_content_type {
                if let Some(ct) = body.raw_content_type() {
                    b = b.header("Content-Type", ct);
                }
            }
            Ok(b)
        }
        "urlencoded" => {
            let pairs: Vec<(String, String)> = body
                .urlencoded
                .iter()
                .filter(|kv| !kv.disabled)
                .map(|kv| (ctx.resolve(&kv.key), ctx.resolve(&kv.value)))
                .collect();
            Ok(builder.form(&pairs))
        }
        "formdata" => {
            let mut form = reqwest::blocking::multipart::Form::new();
            for kv in body.formdata.iter().filter(|kv| !kv.disabled) {
                let key = ctx.resolve(&kv.key);
                let is_file = kv
                    .typ
                    .as_deref()
                    .map(|t| t.eq_ignore_ascii_case("file"))
                    .unwrap_or(false);
                if is_file {
                    let path = kv
                        .src
                        .as_ref()
                        .and_then(|v| v.as_str())
                        .map(|s| ctx.resolve(s))
                        .unwrap_or_default();
                    if path.is_empty() {
                        continue;
                    }
                    let bytes = std::fs::read(&path)
                        .with_context(|| format!("cannot read form-data file: {}", path))?;
                    let fname = std::path::Path::new(&path)
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| "file".to_string());
                    let part = reqwest::blocking::multipart::Part::bytes(bytes).file_name(fname);
                    form = form.part(key, part);
                } else {
                    form = form.text(key, ctx.resolve(&kv.value));
                }
            }
            Ok(builder.multipart(form))
        }
        "file" => {
            let path = body
                .file
                .as_ref()
                .and_then(|f| f.src.as_ref())
                .map(|s| ctx.resolve(s))
                .unwrap_or_default();
            if path.is_empty() {
                return Ok(builder);
            }
            let bytes =
                std::fs::read(&path).with_context(|| format!("cannot read body file: {}", path))?;
            Ok(builder.body(bytes))
        }
        "graphql" => {
            let gql = body.graphql.clone().unwrap_or_default();
            let query = ctx.resolve(&gql.query);
            let vars_text = ctx.resolve(&gql.variables);
            let vars: serde_json::Value = if vars_text.trim().is_empty() {
                serde_json::json!({})
            } else {
                serde_json::from_str(&vars_text).unwrap_or(serde_json::json!({}))
            };
            let payload = serde_json::json!({ "query": query, "variables": vars });
            Ok(builder.json(&payload))
        }
        "" => Ok(builder),
        other => Err(anyhow!("unsupported body mode: {}", other)),
    }
}

/// Open a streaming HTTP response (used by SSE). Caller reads bytes/lines.
pub fn open_stream(
    url: &str,
    headers: &[(String, String)],
    timeout_secs: u64,
) -> Result<reqwest::blocking::Response> {
    let client = build_client(timeout_secs)?;
    let mut rb = client.get(url);
    for (k, v) in headers {
        rb = rb.header(k, v);
    }
    rb.send().context("open stream failed")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::UrlSpec;

    #[test]
    fn append_query_works() {
        assert_eq!(append_query("http://x/a", "k", "v"), "http://x/a?k=v");
        assert_eq!(append_query("http://x/a?b=1", "k", "v"), "http://x/a?b=1&k=v");
    }

    #[test]
    fn empty_url_errors() {
        let req = Request {
            method: "GET".into(),
            url: UrlSpec::default(),
            ..Default::default()
        };
        let ctx = VarContext::new();
        assert!(execute(&req, &ctx, 5).is_err());
    }
}
