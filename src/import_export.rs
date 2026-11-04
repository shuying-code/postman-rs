//! Import from Postman/OpenAPI/cURL/HAR and export to Postman Collection v2.1.
use crate::model::*; // 引入依赖
use anyhow::{anyhow, Result}; // 引入依赖
use serde_json::Value; // 引入依赖

pub enum Imported { // 枚举 Imported
    Collection(Box<Collection>), // 调用 Collection
    Environment(Box<Environment>), // 调用 Environment
} // 块结束

fn new_info(name: &str) -> CollectionInfo { // 函数 new_info
    CollectionInfo { // 语句
        name: name.to_string(), // 字段 name
        postman_id: Some(uuid::Uuid::new_v4().to_string()), // 字段 postman_id
        schema: Some("https://schema.getpostman.com/json/collection/v2.1.0/collection.json".into()), // 字段 schema
        description: None, // 字段 description
    } // 块结束
} // 块结束

fn header(k: &str, v: &str) -> Header { // 函数 header
    Header { // 语句
        key: k.to_string(), // 字段 key
        value: v.to_string(), // 字段 value
        ..Default::default() // 语句
    } // 块结束
} // 块结束

pub fn import_file(path: &str, format: &str) -> Result<Imported> { // 函数 import_file
    let text = std::fs::read_to_string(path)?; // 变量 text
    match format.to_ascii_lowercase().as_str() { // 分支匹配
        "postman" | "auto" => import_postman_text(&text, path), // 匹配分支
        "openapi" | "swagger" => { // 匹配分支
            let v: Value = match serde_json::from_str(&text) { // 变量 v
                Ok(v) => v, // 匹配分支
                Err(_) => serde_yaml::from_str::<Value>(&text) // 匹配分支
                    .map_err(|e| anyhow!("parse openapi yaml: {}", e))?, // 语句
            }; // 块结束
            Ok(Imported::Collection(Box::new(import_openapi(&v)))) // 调用 Ok
        } // 块结束
        "curl" => Ok(Imported::Collection(Box::new(import_curl(&text)))), // 匹配分支
        "har" => { // 匹配分支
            let v: Value = serde_json::from_str(&text)?; // 变量 v
            Ok(Imported::Collection(Box::new(import_har(&v)?))) // 调用 Ok
        } // 块结束
        other => Err(anyhow!("unsupported import format: {}", other)), // 匹配分支
    } // 块结束
} // 块结束

pub fn import_postman_text(text: &str, path: &str) -> Result<Imported> { // 函数 import_postman_text
    let v: Value = serde_json::from_str(text)?; // 变量 v
    // Postman Collection v1 (has requests/folders/order at top level, no `info`).
    if v.get("requests").is_some() || v.get("folders").is_some() { // 条件分支
        let c = import_postman_v1(&v, path); // 变量 c
        return Ok(Imported::Collection(Box::new(c))); // 返回
    } // 块结束
    if v.get("info").is_some() { // 条件分支
        let mut c: Collection = serde_json::from_value(v)?; // 变量 c
        if c.info.name.is_empty() { // 条件分支
            c.info.name = std::path::Path::new(path) // 语句
                .file_stem() // 语句
                .map(|s| s.to_string_lossy().to_string()) // 语句
                .unwrap_or_else(|| "imported".into()); // 语句
        } // 块结束
        Ok(Imported::Collection(Box::new(c))) // 调用 Ok
    } else if v.get("values").is_some() || v.get("globals").is_some() { // 条件分支
        let e = parse_env_or_globals(&v); // 变量 e
        Ok(Imported::Environment(Box::new(e))) // 调用 Ok
    } else { // 语句
        Err(anyhow!("unrecognized Postman file (neither collection nor environment)")) // 调用 Err
    } // 块结束
} // 块结束

/// Parse an environment OR a globals export into `Environment`
/// (globals keep `_postman_variable_scope = "globals"`).
fn parse_env_or_globals(v: &Value) -> Environment { // 函数 parse_env_or_globals
    let name = v // 变量 name
        .get("name") // 语句
        .and_then(|n| n.as_str()) // 语句
        .unwrap_or("Globals") // 语句
        .to_string(); // 语句
    let list = v.get("values").or_else(|| v.get("globals")); // 变量 list
    let values: Vec<Variable> = list // 变量 values
        .and_then(|x| x.as_array()) // 语句
        .map(|a| { // 语句
            a.iter() // 语句
                .filter_map(|e| serde_json::from_value::<Variable>(e.clone()).ok()) // 语句
                .collect() // 语句
        }) // 语句
        .unwrap_or_default(); // 语句
    let scope = v // 变量 scope
        .get("_postman_variable_scope") // 语句
        .and_then(|s| s.as_str()) // 语句
        .map(|s| s.to_string()) // 语句
        .or_else(|| { // 语句
            if v.get("globals").is_some() { // 条件分支
                Some("globals".to_string()) // 调用 Some
            } else { // 语句
                None // 语句
            } // 块结束
        }); // 语句
    Environment { // 语句
        id: v.get("id").and_then(|x| x.as_str()).map(|s| s.to_string()), // 字段 id
        name, // 语句
        values, // 语句
        scope, // 语句
    } // 块结束
} // 块结束

fn parse_header_lines(s: &str) -> Vec<Header> { // 函数 parse_header_lines
    s.lines() // 语句
        .filter_map(|l| l.split_once(':').map(|(k, v)| header(k.trim(), v.trim()))) // 语句
        .collect() // 语句
} // 块结束

fn v1_kv_list(v: Option<&Value>) -> Vec<KV> { // 函数 v1_kv_list
    v.and_then(|x| x.as_array()) // 语句
        .map(|a| { // 语句
            a.iter() // 语句
                .map(|e| KV { // 语句
                    key: e.get("key").and_then(|k| k.as_str()).unwrap_or("").to_string(), // 字段 key
                    value: e // 语句
                        .get("value") // 语句
                        .and_then(|vv| vv.as_str()) // 语句
                        .map(|s| s.to_string()) // 语句
                        .unwrap_or_default(), // 语句
                    disabled: !e.get("enabled").and_then(|x| x.as_bool()).unwrap_or(true), // 字段 disabled
                    ..Default::default() // 语句
                }) // 语句
                .collect() // 语句
        }) // 语句
        .unwrap_or_default() // 语句
} // 块结束

