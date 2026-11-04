//! Postman Collection v2.1 compatible data model.
use serde::{Deserialize, Serialize, Serializer}; // 引入依赖
use serde_json::Value; // 引入依赖

fn default_method() -> String { // 函数 default_method
    "GET".to_string() // 语句
} // 块结束

#[derive(Debug, Clone, Serialize, Deserialize, Default)] // 语句
pub struct Collection { // 结构体 Collection
    #[serde(default)] // 语句
    pub info: CollectionInfo, // 字段 info
    #[serde(default)] // 语句
    pub item: Vec<Item>, // 字段 item
    #[serde(default)] // 语句
    pub variable: Vec<Variable>, // 字段 variable
    #[serde(default, skip_serializing_if = "Option::is_none")] // 语句
    pub auth: Option<Auth>, // 字段 auth
    #[serde(default, skip_serializing_if = "Vec::is_empty")] // 语句
    pub event: Vec<Event>, // 字段 event
} // 块结束

#[derive(Debug, Clone, Serialize, Deserialize, Default)] // 语句
pub struct CollectionInfo { // 结构体 CollectionInfo
    #[serde(default)] // 语句
    pub name: String, // 字段 name
    #[serde(rename = "_postman_id", default, skip_serializing_if = "Option::is_none")] // 语句
    pub postman_id: Option<String>, // 字段 postman_id
    #[serde(default, skip_serializing_if = "Option::is_none")] // 语句
    pub schema: Option<String>, // 字段 schema
    #[serde(default, skip_serializing_if = "Option::is_none")] // 语句
    pub description: Option<Value>, // 字段 description
} // 块结束

#[derive(Debug, Clone, Serialize, Deserialize, Default)] // 语句
pub struct Item { // 结构体 Item
    #[serde(default)] // 语句
    pub name: String, // 字段 name
    #[serde(default, skip_serializing_if = "Option::is_none")] // 语句
    pub item: Option<Vec<Item>>, // 字段 item
    #[serde(default, skip_serializing_if = "Option::is_none")] // 语句
    pub request: Option<Request>, // 字段 request
    /// Saved example responses (Postman `item.response`). Preserved on import/export.
    #[serde(default, skip_serializing_if = "Vec::is_empty")] // 语句
    pub response: Vec<ResponseExample>, // 字段 response
    #[serde(default, skip_serializing_if = "Vec::is_empty")] // 语句
    pub event: Vec<Event>, // 字段 event
    #[serde(default, skip_serializing_if = "Option::is_none")] // 语句
    pub description: Option<Value>, // 字段 description
    /// 文件夹级授权（继承给子项；请求自身 auth 优先）
    #[serde(default, skip_serializing_if = "Option::is_none")] // 语句
    pub auth: Option<Auth>, // 字段 auth
    /// 文件夹级变量（继承给子项；优先级介于环境与集合变量之间）
    #[serde(default, skip_serializing_if = "Vec::is_empty")] // 语句
    pub variable: Vec<Variable>, // 字段 variable
} // 块结束

/// A saved example response attached to a request (Postman "Examples").
#[derive(Debug, Clone, Serialize, Deserialize, Default)] // 语句
pub struct ResponseExample { // 结构体 ResponseExample
    #[serde(default)] // 语句
    pub name: String, // 字段 name
    #[serde(rename = "originalRequest", default, skip_serializing_if = "Option::is_none")] // 语句
    pub original_request: Option<Value>, // 字段 original_request
    #[serde(default, skip_serializing_if = "Option::is_none")] // 语句
    pub status: Option<String>, // 字段 status
    #[serde(default, skip_serializing_if = "Option::is_none")] // 语句
    pub code: Option<u16>, // 字段 code
    #[serde(default)] // 语句
    pub header: Vec<Header>, // 字段 header
    #[serde(default, skip_serializing_if = "Option::is_none")] // 语句
    pub body: Option<String>, // 字段 body
    #[serde(rename = "_postman_previewlanguage", default, skip_serializing_if = "Option::is_none")] // 语句
    pub preview_language: Option<String>, // 字段 preview_language
} // 块结束

