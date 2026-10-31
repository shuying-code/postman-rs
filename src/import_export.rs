//! Import from Postman/OpenAPI/cURL/HAR and export to Postman Collection v2.1.
use crate::model::*;
use anyhow::{anyhow, Result};
use serde_json::Value;

pub enum Imported {
    Collection(Box<Collection>),
    Environment(Box<Environment>),
}

fn new_info(name: &str) -> CollectionInfo {
    CollectionInfo {
        name: name.to_string(),
        postman_id: Some(uuid::Uuid::new_v4().to_string()),
        schema: Some("https://schema.getpostman.com/json/collection/v2.1.0/collection.json".into()),
        description: None,
    }
}

fn header(k: &str, v: &str) -> Header {
    Header {
        key: k.to_string(),
        value: v.to_string(),
        ..Default::default()
    }
}

pub fn import_file(path: &str, format: &str) -> Result<Imported> {
    let text = std::fs::read_to_string(path)?;
    match format.to_ascii_lowercase().as_str() {
        "postman" | "auto" => import_postman_text(&text, path),
        "openapi" | "swagger" => {
            let v: Value = match serde_json::from_str(&text) {
                Ok(v) => v,
                Err(_) => serde_yaml::from_str::<Value>(&text)
                    .map_err(|e| anyhow!("parse openapi yaml: {}", e))?,
            };
            Ok(Imported::Collection(Box::new(import_openapi(&v))))
        }
        "curl" => Ok(Imported::Collection(Box::new(import_curl(&text)))),
        "har" => {
            let v: Value = serde_json::from_str(&text)?;
            Ok(Imported::Collection(Box::new(import_har(&v)?)))
        }
        other => Err(anyhow!("unsupported import format: {}", other)),
    }
}

pub fn import_postman_text(text: &str, path: &str) -> Result<Imported> {
    let v: Value = serde_json::from_str(text)?;
    // Postman Collection v1 (has requests/folders/order at top level, no `info`).
    if v.get("requests").is_some() || v.get("folders").is_some() {
        let c = import_postman_v1(&v, path);
        return Ok(Imported::Collection(Box::new(c)));
    }
    if v.get("info").is_some() {
        let mut c: Collection = serde_json::from_value(v)?;
        if c.info.name.is_empty() {
            c.info.name = std::path::Path::new(path)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "imported".into());
        }
        Ok(Imported::Collection(Box::new(c)))
    } else if v.get("values").is_some() || v.get("globals").is_some() {
        let e = parse_env_or_globals(&v);
        Ok(Imported::Environment(Box::new(e)))
    } else {
        Err(anyhow!("unrecognized Postman file (neither collection nor environment)"))
    }
}

/// Parse an environment OR a globals export into `Environment`
/// (globals keep `_postman_variable_scope = "globals"`).
fn parse_env_or_globals(v: &Value) -> Environment {
    let name = v
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or("Globals")
        .to_string();
    let list = v.get("values").or_else(|| v.get("globals"));
    let values: Vec<Variable> = list
        .and_then(|x| x.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|e| serde_json::from_value::<Variable>(e.clone()).ok())
                .collect()
        })
        .unwrap_or_default();
    let scope = v
        .get("_postman_variable_scope")
        .and_then(|s| s.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            if v.get("globals").is_some() {
                Some("globals".to_string())
            } else {
                None
            }
        });
    Environment {
        id: v.get("id").and_then(|x| x.as_str()).map(|s| s.to_string()),
        name,
        values,
        scope,
    }
}

fn parse_header_lines(s: &str) -> Vec<Header> {
    s.lines()
        .filter_map(|l| l.split_once(':').map(|(k, v)| header(k.trim(), v.trim())))
        .collect()
}