fn v1_request_to_item(r: &Value) -> Item { // 函数 v1_request_to_item
    let name = r // 变量 name
        .get("name") // 语句
        .and_then(|n| n.as_str()) // 语句
        .unwrap_or("request") // 语句
        .to_string(); // 语句
    let method = r // 变量 method
        .get("method") // 语句
        .and_then(|m| m.as_str()) // 语句
        .unwrap_or("GET") // 语句
        .to_string(); // 语句
    let url = r // 变量 url
        .get("url") // 语句
        .and_then(|u| u.as_str().map(|s| s.to_string())) // 语句
        .or_else(|| { // 语句
            r.get("url") // 语句
                .and_then(|u| u.get("raw")) // 语句
                .and_then(|x| x.as_str()) // 语句
                .map(|s| s.to_string()) // 语句
        }) // 语句
        .unwrap_or_default(); // 语句
    let mut headers = Vec::new(); // 变量 headers
    if let Some(hs) = r.get("headers").and_then(|h| h.as_str()) { // 条件分支
        headers = parse_header_lines(hs); // 语句
    } else if let Some(hs) = r.get("headerData").and_then(|h| h.as_array()) { // 条件分支
        for h in hs { // 循环
            if h.get("enabled").and_then(|e| e.as_bool()).unwrap_or(true) { // 条件分支
                headers.push(header( // 语句
                    h.get("key").and_then(|k| k.as_str()).unwrap_or(""), // 语句
                    h.get("value").and_then(|v| v.as_str()).unwrap_or(""), // 语句
                )); // 语句
            } // 块结束
        } // 块结束
    } // 块结束
    let dm = r.get("dataMode").and_then(|d| d.as_str()).unwrap_or(""); // 变量 dm
    let body = match dm { // 变量 body
        "raw" => r // 匹配分支
            .get("rawModeData") // 语句
            .and_then(|d| d.as_str()) // 语句
            .map(|s| Body { // 语句
                mode: "raw".into(), // 字段 mode
                raw: Some(s.to_string()), // 字段 raw
                options: Some(serde_json::json!({"raw":{"language":"json"}})), // 字段 options
                ..Default::default() // 语句
            }), // 语句
        "urlencoded" => Some(Body { // 匹配分支
            mode: "urlencoded".into(), // 字段 mode
            urlencoded: v1_kv_list(r.get("data")), // 字段 urlencoded
            ..Default::default() // 语句
        }), // 语句
        "params" => Some(Body { // 匹配分支
            mode: "formdata".into(), // 字段 mode
            formdata: v1_kv_list(r.get("data")), // 字段 formdata
            ..Default::default() // 语句
        }), // 语句
        _ => None, // 匹配分支
    }; // 块结束
    Item { // 语句
        name, // 语句
        request: Some(Request { // 语句
            method, // 语句
            url: UrlSpec::new(url), // 字段 url
            header: headers, // 字段 header
            body, // 语句
            ..Default::default() // 语句
        }), // 语句
        ..Default::default() // 语句
    } // 块结束
} // 块结束

fn v1_items(container: &Value) -> Vec<Item> { // 函数 v1_items
    let mut items = Vec::new(); // 变量 items
    if let Some(folders) = container.get("folders").and_then(|f| f.as_array()) { // 条件分支
        for f in folders { // 循环
            let name = f // 变量 name
                .get("name") // 语句
                .and_then(|n| n.as_str()) // 语句
                .unwrap_or("folder") // 语句
                .to_string(); // 语句
            items.push(Item { // 语句
                name, // 语句
                item: Some(v1_items(f)), // 字段 item
                ..Default::default() // 语句
            }); // 语句
        } // 块结束
    } // 块结束
    if let Some(reqs) = container.get("requests").and_then(|r| r.as_array()) { // 条件分支
        for r in reqs { // 循环
            items.push(v1_request_to_item(r)); // 语句
        } // 块结束
    } // 块结束
    items // 语句
} // 块结束

/// Import a Postman Collection v1 document into our v2.1-shaped model.
pub fn import_postman_v1(v: &Value, path: &str) -> Collection { // 函数 import_postman_v1
    let name = v // 变量 name
        .get("name") // 语句
        .and_then(|n| n.as_str()) // 语句
        .map(|s| s.to_string()) // 语句
        .unwrap_or_else(|| { // 语句
            std::path::Path::new(path) // 调用 new
                .file_stem() // 语句
                .map(|s| s.to_string_lossy().to_string()) // 语句
                .unwrap_or_else(|| "Imported v1".into()) // 语句
        }); // 语句
    Collection { // 语句
        info: new_info(&name), // 字段 info
        item: v1_items(v), // 字段 item
        ..Default::default() // 语句
    } // 块结束
} // 块结束

/// Base URL for OpenAPI 3 (`servers[0].url`) or Swagger 2 (`schemes://host + basePath`).
fn openapi_base(v: &Value) -> String { // 函数 openapi_base
    if let Some(url) = v // 条件分支
        .get("servers") // 语句
        .and_then(|s| s.get(0)) // 语句
        .and_then(|s| s.get("url")) // 语句
        .and_then(|u| u.as_str()) // 语句
    { // 语句
        return url.trim_end_matches('/').to_string(); // 返回
    } // 块结束
    let host = v.get("host").and_then(|h| h.as_str()).unwrap_or(""); // 变量 host
    let base_path = v.get("basePath").and_then(|b| b.as_str()).unwrap_or(""); // 变量 base_path
    let scheme = v // 变量 scheme
        .get("schemes") // 语句
        .and_then(|s| s.as_array()) // 语句
        .and_then(|a| a.get(0)) // 语句
        .and_then(|s| s.as_str()) // 语句
        .unwrap_or("https"); // 语句
    if !host.is_empty() { // 条件分支
        format!("{}://{}{}", scheme, host, base_path.trim_end_matches('/')) // 语句
    } else { // 语句
        base_path.to_string() // 语句
    } // 块结束
} // 块结束