#[derive(Debug, Clone, Serialize, Deserialize, Default)] // 语句
pub struct Request { // 结构体 Request
    #[serde(default = "default_method")] // 语句
    pub method: String, // 字段 method
    #[serde(default)] // 语句
    pub header: Vec<Header>, // 字段 header
    #[serde(default, skip_serializing_if = "Option::is_none")] // 语句
    pub body: Option<Body>, // 字段 body
    #[serde(default, skip_serializing_if = "Option::is_none")] // 语句
    pub auth: Option<Auth>, // 字段 auth
    #[serde(default, deserialize_with = "de_url", skip_serializing_if = "UrlSpec::is_empty")] // 语句
    pub url: UrlSpec, // 字段 url
    /// Optional declarative assertions (postman-rs extension).
    #[serde(default, skip_serializing_if = "Vec::is_empty")] // 语句
    pub tests: Vec<TestAssertion>, // 字段 tests
    #[serde(default, skip_serializing_if = "Vec::is_empty")] // 语句
    pub event: Vec<Event>, // 字段 event
} // 块结束

/// Structured URL, compatible with Postman `url` object:
/// `{ "raw": "...", "host": [...], "path": [...], "query": [...], "variable": [...], "hash": "..." }`
#[derive(Debug, Clone, Default, PartialEq)] // 语句
pub struct UrlSpec { // 结构体 UrlSpec
    pub raw: String, // 字段 raw
    pub host: Vec<String>, // 字段 host
    pub path: Vec<String>, // 字段 path
    /// query 参数（含 disabled 标记，用于保真 disabled 项）
    pub query: Vec<KV>, // 字段 query
    /// 路径变量（Postman `url.variable`，替换 `:name`）
    pub variable: Vec<KV>, // 字段 variable
    pub hash: Option<String>, // 字段 hash
} // 块结束

impl UrlSpec { // 实现
    pub fn is_empty(&self) -> bool { // 函数 is_empty
        self.raw.is_empty() // 语句
    } // 块结束
    pub fn new(raw: impl Into<String>) -> Self { // 函数 new
        UrlSpec { // 语句
            raw: raw.into(), // 字段 raw
            ..Default::default() // 语句
        } // 块结束
    } // 块结束

    /// 计算实际发送用的 URL：
    /// - 应用路径变量 `:name` → value；
    /// - 若有结构化 query，则按 enabled 项重建（丢弃 disabled 项）；
    /// - 追加 hash。
    pub fn effective_url(&self) -> String { // 函数 effective_url
        let mut base = if self.raw.trim().is_empty() { // 变量 base
            self.build_from_parts() // 语句
        } else { // 语句
            self.raw.clone() // 语句
        }; // 块结束
        for v in &self.variable { // 循环
            if v.disabled || v.key.is_empty() { // 条件分支
                continue; // 语句
            } // 块结束
            base = base.replace(&format!(":{}", v.key), &v.value); // 语句
        } // 块结束
        if !self.query.is_empty() { // 条件分支
            let head = match base.split_once('?') { // 变量 head
                Some((h, _)) => h.to_string(), // 匹配分支
                None => base.clone(), // 匹配分支
            }; // 块结束
            let mut qs = String::new(); // 变量 qs
            for kv in self // 循环
                .query // 语句
                .iter() // 语句
                .filter(|k| !k.disabled && !k.key.is_empty()) // 语句
            { // 语句
                if !qs.is_empty() { // 条件分支
                    qs.push('&'); // 语句
                } // 块结束
                qs.push_str(&format!("{}={}", kv.key, kv.value)); // 语句
            } // 块结束
            base = if qs.is_empty() { // 语句
                head // 语句
            } else { // 语句
                format!("{}?{}", head, qs) // 语句
            }; // 块结束
        } // 块结束
        if let Some(h) = &self.hash { // 条件分支
            if !h.is_empty() && !base.contains('#') { // 条件分支
                base.push('#'); // 语句
                base.push_str(h.trim_start_matches('#')); // 语句
            } // 块结束
        } // 块结束
        base // 语句
    } // 块结束