fn v1_kv_list(v: Option<&Value>) -> Vec<KV> {
    v.and_then(|x| x.as_array())
        .map(|a| {
            a.iter()
                .map(|e| KV {
                    key: e.get("key").and_then(|k| k.as_str()).unwrap_or("").to_string(),
                    value: e
                        .get("value")
                        .and_then(|vv| vv.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_default(),
                    disabled: !e.get("enabled").and_then(|x| x.as_bool()).unwrap_or(true),
                    ..Default::default()
                })
                .collect()
        })
        .unwrap_or_default()
}

fn v1_request_to_item(r: &Value) -> Item {
    let name = r
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or("request")
        .to_string();
    let method = r
        .get("method")
        .and_then(|m| m.as_str())
        .unwrap_or("GET")
        .to_string();
    let url = r
        .get("url")
        .and_then(|u| u.as_str().map(|s| s.to_string()))
        .or_else(|| {
            r.get("url")
                .and_then(|u| u.get("raw"))
                .and_then(|x| x.as_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_default();
    let mut headers = Vec::new();
    if let Some(hs) = r.get("headers").and_then(|h| h.as_str()) {
        headers = parse_header_lines(hs);
    } else if let Some(hs) = r.get("headerData").and_then(|h| h.as_array()) {
        for h in hs {
            if h.get("enabled").and_then(|e| e.as_bool()).unwrap_or(true) {
                headers.push(header(
                    h.get("key").and_then(|k| k.as_str()).unwrap_or(""),
                    h.get("value").and_then(|v| v.as_str()).unwrap_or(""),
                ));
            }
        }
    }
    let dm = r.get("dataMode").and_then(|d| d.as_str()).unwrap_or("");
    let body = match dm {
        "raw" => r
            .get("rawModeData")
            .and_then(|d| d.as_str())
            .map(|s| Body {
                mode: "raw".into(),
                raw: Some(s.to_string()),
                options: Some(serde_json::json!({"raw":{"language":"json"}})),
                ..Default::default()
            }),
        "urlencoded" => Some(Body {
            mode: "urlencoded".into(),
            urlencoded: v1_kv_list(r.get("data")),
            ..Default::default()
        }),
        "params" => Some(Body {
            mode: "formdata".into(),
            formdata: v1_kv_list(r.get("data")),
            ..Default::default()
        }),
        _ => None,
    };
    Item {
        name,
        request: Some(Request {
            method,
            url: UrlSpec::new(url),
            header: headers,
            body,
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn v1_items(container: &Value) -> Vec<Item> {
    let mut items = Vec::new();
    if let Some(folders) = container.get("folders").and_then(|f| f.as_array()) {
        for f in folders {
            let name = f
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("folder")
                .to_string();
            items.push(Item {
                name,
                item: Some(v1_items(f)),
                ..Default::default()
            });
        }
    }
    if let Some(reqs) = container.get("requests").and_then(|r| r.as_array()) {
        for r in reqs {
            items.push(v1_request_to_item(r));
        }
    }
    items
}

/// Import a Postman Collection v1 document into our v2.1-shaped model.
pub fn import_postman_v1(v: &Value, path: &str) -> Collection {
    let name = v
        .get("name")
        .and_then(|n| n.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            std::path::Path::new(path)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "Imported v1".into())
        });
    Collection {
        info: new_info(&name),
        item: v1_items(v),
        ..Default::default()
    }
}

/// Base URL for OpenAPI 3 (`servers[0].url`) or Swagger 2 (`schemes://host + basePath`).
fn openapi_base(v: &Value) -> String {
    if let Some(url) = v
        .get("servers")
        .and_then(|s| s.get(0))
        .and_then(|s| s.get("url"))
        .and_then(|u| u.as_str())
    {
        return url.trim_end_matches('/').to_string();
    }
    let host = v.get("host").and_then(|h| h.as_str()).unwrap_or("");
    let base_path = v.get("basePath").and_then(|b| b.as_str()).unwrap_or("");
    let scheme = v
        .get("schemes")
        .and_then(|s| s.as_array())
        .and_then(|a| a.get(0))
        .and_then(|s| s.as_str())
        .unwrap_or("https");
    if !host.is_empty() {
        format!("{}://{}{}", scheme, host, base_path.trim_end_matches('/'))
    } else {
        base_path.to_string()
    }
}

/// Build an `Auth` from OpenAPI/Swagger `security` + `securitySchemes`/`securityDefinitions`.
fn openapi_security_auth(v: &Value, op: &Value) -> Option<Auth> {
    let sec = op.get("security").or_else(|| v.get("security"))?;
    let first = sec.as_array()?.get(0)?.as_object()?;
    let (name, _) = first.iter().next()?;
    let schemes = v
        .get("components")
        .and_then(|c| c.get("securitySchemes"))
        .or_else(|| v.get("securityDefinitions"))?;
    let scheme = schemes.get(name)?;
    let typ = scheme.get("type").and_then(|t| t.as_str()).unwrap_or("");
    let kv = |k: &str, val: String| KV {
        key: k.to_string(),
        value: val,
        ..Default::default()
    };
    match typ {
        "http" => {
            let s = scheme
                .get("scheme")
                .and_then(|x| x.as_str())
                .unwrap_or("");
            if s.eq_ignore_ascii_case("bearer") {
                Some(Auth {
                    kind: "bearer".into(),
                    bearer: vec![kv("token", String::new())],
                    ..Default::default()
                })
            } else if s.eq_ignore_ascii_case("basic") {
                Some(Auth {
                    kind: "basic".into(),
                    basic: vec![kv("username", String::new()), kv("password", String::new())],
                    ..Default::default()
                })
            } else {
                None
            }
        }
        "basic" => Some(Auth {
            kind: "basic".into(),
            basic: vec![kv("username", String::new()), kv("password", String::new())],
            ..Default::default()
        }),
        "apiKey" => {
            let key = scheme.get("name").and_then(|x| x.as_str()).unwrap_or("");
            let loc = scheme.get("in").and_then(|x| x.as_str()).unwrap_or("header");
            Some(Auth {
                kind: "apikey".into(),
                apikey: vec![
                    kv("key", key.to_string()),
                    kv("value", String::new()),
                    kv("in", loc.to_string()),
                ],
                ..Default::default()
            })
        }
        "oauth2" => {
            let mut extra = serde_json::Map::new();
            extra.insert(
                "oauth2".into(),
                serde_json::json!([
                    {"key":"grant_type","value":"client_credentials"},
                    {"key":"accessTokenUrl","value": scheme.get("tokenUrl").and_then(|x|x.as_str()).unwrap_or("")},
                    {"key":"scope","value": scheme.get("scopes").and_then(|s| s.as_object()).map(|m| m.keys().cloned().collect::<Vec<_>>().join(" ")).unwrap_or_default()}
                ]),
            );
            Some(Auth {
                kind: "oauth2".into(),
                extra,
                ..Default::default()
            })
        }
        _ => None,
    }
}

pub fn import_openapi(v: &Value) -> Collection {
    let title = v
        .get("info")
        .and_then(|i| i.get("title"))
        .and_then(|t| t.as_str())
        .unwrap_or("OpenAPI Import")
        .to_string();
    let mut c = Collection {
        info: new_info(&title),
        ..Default::default()
    };
    let base = openapi_base(v);

    let mut items = Vec::new();
    if let Some(paths) = v.get("paths").and_then(|p| p.as_object()) {
        for (path, path_item) in paths {
            if let Some(obj) = path_item.as_object() {
                for (method, op) in obj {
                    let m = method.to_ascii_uppercase();
                    if !["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"].contains(&m.as_str())
                    {
                        continue;
                    }
                    let summary = op
                        .get("summary")
                        .and_then(|s| s.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| format!("{} {}", m, path));
                    let mut full_path = path.clone();
                    let mut query_pairs: Vec<(String, String)> = Vec::new();
                    let mut headers = Vec::new();
                    let mut body = None;
                    let mut all_params: Vec<&Value> = Vec::new();
                    if let Some(ps) = path_item.get("parameters").and_then(|p| p.as_array()) {
                        all_params.extend(ps.iter());
                    }
                    if let Some(ps) = op.get("parameters").and_then(|p| p.as_array()) {
                        all_params.extend(ps.iter());
                    }
                    for p in all_params {
                        let loc = p.get("in").and_then(|x| x.as_str()).unwrap_or("");
                        let name = p
                            .get("name")
                            .and_then(|x| x.as_str())
                            .unwrap_or("")
                            .to_string();
                        let val = p
                            .get("example")
                            .or_else(|| p.get("schema").and_then(|s| s.get("default")))
                            .or_else(|| p.get("schema").and_then(|s| s.get("example")))
                            .map(|x| {
                                x.as_str()
                                    .map(|s| s.to_string())
                                    .unwrap_or_else(|| x.to_string())
                            })
                            .unwrap_or_default();
                        match loc {
                            "header" => headers.push(header(&name, &val)),
                            "query" => {
                                if !name.is_empty() {
                                    query_pairs.push((name, val));
                                }
                            }
                            "path" => {
                                full_path = full_path.replace(&format!("{{{}}}", name), &val);
                            }
                            // Swagger 2 body parameter
                            "body" => {
                                if let Some(schema) = p.get("schema") {
                                    let example = schema
                                        .get("example")
                                        .or_else(|| schema.get("default"))
                                        .cloned();
                                    let content = example
                                        .map(|e| serde_json::to_string_pretty(&e).unwrap_or_default())
                                        .unwrap_or_else(|| "{}".to_string());
                                    let ct = op
                                        .get("consumes")
                                        .and_then(|c| c.as_array())
                                        .and_then(|a| a.get(0))
                                        .and_then(|x| x.as_str())
                                        .unwrap_or("application/json");
                                    headers.push(header("Content-Type", ct));
                                    body = Some(Body {
                                        mode: "raw".into(),
                                        raw: Some(content),
                                        options: Some(
                                            serde_json::json!({"raw":{"language":"json"}}),
                                        ),
                                        ..Default::default()
                                    });
                                }
                            }
                            _ => {}
                        }
                    }
                    let mut full_url = format!("{}{}", base.trim_end_matches('/'), full_path);
                    if !query_pairs.is_empty() {
                        let mut sep = if full_url.contains('?') { '&' } else { '?' };
                        for (k, v) in &query_pairs {
                            full_url.push(sep);
                            sep = '&';
                            full_url.push_str(&format!("{}={}", k, v));
                        }
                    }
                    if let Some(rb) = op.get("requestBody") {
                        if let Some(content) = rb.get("content").and_then(|c| c.as_object()) {
                            for (ct, media) in content {
                                let example = media
                                    .get("example")
                                    .or_else(|| {
                                        media
                                            .get("schema")
                                            .and_then(|s| s.get("example"))
                                    })
                                    .map(|e| serde_json::to_string_pretty(e).unwrap_or_default());
                                if let Some(ex) = example {
                                    headers.push(header("Content-Type", ct));
                                    body = Some(Body {
                                        mode: "raw".into(),
                                        raw: Some(ex),
                                        options: Some(serde_json::json!({"raw":{"language":"json"}})),
                                        ..Default::default()
                                    });
                                    break;
                                }
                            }
                        }
                    }
                    items.push(Item {
                        name: summary,
                        request: Some(Request {
                            method: m,
                            url: UrlSpec::new(full_url),
                            header: headers,
                            body,
                            auth: openapi_security_auth(v, op),
                            ..Default::default()
                        }),
                        ..Default::default()
                    });
                }
            }
        }
    }
    c.item = items;
    c
}

/// Tokenize a shell-like string honoring single/double quotes.
fn shell_tokens(s: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match quote {
            Some(q) => {
                if c == q {
                    quote = None;
                } else if c == '\\' && q == '"' {
                    if let Some(n) = chars.next() {
                        cur.push(n);
                    }
                } else {
                    cur.push(c);
                }
            }
            None => match c {
                '\'' | '"' => quote = Some(c),
                '\\' => {
                    if let Some(n) = chars.next() {
                        cur.push(n);
                    }
                }
                c if c.is_whitespace() => {
                    if !cur.is_empty() {
                        tokens.push(std::mem::take(&mut cur));
                    }
                }
                _ => cur.push(c),
            },
        }
    }
    if !cur.is_empty() {
        tokens.push(cur);
    }
    tokens
}

pub fn import_curl(cmd: &str) -> Collection {
    let toks = shell_tokens(cmd);
    let mut method = String::new();
    let mut explicit_url = String::new();
    let mut bare_url = String::new();
    let mut headers: Vec<Header> = Vec::new();
    let mut raw_parts: Vec<String> = Vec::new();
    let mut urlencoded: Vec<KV> = Vec::new();
    let mut formdata: Vec<KV> = Vec::new();
    let mut upload_file: Option<String> = None;
    let mut basic: Option<(String, String)> = None;
    let mut cookies: Vec<String> = Vec::new();
    let mut get_mode = false;
    let mut json_mode = false;

    let mut i = 0;
    // 读取某个需要参数的开关的取值；自动跳过后面的 token。
    while i < toks.len() {
        let t = toks[i].as_str();
        // 兼容 `--flag=value` 形式
        let (flag, inline_val) = match t.split_once('=') {
            Some((f, v)) if f.starts_with("--") => (f, Some(v.to_string())),
            _ => (t, None),
        };
        let take_val = |i: &mut usize| -> Option<String> {
            if let Some(v) = &inline_val {
                return Some(v.clone());
            }
            if *i + 1 < toks.len() {
                *i += 1;
                Some(toks[*i].clone())
            } else {
                None
            }
        };
        match flag {
            "-X" | "--request" => {
                if let Some(v) = take_val(&mut i) {
                    method = v.to_uppercase();
                }
            }
            "-H" | "--header" => {
                if let Some(v) = take_val(&mut i) {
                    if let Some((k, val)) = v.split_once(':') {
                        headers.push(header(k.trim(), val.trim()));
                    }
                }
            }
            "-d" | "--data" | "--data-raw" | "--data-binary" | "--data-ascii" => {
                if let Some(v) = take_val(&mut i) {
                    // @file 形式：读文件内容（若存在），否则原样保留
                    if let Some(path) = v.strip_prefix('@') {
                        if let Ok(content) = std::fs::read_to_string(path) {
                            raw_parts.push(content);
                        } else {
                            raw_parts.push(v);
                        }
                    } else {
                        raw_parts.push(v);
                    }
                }
            }
            "--data-urlencode" => {
                if let Some(v) = take_val(&mut i) {
                    match v.split_once('=') {
                        Some((k, val)) => urlencoded.push(KV {
                            key: k.to_string(),
                            value: val.to_string(),
                            ..Default::default()
                        }),
                        None => urlencoded.push(KV {
                            key: v.clone(),
                            value: String::new(),
                            ..Default::default()
                        }),
                    }
                }
            }
            "-F" | "--form" => {
                if let Some(v) = take_val(&mut i) {
                    let (k, val) = v.split_once('=').unwrap_or((v.as_str(), ""));
                    let key = k.split(';').next().unwrap_or(k).trim().to_string();
                    if let Some(src) = val.strip_prefix('@') {
                        let path = src.split(';').next().unwrap_or(src).trim();
                        formdata.push(KV {
                            key,
                            typ: Some("file".into()),
                            src: Some(serde_json::json!(path)),
                            ..Default::default()
                        });
                    } else {
                        formdata.push(KV {
                            key,
                            value: val.to_string(),
                            ..Default::default()
                        });
                    }
                }
            }
            "-u" | "--user" => {
                if let Some(v) = take_val(&mut i) {
                    if let Some((u, p)) = v.split_once(':') {
                        basic = Some((u.to_string(), p.to_string()));
                    }
                }
            }
            "-b" | "--cookie" => {
                if let Some(v) = take_val(&mut i) {
                    cookies.push(v);
                }
            }
            "-A" | "--user-agent" => {
                if let Some(v) = take_val(&mut i) {
                    headers.push(header("User-Agent", &v));
                }
            }
            "-e" | "--referer" => {
                if let Some(v) = take_val(&mut i) {
                    headers.push(header("Referer", &v));
                }
            }
            "-T" | "--upload-file" => {
                if let Some(v) = take_val(&mut i) {
                    upload_file = Some(v);
                }
            }
            "--url" => {
                if let Some(v) = take_val(&mut i) {
                    explicit_url = v;
                }
            }
            "-G" | "--get" => get_mode = true,
            "--json" => {
                if let Some(v) = take_val(&mut i) {
                    raw_parts.push(v);
                }
                json_mode = true;
            }
            // 以下开关无参数或对本模型无意义，直接忽略
            "-k" | "--insecure" | "--compressed" | "-L" | "--location" | "-s" | "--silent"
            | "-i" | "--include" | "-v" | "--verbose" | "--globoff" | "-g" => {}
            "-o" | "--output" | "-x" | "--proxy" | "--cacert" | "--cert" | "--key" => {
                let _ = take_val(&mut i);
            }
            "curl" | "curl.exe" => {}
            _ => {
                if !flag.starts_with('-') && bare_url.is_empty() {
                    bare_url = toks[i].clone();
                }
            }
        }
        i += 1;
    }

    let mut url = if !explicit_url.is_empty() {
        explicit_url
    } else {
        bare_url
    };

    // 方法推断
    if method.is_empty() {
        method = if get_mode {
            "GET".into()
        } else if !formdata.is_empty()
            || !raw_parts.is_empty()
            || !urlencoded.is_empty()
            || upload_file.is_some()
        {
            "POST".into()
        } else {
            "GET".into()
        };
    }
    if get_mode {
        method = "GET".into();
    }

    // -G：把数据拼到查询串
    if get_mode {
        let mut qs = String::new();
        let mut push_q = |pair: String| {
            if !qs.is_empty() {
                qs.push('&');
            }
            qs.push_str(&pair);
        };
        for p in &raw_parts {
            push_q(p.clone());
        }
        for kv in &urlencoded {
            push_q(format!("{}={}", kv.key, kv.value));
        }
        if !qs.is_empty() {
            url.push(if url.contains('?') { '&' } else { '?' });
            url.push_str(&qs);
        }
        raw_parts.clear();
        urlencoded.clear();
    }

    // Cookie 头
    if !cookies.is_empty() {
        headers.push(header("Cookie", &cookies.join("; ")));
    }
    if json_mode && !raw_parts.is_empty() {
        headers.push(header("Content-Type", "application/json"));
    }

    // 请求体
    let body = if !formdata.is_empty() {
        Some(Body {
            mode: "formdata".into(),
            formdata,
            ..Default::default()
        })
    } else if !urlencoded.is_empty() {
        Some(Body {
            mode: "urlencoded".into(),
            urlencoded,
            ..Default::default()
        })
    } else if let Some(path) = upload_file {
        Some(Body {
            mode: "file".into(),
            file: Some(FileRef {
                src: Some(path),
                content: None,
            }),
            ..Default::default()
        })
    } else if !raw_parts.is_empty() {
        let raw = raw_parts.join("&");
        let trimmed = raw.trim_start();
        let lang = if json_mode || trimmed.starts_with('{') || trimmed.starts_with('[') {
            "json"
        } else {
            "text"
        };
        Some(Body {
            mode: "raw".into(),
            raw: Some(raw),
            options: Some(serde_json::json!({"raw":{"language":lang}})),
            ..Default::default()
        })
    } else {
        None
    };

    let auth = basic.map(|(u, p)| Auth {
        kind: "basic".into(),
        basic: vec![
            KV { key: "username".into(), value: u, ..Default::default() },
            KV { key: "password".into(), value: p, ..Default::default() },
        ],
        ..Default::default()
    });

    let req = Request {
        method,
        url: UrlSpec::new(url),
        header: headers,
        body,
        auth,
        ..Default::default()
    };
    Collection {
        info: new_info("cURL Import"),
        item: vec![Item {
            name: "cURL request".into(),
            request: Some(req),
            ..Default::default()
        }],
        ..Default::default()
    }
}

pub fn import_har(v: &Value) -> Result<Collection> {
    let entries = v
        .get("log")
        .and_then(|l| l.get("entries"))
        .and_then(|e| e.as_array())
        .ok_or_else(|| anyhow!("HAR missing log.entries"))?;
    let mut items = Vec::new();
    for entry in entries {
        let req = entry.get("request");
        let url = req
            .and_then(|r| r.get("url"))
            .and_then(|u| u.as_str())
            .unwrap_or("")
            .to_string();
        let method = req
            .and_then(|r| r.get("method"))
            .and_then(|m| m.as_str())
            .unwrap_or("GET")
            .to_uppercase();
        let mut headers = Vec::new();
        if let Some(hs) = req.and_then(|r| r.get("headers")).and_then(|h| h.as_array()) {
            for h in hs {
                let k = h.get("name").and_then(|x| x.as_str()).unwrap_or("");
                let val = h.get("value").and_then(|x| x.as_str()).unwrap_or("");
                headers.push(header(k, val));
            }
        }
        let body = req
            .and_then(|r| r.get("postData"))
            .and_then(|p| p.get("text"))
            .and_then(|t| t.as_str())
            .map(|t| Body {
                mode: "raw".into(),
                raw: Some(t.to_string()),
                options: Some(serde_json::json!({"raw":{"language":"json"}})),
                ..Default::default()
            });
        items.push(Item {
            name: format!("{} {}", method, url),
            request: Some(Request {
                method,
                url: UrlSpec::new(url),
                header: headers,
                body,
                ..Default::default()
            }),
            ..Default::default()
        });
    }
    Ok(Collection {
        info: new_info("HAR Import"),
        item: items,
        ..Default::default()
    })
}

/// Merge `incoming` into `existing` non-destructively: variables by key, items by name
/// (duplicate item names get a " (n)" suffix instead of replacing existing requests).
pub fn merge_collections(existing: &mut Collection, incoming: Collection) {
    for v in incoming.variable {
        if let Some(slot) = existing.variable.iter_mut().find(|x| x.key == v.key) {
            *slot = v;
        } else {
            existing.variable.push(v);
        }
    }
    for mut item in incoming.item {
        if existing.item.iter().any(|x| x.name == item.name) {
            let base = item.name.clone();
            let mut n = 1;
            while existing.item.iter().any(|x| x.name == item.name) {
                item.name = format!("{} ({})", base, n);
                n += 1;
            }
        }
        existing.item.push(item);
    }
}

pub fn export_collection(c: &mut Collection) -> Result<String> {
    if c.info.postman_id.is_none() {
        c.info.postman_id = Some(uuid::Uuid::new_v4().to_string());
    }
    if c.info.schema.is_none() {
        c.info.schema =
            Some("https://schema.getpostman.com/json/collection/v2.1.0/collection.json".into());
    }
    Ok(serde_json::to_string_pretty(c)?)
}

/// Export an environment (or globals) as Postman-compatible JSON.
pub fn export_environment(e: &mut Environment) -> Result<String> {
    if e.scope.is_none() {
        e.scope = Some("environment".to_string());
    }
    Ok(serde_json::to_string_pretty(e)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn postman_roundtrip() {
        let json = r#"{
          "info": {"name":"Demo","_postman_id":"11111111-1111-1111-1111-111111111111",
                   "schema":"https://schema.getpostman.com/json/collection/v2.1.0/collection.json"},
          "item": [{"name":"r1","request":{"method":"GET","url":{"raw":"https://api.test/x"},
                    "header":[{"key":"A","value":"b"}]}}]
        }"#;
        let Imported::Collection(c) = import_postman_text(json, "demo.json").unwrap() else {
            panic!("expected collection");
        };
        assert_eq!(c.info.name, "Demo");
        assert_eq!(c.item.len(), 1);
        let mut c2 = *c;
        let out = export_collection(&mut c2).unwrap();
        let v: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["item"][0]["request"]["url"]["raw"], "https://api.test/x");
    }

    #[test]
    fn curl_import() {
        let cmd = r#"curl 'https://api.test/users?x=1' -X POST -H 'Content-Type: application/json' -d '{"a":1}' -u user:pass"#;
        let c = import_curl(cmd);
        let r = c.item[0].request.as_ref().unwrap();
        assert_eq!(r.method, "POST");
        assert_eq!(r.url.raw, "https://api.test/users?x=1");
        assert!(r.body.is_some());
        assert_eq!(r.auth.as_ref().unwrap().kind, "basic");
    }

    #[test]
    fn curl_import_form_cookie_agent_and_upload() {
        let cmd = r#"curl -X POST 'https://api.test/upload' -b 'sid=abc; theme=dark' -A 'pmrs/1.0' -F 'name=z' -F 'avatar=@C:\tmp\a.png;type=image/png'"#;
        let r = import_curl(cmd).item[0].request.clone().unwrap();
        assert_eq!(r.method, "POST");
        let b = r.body.as_ref().unwrap();
        assert_eq!(b.mode, "formdata");
        assert_eq!(b.formdata.len(), 2);
        assert_eq!(b.formdata[0].key, "name");
        assert_eq!(b.formdata[0].value, "z");
        assert_eq!(b.formdata[1].key, "avatar");
        assert_eq!(b.formdata[1].typ.as_deref(), Some("file"));
        assert_eq!(
            b.formdata[1].src.as_ref().and_then(|v| v.as_str()),
            Some("C:\\tmp\\a.png")
        );
        assert!(r
            .header
            .iter()
            .any(|h| h.key.eq_ignore_ascii_case("User-Agent") && h.value == "pmrs/1.0"));
        assert!(r
            .header
            .iter()
            .any(|h| h.key.eq_ignore_ascii_case("Cookie") && h.value == "sid=abc; theme=dark"));
    }

    #[test]
    fn curl_import_get_data_urlencode_and_multiple_data() {
        let cmd = r#"curl 'https://api.test/search' -G -d 'q=rust' -d 'page=2'"#;
        let r = import_curl(cmd).item[0].request.clone().unwrap();
        assert_eq!(r.method, "GET");
        assert_eq!(r.url.raw, "https://api.test/search?q=rust&page=2");
        assert!(r.body.is_none());

        let cmd2 =
            r#"curl 'https://api.test/form' --data-urlencode 'a=1' --data-urlencode 'b=hi there'"#;
        let r2 = import_curl(cmd2).item[0].request.clone().unwrap();
        assert_eq!(r2.method, "POST");
        let b = r2.body.as_ref().unwrap();
        assert_eq!(b.mode, "urlencoded");
        assert_eq!(b.urlencoded.len(), 2);
        assert_eq!(b.urlencoded[1].value, "hi there");
    }

    #[test]
    fn curl_import_upload_file_and_json() {
        let cmd = r#"curl -T 'C:\tmp\data.bin' 'https://api.test/upload'"#;
        let r = import_curl(cmd).item[0].request.clone().unwrap();
        let b = r.body.as_ref().unwrap();
        assert_eq!(b.mode, "file");
        assert_eq!(
            b.file.as_ref().and_then(|f| f.src.as_deref()),
            Some("C:\\tmp\\data.bin")
        );

        let cmd2 = r#"curl --json '{"x":1}' https://api.test/j"#;
        let r2 = import_curl(cmd2).item[0].request.clone().unwrap();
        assert_eq!(r2.method, "POST");
        assert!(r2
            .header
            .iter()
            .any(|h| h.key.eq_ignore_ascii_case("content-type") && h.value.contains("json")));
        assert_eq!(r2.body.as_ref().unwrap().mode, "raw");
    }

    #[test]
    fn openapi_import() {
        let v: Value = serde_json::json!({
          "info":{"title":"T","version":"1"},
          "servers":[{"url":"https://api.test"}],
          "paths":{"/ping":{"get":{"summary":"Ping"}}}
        });
        let c = import_openapi(&v);
        assert_eq!(c.item.len(), 1);
        assert_eq!(c.item[0].request.as_ref().unwrap().method, "GET");
    }

    #[test]
    fn swagger2_import_base_host_and_apikey() {
        let v: Value = serde_json::json!({
          "swagger":"2.0",
          "info":{"title":"S2","version":"1"},
          "host":"api.test",
          "basePath":"/v1",
          "schemes":["https"],
          "securityDefinitions":{"k":{"type":"apiKey","name":"X-Key","in":"header"}},
          "security":[{"k":[]}],
          "paths":{"/items/{id}":{"get":{
             "summary":"Get",
             "parameters":[{"in":"path","name":"id","required":true,"schema":{"default":"9"}}]
          }}}
        });
        let c = import_openapi(&v);
        let r = c.item[0].request.as_ref().unwrap();
        assert_eq!(r.url.raw, "https://api.test/v1/items/9");
        let auth = r.auth.as_ref().unwrap();
        assert_eq!(auth.kind, "apikey");
        assert_eq!(auth.apikey_key(), Some("X-Key"));
    }

    #[test]
    fn postman_v1_import() {
        let json = r#"{
          "name":"V1 Demo",
          "requests":[{
            "name":"Login","method":"POST","url":"https://api.test/login",
            "headers":"Content-Type: application/json\nX-Test: 1",
            "dataMode":"raw","rawModeData":"{\"u\":1}"
          }],
          "folders":[{"name":"F1","order":[],"requests":[{"name":"Ping","method":"GET","url":"https://api.test/ping","headers":""}]}]
        }"#;
        let Imported::Collection(c) = import_postman_text(json, "v1.json").unwrap() else {
            panic!("expected collection");
        };
        assert_eq!(c.info.name, "V1 Demo");
        assert_eq!(c.item.len(), 2); // folder + request
        let folder = &c.item[0];
        assert_eq!(folder.name, "F1");
        assert_eq!(folder.item.as_ref().unwrap()[0].name, "Ping");
        let login = &c.item[1];
        let r = login.request.as_ref().unwrap();
        assert_eq!(r.method, "POST");
        assert_eq!(r.header.len(), 2);
        assert_eq!(r.body.as_ref().unwrap().mode, "raw");
    }

    #[test]
    fn globals_import() {
        let json = r#"{"name":"Globals","values":[{"key":"g","value":"1","type":"default"}],"_postman_variable_scope":"globals"}"#;
        let Imported::Environment(e) = import_postman_text(json, "g.json").unwrap() else {
            panic!("expected environment/globals");
        };
        assert_eq!(e.scope.as_deref(), Some("globals"));
        assert_eq!(e.values[0].key, "g");
    }
}