/// Build an `Auth` from OpenAPI/Swagger `security` + `securitySchemes`/`securityDefinitions`.
fn openapi_security_auth(v: &Value, op: &Value) -> Option<Auth> { // 函数 openapi_security_auth
    let sec = op.get("security").or_else(|| v.get("security"))?; // 变量 sec
    let first = sec.as_array()?.get(0)?.as_object()?; // 变量 first
    let (name, _) = first.iter().next()?; // 调用 let
    let schemes = v // 变量 schemes
        .get("components") // 语句
        .and_then(|c| c.get("securitySchemes")) // 语句
        .or_else(|| v.get("securityDefinitions"))?; // 语句
    let scheme = schemes.get(name)?; // 变量 scheme
    let typ = scheme.get("type").and_then(|t| t.as_str()).unwrap_or(""); // 变量 typ
    let kv = |k: &str, val: String| KV { // 变量 kv
        key: k.to_string(), // 字段 key
        value: val, // 字段 value
        ..Default::default() // 语句
    }; // 块结束
    match typ { // 分支匹配
        "http" => { // 匹配分支
            let s = scheme // 变量 s
                .get("scheme") // 语句
                .and_then(|x| x.as_str()) // 语句
                .unwrap_or(""); // 语句
            if s.eq_ignore_ascii_case("bearer") { // 条件分支
                Some(Auth { // 调用 Some
                    kind: "bearer".into(), // 字段 kind
                    bearer: vec![kv("token", String::new())], // 字段 bearer
                    ..Default::default() // 语句
                }) // 语句
            } else if s.eq_ignore_ascii_case("basic") { // 条件分支
                Some(Auth { // 调用 Some
                    kind: "basic".into(), // 字段 kind
                    basic: vec![kv("username", String::new()), kv("password", String::new())], // 字段 basic
                    ..Default::default() // 语句
                }) // 语句
            } else { // 语句
                None // 语句
            } // 块结束
        } // 块结束
        "basic" => Some(Auth { // 匹配分支
            kind: "basic".into(), // 字段 kind
            basic: vec![kv("username", String::new()), kv("password", String::new())], // 字段 basic
            ..Default::default() // 语句
        }), // 语句
        "apiKey" => { // 匹配分支
            let key = scheme.get("name").and_then(|x| x.as_str()).unwrap_or(""); // 变量 key
            let loc = scheme.get("in").and_then(|x| x.as_str()).unwrap_or("header"); // 变量 loc
            Some(Auth { // 调用 Some
                kind: "apikey".into(), // 字段 kind
                apikey: vec![ // 语句
                    kv("key", key.to_string()), // 调用 kv
                    kv("value", String::new()), // 调用 kv
                    kv("in", loc.to_string()), // 调用 kv
                ], // 语句
                ..Default::default() // 语句
            }) // 语句
        } // 块结束
        "oauth2" => { // 匹配分支
            let mut extra = serde_json::Map::new(); // 变量 extra
            extra.insert( // 语句
                "oauth2".into(), // 语句
                serde_json::json!([ // 语句
                    {"key":"grant_type","value":"client_credentials"}, // 语句
                    {"key":"accessTokenUrl","value": scheme.get("tokenUrl").and_then(|x|x.as_str()).unwrap_or("")}, // 语句
                    {"key":"scope","value": scheme.get("scopes").and_then(|s| s.as_object()).map(|m| m.keys().cloned().collect::<Vec<_>>().join(" ")).unwrap_or_default()} // 语句
                ]), // 语句
            ); // 块结束
            Some(Auth { // 调用 Some
                kind: "oauth2".into(), // 字段 kind
                extra, // 语句
                ..Default::default() // 语句
            }) // 语句
        } // 块结束
        _ => None, // 匹配分支
    } // 块结束
} // 块结束