    fn build_from_parts(&self) -> String { // 函数 build_from_parts
        let mut s = String::new(); // 变量 s
        if !self.host.is_empty() { // 条件分支
            s.push_str(&self.host.join(".")); // 语句
        } // 块结束
        for p in &self.path { // 循环
            s.push('/'); // 语句
            s.push_str(p); // 语句
        } // 块结束
        s // 语句
    } // 块结束
} // 块结束

fn de_url<'de, D>(d: D) -> Result<UrlSpec, D::Error> // 函数 de_url
where // 语句
    D: serde::Deserializer<'de>, // 字段 D
{ // 语句
    let v = Value::deserialize(d)?; // 变量 v
    Ok(match v { // 调用 Ok
        Value::String(s) => UrlSpec::new(s), // 匹配分支
        Value::Object(m) => { // 匹配分支
            let str_vec = |key: &str| -> Vec<String> { // 变量 str_vec
                m.get(key) // 语句
                    .and_then(|x| x.as_array()) // 语句
                    .map(|a| { // 语句
                        a.iter() // 语句
                            .map(|e| { // 语句
                                e.as_str() // 语句
                                    .map(|s| s.to_string()) // 语句
                                    .unwrap_or_else(|| e.to_string()) // 语句
                            }) // 语句
                            .collect() // 语句
                    }) // 语句
                    .unwrap_or_default() // 语句
            }; // 块结束
            let kv_vec = |key: &str| -> Vec<KV> { // 变量 kv_vec
                m.get(key) // 语句
                    .and_then(|x| x.as_array()) // 语句
                    .map(|a| { // 语句
                        a.iter() // 语句
                            .filter_map(|e| serde_json::from_value::<KV>(e.clone()).ok()) // 语句
                            .collect() // 语句
                    }) // 语句
                    .unwrap_or_default() // 语句
            }; // 块结束
            UrlSpec { // 语句
                raw: m // 语句
                    .get("raw") // 语句
                    .and_then(|x| x.as_str()) // 语句
                    .map(|s| s.to_string()) // 语句
                    .unwrap_or_default(), // 语句
                host: str_vec("host"), // 字段 host
                path: str_vec("path"), // 字段 path
                query: kv_vec("query"), // 字段 query
                variable: kv_vec("variable"), // 字段 variable
                hash: m.get("hash").and_then(|x| x.as_str()).map(|s| s.to_string()), // 字段 hash
            } // 块结束
        } // 块结束
        _ => UrlSpec::default(), // 匹配分支
    }) // 语句
} // 块结束

impl Serialize for UrlSpec { // 实现
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error> // 函数 serialize
    where // 语句
        S: Serializer, // 字段 S
    { // 语句
        use serde::ser::SerializeMap; // 引入依赖
        let mut map = s.serialize_map(None)?; // 变量 map
        map.serialize_entry("raw", &self.raw)?; // 语句
        if !self.host.is_empty() { // 条件分支
            map.serialize_entry("host", &self.host)?; // 语句
        } // 块结束
        if !self.path.is_empty() { // 条件分支
            map.serialize_entry("path", &self.path)?; // 语句
        } // 块结束
        if !self.query.is_empty() { // 条件分支
            map.serialize_entry("query", &self.query)?; // 语句
        } // 块结束
        if !self.variable.is_empty() { // 条件分支
            map.serialize_entry("variable", &self.variable)?; // 语句
        } // 块结束
        if let Some(h) = &self.hash { // 条件分支
            if !h.is_empty() { // 条件分支
                map.serialize_entry("hash", h)?; // 语句
            } // 块结束
        } // 块结束
        map.end() // 语句
    } // 块结束
} // 块结束

#[derive(Debug, Clone, Serialize, Deserialize, Default)] // 语句
pub struct Header { // 结构体 Header
    #[serde(default)] // 语句
    pub key: String, // 字段 key
    #[serde(default)] // 语句
    pub value: String, // 字段 value
    #[serde(default)] // 语句
    pub disabled: bool, // 字段 disabled
    #[serde(default, skip_serializing_if = "Option::is_none")] // 语句
    pub description: Option<Value>, // 字段 description
} // 块结束

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)] // 语句
pub struct KV { // 结构体 KV
    #[serde(default)] // 语句
    pub key: String, // 字段 key
    #[serde(default)] // 语句
    pub value: String, // 字段 value
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")] // 语句
    pub typ: Option<String>, // 字段 typ
    #[serde(default, skip_serializing_if = "Option::is_none")] // 语句
    pub src: Option<Value>, // 字段 src
    #[serde(default)] // 语句
    pub disabled: bool, // 字段 disabled
} // 块结束

#[derive(Debug, Clone, Serialize, Deserialize, Default)] // 语句
pub struct FileRef { // 结构体 FileRef
    #[serde(default)] // 语句
    pub src: Option<String>, // 字段 src
    #[serde(default, skip_serializing_if = "Option::is_none")] // 语句
    pub content: Option<String>, // 字段 content
} // 块结束

#[derive(Debug, Clone, Serialize, Deserialize, Default)] // 语句
pub struct GraphQL { // 结构体 GraphQL
    #[serde(default)] // 语句
    pub query: String, // 字段 query
    #[serde(default)] // 语句
    pub variables: String, // 字段 variables
} // 块结束

#[derive(Debug, Clone, Serialize, Deserialize, Default)] // 语句
pub struct Body { // 结构体 Body
    #[serde(default)] // 语句
    pub mode: String, // 字段 mode
    #[serde(default, skip_serializing_if = "Option::is_none")] // 语句
    pub raw: Option<String>, // 字段 raw
    #[serde(default, skip_serializing_if = "Vec::is_empty")] // 语句
    pub urlencoded: Vec<KV>, // 字段 urlencoded
    #[serde(default, skip_serializing_if = "Vec::is_empty")] // 语句
    pub formdata: Vec<KV>, // 字段 formdata
    #[serde(default, skip_serializing_if = "Option::is_none")] // 语句
    pub file: Option<FileRef>, // 字段 file
    #[serde(default, skip_serializing_if = "Option::is_none")] // 语句
    pub graphql: Option<GraphQL>, // 字段 graphql
    #[serde(default, skip_serializing_if = "Option::is_none")] // 语句
    pub options: Option<Value>, // 字段 options
} // 块结束

impl Body { // 实现
    /// Best-effort content type for raw bodies.
    pub fn raw_content_type(&self) -> Option<String> { // 函数 raw_content_type
        let lang = self // 变量 lang
            .options // 语句
            .as_ref() // 语句
            .and_then(|o| o.get("raw")) // 语句
            .and_then(|r| r.get("language")) // 语句
            .and_then(|l| l.as_str()) // 语句
            .unwrap_or("text"); // 语句
        let ct = match lang { // 变量 ct
            "json" => "application/json", // 匹配分支
            "xml" => "application/xml", // 匹配分支
            "html" => "text/html", // 匹配分支
            "javascript" => "application/javascript", // 匹配分支
            "text" => "text/plain", // 匹配分支
            other => { // 匹配分支
                let _ = other; // 变量 _
                "text/plain" // 语句
            } // 块结束
        }; // 块结束
        Some(ct.to_string()) // 调用 Some
    } // 块结束
} // 块结束