pub fn import_openapi(v: &Value) -> Collection { // 函数 import_openapi
    let title = v // 变量 title
        .get("info") // 语句
        .and_then(|i| i.get("title")) // 语句
        .and_then(|t| t.as_str()) // 语句
        .unwrap_or("OpenAPI Import") // 语句
        .to_string(); // 语句
    let mut c = Collection { // 变量 c
        info: new_info(&title), // 字段 info
        ..Default::default() // 语句
    }; // 块结束
    let base = openapi_base(v); // 变量 base

    let mut items = Vec::new(); // 变量 items
    if let Some(paths) = v.get("paths").and_then(|p| p.as_object()) { // 条件分支
        for (path, path_item) in paths { // 循环
            if let Some(obj) = path_item.as_object() { // 条件分支
                for (method, op) in obj { // 循环
                    let m = method.to_ascii_uppercase(); // 变量 m
                    if !["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"].contains(&m.as_str()) // 条件分支
                    { // 语句
                        continue; // 语句
                    } // 块结束
                    let summary = op // 变量 summary
                        .get("summary") // 语句
                        .and_then(|s| s.as_str()) // 语句
                        .map(|s| s.to_string()) // 语句
                        .unwrap_or_else(|| format!("{} {}", m, path)); // 语句
                    let mut full_path = path.clone(); // 变量 full_path
                    let mut query_pairs: Vec<(String, String)> = Vec::new(); // 变量 query_pairs
                    let mut headers = Vec::new(); // 变量 headers
                    let mut body = None; // 变量 body
                    let mut all_params: Vec<&Value> = Vec::new(); // 变量 all_params
                    if let Some(ps) = path_item.get("parameters").and_then(|p| p.as_array()) { // 条件分支
                        all_params.extend(ps.iter()); // 语句
                    } // 块结束
                    if let Some(ps) = op.get("parameters").and_then(|p| p.as_array()) { // 条件分支
                        all_params.extend(ps.iter()); // 语句
                    } // 块结束
                    for p in all_params { // 循环
                        let loc = p.get("in").and_then(|x| x.as_str()).unwrap_or(""); // 变量 loc
                        let name = p // 变量 name
                            .get("name") // 语句
                            .and_then(|x| x.as_str()) // 语句
                            .unwrap_or("") // 语句
                            .to_string(); // 语句
                        let val = p // 变量 val
                            .get("example") // 语句
                            .or_else(|| p.get("schema").and_then(|s| s.get("default"))) // 语句
                            .or_else(|| p.get("schema").and_then(|s| s.get("example"))) // 语句
                            .map(|x| { // 语句
                                x.as_str() // 语句
                                    .map(|s| s.to_string()) // 语句
                                    .unwrap_or_else(|| x.to_string()) // 语句
                            }) // 语句
                            .unwrap_or_default(); // 语句
                        match loc { // 分支匹配
                            "header" => headers.push(header(&name, &val)), // 匹配分支
                            "query" => { // 匹配分支
                                if !name.is_empty() { // 条件分支
                                    query_pairs.push((name, val)); // 语句
                                } // 块结束
                            } // 块结束
                            "path" => { // 匹配分支
                                full_path = full_path.replace(&format!("{{{}}}", name), &val); // 语句
                            } // 块结束
                            // Swagger 2 body parameter
                            "body" => { // 匹配分支
                                if let Some(schema) = p.get("schema") { // 条件分支
                                    let example = schema // 变量 example
                                        .get("example") // 语句
                                        .or_else(|| schema.get("default")) // 语句
                                        .cloned(); // 语句
                                    let content = example // 变量 content
                                        .map(|e| serde_json::to_string_pretty(&e).unwrap_or_default()) // 语句
                                        .unwrap_or_else(|| "{}".to_string()); // 语句
                                    let ct = op // 变量 ct
                                        .get("consumes") // 语句
                                        .and_then(|c| c.as_array()) // 语句
                                        .and_then(|a| a.get(0)) // 语句
                                        .and_then(|x| x.as_str()) // 语句
                                        .unwrap_or("application/json"); // 语句
                                    headers.push(header("Content-Type", ct)); // 语句
                                    body = Some(Body { // 语句
                                        mode: "raw".into(), // 字段 mode
                                        raw: Some(content), // 字段 raw
                                        options: Some( // 语句
                                            serde_json::json!({"raw":{"language":"json"}}), // 字段 serde_json
                                        ), // 语句
                                        ..Default::default() // 语句
                                    }); // 语句
                                } // 块结束
                            } // 块结束
                            _ => {} // 匹配分支
                        } // 块结束
                    } // 块结束
                    let mut full_url = format!("{}{}", base.trim_end_matches('/'), full_path); // 变量 full_url
                    if !query_pairs.is_empty() { // 条件分支
                        let mut sep = if full_url.contains('?') { '&' } else { '?' }; // 变量 sep
                        for (k, v) in &query_pairs { // 循环
                            full_url.push(sep); // 语句
                            sep = '&'; // 语句
                            full_url.push_str(&format!("{}={}", k, v)); // 语句
                        } // 块结束
                    } // 块结束
                    if let Some(rb) = op.get("requestBody") { // 条件分支
                        if let Some(content) = rb.get("content").and_then(|c| c.as_object()) { // 条件分支
                            for (ct, media) in content { // 循环
                                let example = media // 变量 example
                                    .get("example") // 语句
                                    .or_else(|| { // 语句
                                        media // 语句
                                            .get("schema") // 语句
                                            .and_then(|s| s.get("example")) // 语句
                                    }) // 语句
                                    .map(|e| serde_json::to_string_pretty(e).unwrap_or_default()); // 语句
                                if let Some(ex) = example { // 条件分支
                                    headers.push(header("Content-Type", ct)); // 语句
                                    body = Some(Body { // 语句
                                        mode: "raw".into(), // 字段 mode
                                        raw: Some(ex), // 字段 raw
                                        options: Some(serde_json::json!({"raw":{"language":"json"}})), // 字段 options
                                        ..Default::default() // 语句
                                    }); // 语句
                                    break; // 语句
                                } // 块结束
                            } // 块结束
                        } // 块结束
                    } // 块结束
                    items.push(Item { // 语句
                        name: summary, // 字段 name
                        request: Some(Request { // 语句
                            method: m, // 字段 method
                            url: UrlSpec::new(full_url), // 字段 url
                            header: headers, // 字段 header
                            body, // 语句
                            auth: openapi_security_auth(v, op), // 字段 auth
                            ..Default::default() // 语句
                        }), // 语句
                        ..Default::default() // 语句
                    }); // 语句
                } // 块结束
            } // 块结束
        } // 块结束
    } // 块结束
    c.item = items; // 语句
    c // 语句
} // 块结束

/// Tokenize a shell-like string honoring single/double quotes.
fn shell_tokens(s: &str) -> Vec<String> { // 函数 shell_tokens
    let mut tokens = Vec::new(); // 变量 tokens
    let mut cur = String::new(); // 变量 cur
    let mut quote: Option<char> = None; // 变量 quote
    let mut chars = s.chars().peekable(); // 变量 chars
    while let Some(c) = chars.next() { // 循环
        match quote { // 分支匹配
            Some(q) => { // 匹配分支
                if c == q { // 条件分支
                    quote = None; // 语句
                } else if c == '\\' && q == '"' { // 条件分支
                    if let Some(n) = chars.next() { // 条件分支
                        cur.push(n); // 语句
                    } // 块结束
                } else { // 语句
                    cur.push(c); // 语句
                } // 块结束
            } // 块结束
            None => match c { // 匹配分支
                '\'' | '"' => quote = Some(c), // 匹配分支
                '\\' => { // 匹配分支
                    if let Some(n) = chars.next() { // 条件分支
                        cur.push(n); // 语句
                    } // 块结束
                } // 块结束
                c if c.is_whitespace() => { // 匹配分支
                    if !cur.is_empty() { // 条件分支
                        tokens.push(std::mem::take(&mut cur)); // 语句
                    } // 块结束
                } // 块结束
                _ => cur.push(c), // 匹配分支
            }, // 块结束
        } // 块结束
    } // 块结束
    if !cur.is_empty() { // 条件分支
        tokens.push(cur); // 语句
    } // 块结束
    tokens // 语句
} // 块结束

pub fn import_curl(cmd: &str) -> Collection { // 函数 import_curl
    let toks = shell_tokens(cmd); // 变量 toks
    let mut method = String::new(); // 变量 method
    let mut explicit_url = String::new(); // 变量 explicit_url
    let mut bare_url = String::new(); // 变量 bare_url
    let mut headers: Vec<Header> = Vec::new(); // 变量 headers
    let mut raw_parts: Vec<String> = Vec::new(); // 变量 raw_parts
    let mut urlencoded: Vec<KV> = Vec::new(); // 变量 urlencoded
    let mut formdata: Vec<KV> = Vec::new(); // 变量 formdata
    let mut upload_file: Option<String> = None; // 变量 upload_file
    let mut basic: Option<(String, String)> = None; // 变量 basic
    let mut cookies: Vec<String> = Vec::new(); // 变量 cookies
    let mut get_mode = false; // 变量 get_mode
    let mut json_mode = false; // 变量 json_mode

    let mut i = 0; // 变量 i
    // 读取某个需要参数的开关的取值；自动跳过后面的 token。
    while i < toks.len() { // 循环
        let t = toks[i].as_str(); // 变量 t
        // 兼容 `--flag=value` 形式
        let (flag, inline_val) = match t.split_once('=') { // 调用 let
            Some((f, v)) if f.starts_with("--") => (f, Some(v.to_string())), // 匹配分支
            _ => (t, None), // 匹配分支
        }; // 块结束
        let take_val = |i: &mut usize| -> Option<String> { // 变量 take_val
            if let Some(v) = &inline_val { // 条件分支
                return Some(v.clone()); // 返回
            } // 块结束
            if *i + 1 < toks.len() { // 条件分支
                *i += 1; // 语句
                Some(toks[*i].clone()) // 调用 Some
            } else { // 语句
                None // 语句
            } // 块结束
        }; // 块结束
        match flag { // 分支匹配
            "-X" | "--request" => { // 匹配分支
                if let Some(v) = take_val(&mut i) { // 条件分支
                    method = v.to_uppercase(); // 语句
                } // 块结束
            } // 块结束
            "-H" | "--header" => { // 匹配分支
                if let Some(v) = take_val(&mut i) { // 条件分支
                    if let Some((k, val)) = v.split_once(':') { // 条件分支
                        headers.push(header(k.trim(), val.trim())); // 语句
                    } // 块结束
                } // 块结束
            } // 块结束
            "-d" | "--data" | "--data-raw" | "--data-binary" | "--data-ascii" => { // 匹配分支
                if let Some(v) = take_val(&mut i) { // 条件分支
                    // @file 形式：读文件内容（若存在），否则原样保留
                    if let Some(path) = v.strip_prefix('@') { // 条件分支
                        if let Ok(content) = std::fs::read_to_string(path) { // 条件分支
                            raw_parts.push(content); // 语句
                        } else { // 语句
                            raw_parts.push(v); // 语句
                        } // 块结束
                    } else { // 语句
                        raw_parts.push(v); // 语句
                    } // 块结束
                } // 块结束
            } // 块结束
            "--data-urlencode" => { // 匹配分支
                if let Some(v) = take_val(&mut i) { // 条件分支
                    match v.split_once('=') { // 分支匹配
                        Some((k, val)) => urlencoded.push(KV { // 匹配分支
                            key: k.to_string(), // 字段 key
                            value: val.to_string(), // 字段 value
                            ..Default::default() // 语句
                        }), // 语句
                        None => urlencoded.push(KV { // 匹配分支
                            key: v.clone(), // 字段 key
                            value: String::new(), // 字段 value
                            ..Default::default() // 语句
                        }), // 语句
                    } // 块结束
                } // 块结束
            } // 块结束
            "-F" | "--form" => { // 匹配分支
                if let Some(v) = take_val(&mut i) { // 条件分支
                    let (k, val) = v.split_once('=').unwrap_or((v.as_str(), "")); // 调用 let
                    let key = k.split(';').next().unwrap_or(k).trim().to_string(); // 变量 key
                    if let Some(src) = val.strip_prefix('@') { // 条件分支
                        let path = src.split(';').next().unwrap_or(src).trim(); // 变量 path
                        formdata.push(KV { // 语句
                            key, // 语句
                            typ: Some("file".into()), // 字段 typ
                            src: Some(serde_json::json!(path)), // 字段 src
                            ..Default::default() // 语句
                        }); // 语句
                    } else { // 语句
                        formdata.push(KV { // 语句
                            key, // 语句
                            value: val.to_string(), // 字段 value
                            ..Default::default() // 语句
                        }); // 语句
                    } // 块结束
                } // 块结束
            } // 块结束
            "-u" | "--user" => { // 匹配分支
                if let Some(v) = take_val(&mut i) { // 条件分支
                    if let Some((u, p)) = v.split_once(':') { // 条件分支
                        basic = Some((u.to_string(), p.to_string())); // 语句
                    } // 块结束
                } // 块结束
            } // 块结束
            "-b" | "--cookie" => { // 匹配分支
                if let Some(v) = take_val(&mut i) { // 条件分支
                    cookies.push(v); // 语句
                } // 块结束
            } // 块结束
            "-A" | "--user-agent" => { // 匹配分支
                if let Some(v) = take_val(&mut i) { // 条件分支
                    headers.push(header("User-Agent", &v)); // 语句
                } // 块结束
            } // 块结束
            "-e" | "--referer" => { // 匹配分支
                if let Some(v) = take_val(&mut i) { // 条件分支
                    headers.push(header("Referer", &v)); // 语句
                } // 块结束
            } // 块结束
            "-T" | "--upload-file" => { // 匹配分支
                if let Some(v) = take_val(&mut i) { // 条件分支
                    upload_file = Some(v); // 语句
                } // 块结束
            } // 块结束
            "--url" => { // 匹配分支
                if let Some(v) = take_val(&mut i) { // 条件分支
                    explicit_url = v; // 语句
                } // 块结束
            } // 块结束
            "-G" | "--get" => get_mode = true, // 匹配分支
            "--json" => { // 匹配分支
                if let Some(v) = take_val(&mut i) { // 条件分支
                    raw_parts.push(v); // 语句
                } // 块结束
                json_mode = true; // 语句
            } // 块结束
            // 以下开关无参数或对本模型无意义，直接忽略
            "-k" | "--insecure" | "--compressed" | "-L" | "--location" | "-s" | "--silent" // 语句
            | "-i" | "--include" | "-v" | "--verbose" | "--globoff" | "-g" => {} // 匹配分支
            "-o" | "--output" | "-x" | "--proxy" | "--cacert" | "--cert" | "--key" => { // 匹配分支
                let _ = take_val(&mut i); // 变量 _
            } // 块结束
            "curl" | "curl.exe" => {} // 匹配分支
            _ => { // 匹配分支
                if !flag.starts_with('-') && bare_url.is_empty() { // 条件分支
                    bare_url = toks[i].clone(); // 语句
                } // 块结束
            } // 块结束
        } // 块结束
        i += 1; // 语句
    } // 块结束

    let mut url = if !explicit_url.is_empty() { // 变量 url
        explicit_url // 语句
    } else { // 语句
        bare_url // 语句
    }; // 块结束

    // 方法推断
    if method.is_empty() { // 条件分支
        method = if get_mode { // 语句
            "GET".into() // 语句
        } else if !formdata.is_empty() // 条件分支
            || !raw_parts.is_empty() // 语句
            || !urlencoded.is_empty() // 语句
            || upload_file.is_some() // 语句
        { // 语句
            "POST".into() // 语句
        } else { // 语句
            "GET".into() // 语句
        }; // 块结束
    } // 块结束
    if get_mode { // 条件分支
        method = "GET".into(); // 语句
    } // 块结束

    // -G：把数据拼到查询串
    if get_mode { // 条件分支
        let mut qs = String::new(); // 变量 qs
        let mut push_q = |pair: String| { // 变量 push_q
            if !qs.is_empty() { // 条件分支
                qs.push('&'); // 语句
            } // 块结束
            qs.push_str(&pair); // 语句
        }; // 块结束
        for p in &raw_parts { // 循环
            push_q(p.clone()); // 调用 push_q
        } // 块结束
        for kv in &urlencoded { // 循环
            push_q(format!("{}={}", kv.key, kv.value)); // 调用 push_q
        } // 块结束
        if !qs.is_empty() { // 条件分支
            url.push(if url.contains('?') { '&' } else { '?' }); // 语句
            url.push_str(&qs); // 语句
        } // 块结束
        raw_parts.clear(); // 语句
        urlencoded.clear(); // 语句
    } // 块结束

    // Cookie 头
    if !cookies.is_empty() { // 条件分支
        headers.push(header("Cookie", &cookies.join("; "))); // 语句
    } // 块结束
    if json_mode && !raw_parts.is_empty() { // 条件分支
        headers.push(header("Content-Type", "application/json")); // 语句
    } // 块结束

    // 请求体
    let body = if !formdata.is_empty() { // 变量 body
        Some(Body { // 调用 Some
            mode: "formdata".into(), // 字段 mode
            formdata, // 语句
            ..Default::default() // 语句
        }) // 语句
    } else if !urlencoded.is_empty() { // 条件分支
        Some(Body { // 调用 Some
            mode: "urlencoded".into(), // 字段 mode
            urlencoded, // 语句
            ..Default::default() // 语句
        }) // 语句
    } else if let Some(path) = upload_file { // 条件分支
        Some(Body { // 调用 Some
            mode: "file".into(), // 字段 mode
            file: Some(FileRef { // 语句
                src: Some(path), // 字段 src
                content: None, // 字段 content
            }), // 语句
            ..Default::default() // 语句
        }) // 语句
    } else if !raw_parts.is_empty() { // 条件分支
        let raw = raw_parts.join("&"); // 变量 raw
        let trimmed = raw.trim_start(); // 变量 trimmed
        let lang = if json_mode || trimmed.starts_with('{') || trimmed.starts_with('[') { // 变量 lang
            "json" // 语句
        } else { // 语句
            "text" // 语句
        }; // 块结束
        Some(Body { // 调用 Some
            mode: "raw".into(), // 字段 mode
            raw: Some(raw), // 字段 raw
            options: Some(serde_json::json!({"raw":{"language":lang}})), // 字段 options
            ..Default::default() // 语句
        }) // 语句
    } else { // 语句
        None // 语句
    }; // 块结束

    let auth = basic.map(|(u, p)| Auth { // 变量 auth
        kind: "basic".into(), // 字段 kind
        basic: vec![ // 语句
            KV { key: "username".into(), value: u, ..Default::default() }, // 语句
            KV { key: "password".into(), value: p, ..Default::default() }, // 语句
        ], // 语句
        ..Default::default() // 语句
    }); // 语句

    let req = Request { // 变量 req
        method, // 语句
        url: UrlSpec::new(url), // 字段 url
        header: headers, // 字段 header
        body, // 语句
        auth, // 语句
        ..Default::default() // 语句
    }; // 块结束
    Collection { // 语句
        info: new_info("cURL Import"), // 字段 info
        item: vec![Item { // 语句
            name: "cURL request".into(), // 字段 name
            request: Some(req), // 字段 request
            ..Default::default() // 语句
        }], // 语句
        ..Default::default() // 语句
    } // 块结束
} // 块结束