/// Postman auth object: `{ "type": "bearer", "bearer": [{"key":"token","value":"..."}] }`
#[derive(Debug, Clone, Serialize, Deserialize, Default)] // 语句
pub struct Auth { // 结构体 Auth
    #[serde(rename = "type", default)] // 语句
    pub kind: String, // 字段 kind
    #[serde(default, skip_serializing_if = "Vec::is_empty")] // 语句
    pub bearer: Vec<KV>, // 字段 bearer
    #[serde(default, skip_serializing_if = "Vec::is_empty")] // 语句
    pub basic: Vec<KV>, // 字段 basic
    #[serde(default, skip_serializing_if = "Vec::is_empty")] // 语句
    pub apikey: Vec<KV>, // 字段 apikey
    #[serde(flatten, default)] // 语句
    pub extra: serde_json::Map<String, Value>, // 字段 extra
} // 块结束

impl Auth { // 实现
    fn kv<'a>(list: &'a [KV], key: &str) -> Option<&'a str> { // 函数 kv
        list.iter() // 语句
            .find(|kv| kv.key.eq_ignore_ascii_case(key) && !kv.disabled) // 语句
            .map(|kv| kv.value.as_str()) // 语句
    } // 块结束
    pub fn bearer_token(&self) -> Option<&str> { // 函数 bearer_token
        Self::kv(&self.bearer, "token") // 调用 kv
    } // 块结束
    pub fn basic_user(&self) -> Option<&str> { // 函数 basic_user
        Self::kv(&self.basic, "username") // 调用 kv
    } // 块结束
    pub fn basic_pass(&self) -> Option<&str> { // 函数 basic_pass
        Self::kv(&self.basic, "password") // 调用 kv
    } // 块结束
    pub fn apikey_key(&self) -> Option<&str> { // 函数 apikey_key
        Self::kv(&self.apikey, "key") // 调用 kv
    } // 块结束
    pub fn apikey_value(&self) -> Option<&str> { // 函数 apikey_value
        Self::kv(&self.apikey, "value") // 调用 kv
    } // 块结束
    pub fn apikey_in(&self) -> &str { // 函数 apikey_in
        Self::kv(&self.apikey, "in").unwrap_or("header") // 调用 kv
    } // 块结束
    /// Extract OAuth2 fields from the flattened `oauth2` array (Postman stores it as KV list).
    pub fn oauth2_params(&self) -> std::collections::HashMap<String, String> { // 函数 oauth2_params
        let mut m = std::collections::HashMap::new(); // 变量 m
        if let Some(Value::Array(arr)) = self.extra.get("oauth2") { // 条件分支
            for item in arr { // 循环
                if let (Some(k), Some(v)) = ( // 条件分支
                    item.get("key").and_then(|x| x.as_str()), // 语句
                    item.get("value"), // 语句
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
} // 块结束

#[derive(Debug, Clone, Serialize, Deserialize, Default)] // 语句
pub struct Script { // 结构体 Script
    #[serde(default)] // 语句
    pub exec: Vec<String>, // 字段 exec
    #[serde(rename = "type", default)] // 语句
    pub kind: Option<String>, // 字段 kind
} // 块结束

#[derive(Debug, Clone, Serialize, Deserialize, Default)] // 语句
pub struct Event { // 结构体 Event
    #[serde(default)] // 语句
    pub listen: String, // 字段 listen
    #[serde(default, skip_serializing_if = "Option::is_none")] // 语句
    pub script: Option<Script>, // 字段 script
} // 块结束

#[derive(Debug, Clone, Serialize, Deserialize, Default)] // 语句
pub struct Variable { // 结构体 Variable
    #[serde(default)] // 语句
    pub key: String, // 字段 key
    #[serde(default)] // 语句
    pub value: Value, // 字段 value
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")] // 语句
    pub typ: Option<String>, // 字段 typ
    #[serde(default, skip_serializing_if = "Option::is_none")] // 语句
    pub enabled: Option<bool>, // 字段 enabled
    #[serde(default, skip_serializing_if = "Option::is_none")] // 语句
    pub description: Option<Value>, // 字段 description
} // 块结束

#[derive(Debug, Clone, Serialize, Deserialize, Default)] // 语句
pub struct Environment { // 结构体 Environment
    #[serde(default, skip_serializing_if = "Option::is_none")] // 语句
    pub id: Option<String>, // 字段 id
    #[serde(default)] // 语句
    pub name: String, // 字段 name
    #[serde(default)] // 语句
    pub values: Vec<Variable>, // 字段 values
    #[serde(rename = "_postman_variable_scope", default, skip_serializing_if = "Option::is_none")] // 语句
    pub scope: Option<String>, // 字段 scope
} // 块结束

/// Declarative assertion, evaluated by `tests_engine`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)] // 语句
pub struct TestAssertion { // 结构体 TestAssertion
    #[serde(rename = "type", default)] // 语句
    pub kind: String, // 字段 kind
    #[serde(default, skip_serializing_if = "Option::is_none")] // 语句
    pub path: Option<String>, // 字段 path
    #[serde(default, skip_serializing_if = "Option::is_none")] // 语句
    pub key: Option<String>, // 字段 key
    #[serde(default, skip_serializing_if = "Option::is_none")] // 语句
    pub value: Option<Value>, // 字段 value
    #[serde(default, skip_serializing_if = "Option::is_none")] // 语句
    pub contains: Option<String>, // 字段 contains
    #[serde(default, skip_serializing_if = "Option::is_none")] // 语句
    pub equals: Option<Value>, // 字段 equals
    #[serde(default, skip_serializing_if = "Option::is_none")] // 语句
    pub max_time_ms: Option<u64>, // 字段 max_time_ms
} // 块结束

impl Variable { // 实现
    pub fn is_enabled(&self) -> bool { // 函数 is_enabled
        self.enabled.unwrap_or(true) // 语句
    } // 块结束
} // 块结束

#[cfg(test)] // 语句
mod url_tests { // 模块 url_tests
    use super::*; // 引入依赖

    #[test] // 语句
    fn effective_url_applies_path_vars_disabled_query_and_hash() { // 函数 effective_url_applies_path_vars_disabled_query_and_hash
        let u = UrlSpec { // 变量 u
            raw: "https://api.test/users/:id?keep=1&drop=2".into(), // 字段 raw
            query: vec![ // 语句
                KV { key: "keep".into(), value: "1".into(), ..Default::default() }, // 语句
                KV { key: "drop".into(), value: "2".into(), disabled: true, ..Default::default() }, // 语句
            ], // 语句
            variable: vec![KV { key: "id".into(), value: "42".into(), ..Default::default() }], // 字段 variable
            hash: Some("sec".into()), // 字段 hash
            ..Default::default() // 语句
        }; // 块结束
        let eff = u.effective_url(); // 变量 eff
        assert_eq!(eff, "https://api.test/users/42?keep=1#sec"); // 语句
    } // 块结束

    #[test] // 语句
    fn url_object_roundtrip_preserves_structure() { // 函数 url_object_roundtrip_preserves_structure
        let json = r#"{"method":"GET","url":{"raw":"https://h/x?a=1","query":[{"key":"a","value":"1"},{"key":"b","value":"2","disabled":true}],"variable":[{"key":"id","value":"7"}],"hash":"frag"}}"#; // 变量 json
        let req: Request = serde_json::from_str(json).unwrap(); // 变量 req
        let u: UrlSpec = req.url; // 变量 u
        assert_eq!(u.query.len(), 2); // 语句
        assert!(u.query[1].disabled); // 语句
        assert_eq!(u.variable[0].value, "7"); // 语句
        assert_eq!(u.hash.as_deref(), Some("frag")); // 语句
        let out = serde_json::to_string(&Request { url: u, ..Default::default() }).unwrap(); // 变量 out
        let v: serde_json::Value = serde_json::from_str(&out).unwrap(); // 变量 v
        assert_eq!(v["url"]["query"][1]["disabled"], true); // 语句
    } // 块结束
} // 块结束