pub fn import_har(v: &Value) -> Result<Collection> { // 函数 import_har
    let entries = v // 变量 entries
        .get("log") // 语句
        .and_then(|l| l.get("entries")) // 语句
        .and_then(|e| e.as_array()) // 语句
        .ok_or_else(|| anyhow!("HAR missing log.entries"))?; // 语句
    let mut items = Vec::new(); // 变量 items
    for entry in entries { // 循环
        let req = entry.get("request"); // 变量 req
        let url = req // 变量 url
            .and_then(|r| r.get("url")) // 语句
            .and_then(|u| u.as_str()) // 语句
            .unwrap_or("") // 语句
            .to_string(); // 语句
        let method = req // 变量 method
            .and_then(|r| r.get("method")) // 语句
            .and_then(|m| m.as_str()) // 语句
            .unwrap_or("GET") // 语句
            .to_uppercase(); // 语句
        let mut headers = Vec::new(); // 变量 headers
        if let Some(hs) = req.and_then(|r| r.get("headers")).and_then(|h| h.as_array()) { // 条件分支
            for h in hs { // 循环
                let k = h.get("name").and_then(|x| x.as_str()).unwrap_or(""); // 变量 k
                let val = h.get("value").and_then(|x| x.as_str()).unwrap_or(""); // 变量 val
                headers.push(header(k, val)); // 语句
            } // 块结束
        } // 块结束
        let body = req // 变量 body
            .and_then(|r| r.get("postData")) // 语句
            .and_then(|p| p.get("text")) // 语句
            .and_then(|t| t.as_str()) // 语句
            .map(|t| Body { // 语句
                mode: "raw".into(), // 字段 mode
                raw: Some(t.to_string()), // 字段 raw
                options: Some(serde_json::json!({"raw":{"language":"json"}})), // 字段 options
                ..Default::default() // 语句
            }); // 语句
        items.push(Item { // 语句
            name: format!("{} {}", method, url), // 字段 name
            request: Some(Request { // 语句
                method, // 语句
                url: UrlSpec::new(url), // 字段 url
                header: headers, // 字段 header
                body, // 语句
                ..Default::default() // 语句
            }), // 语句
            ..Default::default() // 语句
        }); // 语句
    } // 块结束
    Ok(Collection { // 调用 Ok
        info: new_info("HAR Import"), // 字段 info
        item: items, // 字段 item
        ..Default::default() // 语句
    }) // 语句
} // 块结束

/// Merge `incoming` into `existing` non-destructively: variables by key, items by name
/// (duplicate item names get a " (n)" suffix instead of replacing existing requests).
pub fn merge_collections(existing: &mut Collection, incoming: Collection) { // 函数 merge_collections
    for v in incoming.variable { // 循环
        if let Some(slot) = existing.variable.iter_mut().find(|x| x.key == v.key) { // 条件分支
            *slot = v; // 语句
        } else { // 语句
            existing.variable.push(v); // 语句
        } // 块结束
    } // 块结束
    for mut item in incoming.item { // 循环
        if existing.item.iter().any(|x| x.name == item.name) { // 条件分支
            let base = item.name.clone(); // 变量 base
            let mut n = 1; // 变量 n
            while existing.item.iter().any(|x| x.name == item.name) { // 循环
                item.name = format!("{} ({})", base, n); // 语句
                n += 1; // 语句
            } // 块结束
        } // 块结束
        existing.item.push(item); // 语句
    } // 块结束
} // 块结束

pub fn export_collection(c: &mut Collection) -> Result<String> { // 函数 export_collection
    if c.info.postman_id.is_none() { // 条件分支
        c.info.postman_id = Some(uuid::Uuid::new_v4().to_string()); // 语句
    } // 块结束
    if c.info.schema.is_none() { // 条件分支
        c.info.schema = // 语句
            Some("https://schema.getpostman.com/json/collection/v2.1.0/collection.json".into()); // 调用 Some
    } // 块结束
    Ok(serde_json::to_string_pretty(c)?) // 调用 Ok
} // 块结束

/// Export an environment (or globals) as Postman-compatible JSON.
pub fn export_environment(e: &mut Environment) -> Result<String> { // 函数 export_environment
    if e.scope.is_none() { // 条件分支
        e.scope = Some("environment".to_string()); // 语句
    } // 块结束
    Ok(serde_json::to_string_pretty(e)?) // 调用 Ok
} // 块结束

#[cfg(test)] // 语句
mod tests { // 模块 tests
    use super::*; // 引入依赖

    #[test] // 语句
    fn postman_roundtrip() { // 函数 postman_roundtrip
        let json = r#"{
          "info": {"name":"Demo","_postman_id":"11111111-1111-1111-1111-111111111111",
                   "schema":"https://schema.getpostman.com/json/collection/v2.1.0/collection.json"},
          "item": [{"name":"r1","request":{"method":"GET","url":{"raw":"https://api.test/x"},
                    "header":[{"key":"A","value":"b"}]}}]
        }"#; // 语句
        let Imported::Collection(c) = import_postman_text(json, "demo.json").unwrap() else { // 变量 Imported
            panic!("expected collection"); // 语句
        }; // 块结束
        assert_eq!(c.info.name, "Demo"); // 语句
        assert_eq!(c.item.len(), 1); // 语句
        let mut c2 = *c; // 变量 c2
        let out = export_collection(&mut c2).unwrap(); // 变量 out
        let v: Value = serde_json::from_str(&out).unwrap(); // 变量 v
        assert_eq!(v["item"][0]["request"]["url"]["raw"], "https://api.test/x"); // 语句
    } // 块结束

    #[test] // 语句
    fn curl_import() { // 函数 curl_import
        let cmd = r#"curl 'https://api.test/users?x=1' -X POST -H 'Content-Type: application/json' -d '{"a":1}' -u user:pass"#; // 变量 cmd
        let c = import_curl(cmd); // 变量 c
        let r = c.item[0].request.as_ref().unwrap(); // 变量 r
        assert_eq!(r.method, "POST"); // 语句
        assert_eq!(r.url.raw, "https://api.test/users?x=1"); // 语句
        assert!(r.body.is_some()); // 语句
        assert_eq!(r.auth.as_ref().unwrap().kind, "basic"); // 语句
    } // 块结束

    #[test] // 语句
    fn curl_import_form_cookie_agent_and_upload() { // 函数 curl_import_form_cookie_agent_and_upload
        let cmd = r#"curl -X POST 'https://api.test/upload' -b 'sid=abc; theme=dark' -A 'pmrs/1.0' -F 'name=z' -F 'avatar=@C:\tmp\a.png;type=image/png'"#; // 变量 cmd
        let r = import_curl(cmd).item[0].request.clone().unwrap(); // 变量 r
        assert_eq!(r.method, "POST"); // 语句
        let b = r.body.as_ref().unwrap(); // 变量 b
        assert_eq!(b.mode, "formdata"); // 语句
        assert_eq!(b.formdata.len(), 2); // 语句
        assert_eq!(b.formdata[0].key, "name"); // 语句
        assert_eq!(b.formdata[0].value, "z"); // 语句
        assert_eq!(b.formdata[1].key, "avatar"); // 语句
        assert_eq!(b.formdata[1].typ.as_deref(), Some("file")); // 语句
        assert_eq!( // 语句
            b.formdata[1].src.as_ref().and_then(|v| v.as_str()), // 语句
            Some("C:\\tmp\\a.png") // 调用 Some
        ); // 块结束
        assert!(r // 语句
            .header // 语句
            .iter() // 语句
            .any(|h| h.key.eq_ignore_ascii_case("User-Agent") && h.value == "pmrs/1.0")); // 语句
        assert!(r // 语句
            .header // 语句
            .iter() // 语句
            .any(|h| h.key.eq_ignore_ascii_case("Cookie") && h.value == "sid=abc; theme=dark")); // 语句
    } // 块结束

    #[test] // 语句
    fn curl_import_get_data_urlencode_and_multiple_data() { // 函数 curl_import_get_data_urlencode_and_multiple_data
        let cmd = r#"curl 'https://api.test/search' -G -d 'q=rust' -d 'page=2'"#; // 变量 cmd
        let r = import_curl(cmd).item[0].request.clone().unwrap(); // 变量 r
        assert_eq!(r.method, "GET"); // 语句
        assert_eq!(r.url.raw, "https://api.test/search?q=rust&page=2"); // 语句
        assert!(r.body.is_none()); // 语句

        let cmd2 = // 变量 cmd2
            r#"curl 'https://api.test/form' --data-urlencode 'a=1' --data-urlencode 'b=hi there'"#; // 语句
        let r2 = import_curl(cmd2).item[0].request.clone().unwrap(); // 变量 r2
        assert_eq!(r2.method, "POST"); // 语句
        let b = r2.body.as_ref().unwrap(); // 变量 b
        assert_eq!(b.mode, "urlencoded"); // 语句
        assert_eq!(b.urlencoded.len(), 2); // 语句
        assert_eq!(b.urlencoded[1].value, "hi there"); // 语句
    } // 块结束

    #[test] // 语句
    fn curl_import_upload_file_and_json() { // 函数 curl_import_upload_file_and_json
        let cmd = r#"curl -T 'C:\tmp\data.bin' 'https://api.test/upload'"#; // 变量 cmd
        let r = import_curl(cmd).item[0].request.clone().unwrap(); // 变量 r
        let b = r.body.as_ref().unwrap(); // 变量 b
        assert_eq!(b.mode, "file"); // 语句
        assert_eq!( // 语句
            b.file.as_ref().and_then(|f| f.src.as_deref()), // 语句
            Some("C:\\tmp\\data.bin") // 调用 Some
        ); // 块结束

        let cmd2 = r#"curl --json '{"x":1}' https://api.test/j"#; // 变量 cmd2
        let r2 = import_curl(cmd2).item[0].request.clone().unwrap(); // 变量 r2
        assert_eq!(r2.method, "POST"); // 语句
        assert!(r2 // 语句
            .header // 语句
            .iter() // 语句
            .any(|h| h.key.eq_ignore_ascii_case("content-type") && h.value.contains("json"))); // 语句
        assert_eq!(r2.body.as_ref().unwrap().mode, "raw"); // 语句
    } // 块结束

    #[test] // 语句
    fn openapi_import() { // 函数 openapi_import
        let v: Value = serde_json::json!({ // 变量 v
          "info":{"title":"T","version":"1"}, // 语句
          "servers":[{"url":"https://api.test"}], // 语句
          "paths":{"/ping":{"get":{"summary":"Ping"}}} // 语句
        }); // 语句
        let c = import_openapi(&v); // 变量 c
        assert_eq!(c.item.len(), 1); // 语句
        assert_eq!(c.item[0].request.as_ref().unwrap().method, "GET"); // 语句
    } // 块结束

    #[test] // 语句
    fn swagger2_import_base_host_and_apikey() { // 函数 swagger2_import_base_host_and_apikey
        let v: Value = serde_json::json!({ // 变量 v
          "swagger":"2.0", // 语句
          "info":{"title":"S2","version":"1"}, // 语句
          "host":"api.test", // 语句
          "basePath":"/v1", // 语句
          "schemes":["https"], // 语句
          "securityDefinitions":{"k":{"type":"apiKey","name":"X-Key","in":"header"}}, // 语句
          "security":[{"k":[]}], // 语句
          "paths":{"/items/{id}":{"get":{ // 语句
             "summary":"Get", // 语句
             "parameters":[{"in":"path","name":"id","required":true,"schema":{"default":"9"}}] // 语句
          }}} // 语句
        }); // 语句
        let c = import_openapi(&v); // 变量 c
        let r = c.item[0].request.as_ref().unwrap(); // 变量 r
        assert_eq!(r.url.raw, "https://api.test/v1/items/9"); // 语句
        let auth = r.auth.as_ref().unwrap(); // 变量 auth
        assert_eq!(auth.kind, "apikey"); // 语句
        assert_eq!(auth.apikey_key(), Some("X-Key")); // 语句
    } // 块结束

    #[test] // 语句
    fn postman_v1_import() { // 函数 postman_v1_import
        let json = r#"{
          "name":"V1 Demo",
          "requests":[{
            "name":"Login","method":"POST","url":"https://api.test/login",
            "headers":"Content-Type: application/json\nX-Test: 1",
            "dataMode":"raw","rawModeData":"{\"u\":1}"
          }],
          "folders":[{"name":"F1","order":[],"requests":[{"name":"Ping","method":"GET","url":"https://api.test/ping","headers":""}]}]
        }"#; // 语句
        let Imported::Collection(c) = import_postman_text(json, "v1.json").unwrap() else { // 变量 Imported
            panic!("expected collection"); // 语句
        }; // 块结束
        assert_eq!(c.info.name, "V1 Demo"); // 语句
        assert_eq!(c.item.len(), 2); // folder + request
        let folder = &c.item[0]; // 变量 folder
        assert_eq!(folder.name, "F1"); // 语句
        assert_eq!(folder.item.as_ref().unwrap()[0].name, "Ping"); // 语句
        let login = &c.item[1]; // 变量 login
        let r = login.request.as_ref().unwrap(); // 变量 r
        assert_eq!(r.method, "POST"); // 语句
        assert_eq!(r.header.len(), 2); // 语句
        assert_eq!(r.body.as_ref().unwrap().mode, "raw"); // 语句
    } // 块结束

    #[test] // 语句
    fn globals_import() { // 函数 globals_import
        let json = r#"{"name":"Globals","values":[{"key":"g","value":"1","type":"default"}],"_postman_variable_scope":"globals"}"#; // 变量 json
        let Imported::Environment(e) = import_postman_text(json, "g.json").unwrap() else { // 变量 Imported
            panic!("expected environment/globals"); // 语句
        }; // 块结束
        assert_eq!(e.scope.as_deref(), Some("globals")); // 语句
        assert_eq!(e.values[0].key, "g"); // 语句
    } // 块结束
} // 块结束
