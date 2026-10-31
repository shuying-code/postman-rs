//! Postman Collection v2.1 compatible data model.
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;

fn default_method() -> String {
    "GET".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Collection {
    #[serde(default)]
    pub info: CollectionInfo,
    #[serde(default)]
    pub item: Vec<Item>,
    #[serde(default)]
    pub variable: Vec<Variable>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth: Option<Auth>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub event: Vec<Event>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CollectionInfo {
    #[serde(default)]
    pub name: String,
    #[serde(rename = "_postman_id", default, skip_serializing_if = "Option::is_none")]
    pub postman_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Item {
    #[serde(default)]
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item: Option<Vec<Item>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request: Option<Request>,
    /// Saved example responses (Postman `item.response`). Preserved on import/export.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub response: Vec<ResponseExample>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub event: Vec<Event>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<Value>,
    /// 文件夹级授权（继承给子项；请求自身 auth 优先）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth: Option<Auth>,
    /// 文件夹级变量（继承给子项；优先级介于环境与集合变量之间）
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variable: Vec<Variable>,
}

/// A saved example response attached to a request (Postman "Examples").
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResponseExample {
    #[serde(default)]
    pub name: String,
    #[serde(rename = "originalRequest", default, skip_serializing_if = "Option::is_none")]
    pub original_request: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<u16>,
    #[serde(default)]
    pub header: Vec<Header>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(rename = "_postman_previewlanguage", default, skip_serializing_if = "Option::is_none")]
    pub preview_language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Request {
    #[serde(default = "default_method")]
    pub method: String,
    #[serde(default)]
    pub header: Vec<Header>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<Body>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth: Option<Auth>,
    #[serde(default, deserialize_with = "de_url", skip_serializing_if = "UrlSpec::is_empty")]
    pub url: UrlSpec,
    /// Optional declarative assertions (postman-rs extension).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tests: Vec<TestAssertion>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub event: Vec<Event>,
}

/// Structured URL, compatible with Postman `url` object:
/// `{ "raw": "...", "host": [...], "path": [...], "query": [...], "variable": [...], "hash": "..." }`
#[derive(Debug, Clone, Default, PartialEq)]
pub struct UrlSpec {
    pub raw: String,
    pub host: Vec<String>,
    pub path: Vec<String>,
    /// query 参数（含 disabled 标记，用于保真 disabled 项）
    pub query: Vec<KV>,
    /// 路径变量（Postman `url.variable`，替换 `:name`）
    pub variable: Vec<KV>,
    pub hash: Option<String>,
}

impl UrlSpec {
    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }
    pub fn new(raw: impl Into<String>) -> Self {
        UrlSpec {
            raw: raw.into(),
            ..Default::default()
        }
    }

    /// 计算实际发送用的 URL：
    /// - 应用路径变量 `:name` → value；
    /// - 若有结构化 query，则按 enabled 项重建（丢弃 disabled 项）；
    /// - 追加 hash。
    pub fn effective_url(&self) -> String {
        let mut base = if self.raw.trim().is_empty() {
            self.build_from_parts()
        } else {
            self.raw.clone()
        };
        for v in &self.variable {
            if v.disabled || v.key.is_empty() {
                continue;
            }
            base = base.replace(&format!(":{}", v.key), &v.value);
        }
        if !self.query.is_empty() {
            let head = match base.split_once('?') {
                Some((h, _)) => h.to_string(),
                None => base.clone(),
            };
            let mut qs = String::new();
            for kv in self
                .query
                .iter()
                .filter(|k| !k.disabled && !k.key.is_empty())
            {
                if !qs.is_empty() {
                    qs.push('&');
                }
                qs.push_str(&format!("{}={}", kv.key, kv.value));
            }
            base = if qs.is_empty() {
                head
            } else {
                format!("{}?{}", head, qs)
            };
        }
        if let Some(h) = &self.hash {
            if !h.is_empty() && !base.contains('#') {
                base.push('#');
                base.push_str(h.trim_start_matches('#'));
            }
        }
        base
    }

    fn build_from_parts(&self) -> String {
        let mut s = String::new();
        if !self.host.is_empty() {
            s.push_str(&self.host.join("."));
        }
        for p in &self.path {
            s.push('/');
            s.push_str(p);
        }
        s
    }
}

fn de_url<'de, D>(d: D) -> Result<UrlSpec, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v = Value::deserialize(d)?;
    Ok(match v {
        Value::String(s) => UrlSpec::new(s),
        Value::Object(m) => {
            let str_vec = |key: &str| -> Vec<String> {
                m.get(key)
                    .and_then(|x| x.as_array())
                    .map(|a| {
                        a.iter()
                            .map(|e| {
                                e.as_str()
                                    .map(|s| s.to_string())
                                    .unwrap_or_else(|| e.to_string())
                            })
                            .collect()
                    })
                    .unwrap_or_default()
            };
            let kv_vec = |key: &str| -> Vec<KV> {
                m.get(key)
                    .and_then(|x| x.as_array())
                    .map(|a| {
                        a.iter()
                            .filter_map(|e| serde_json::from_value::<KV>(e.clone()).ok())
                            .collect()
                    })
                    .unwrap_or_default()
            };
            UrlSpec {
                raw: m
                    .get("raw")
                    .and_then(|x| x.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
                host: str_vec("host"),
                path: str_vec("path"),
                query: kv_vec("query"),
                variable: kv_vec("variable"),
                hash: m.get("hash").and_then(|x| x.as_str()).map(|s| s.to_string()),
            }
        }
        _ => UrlSpec::default(),
    })
}

impl Serialize for UrlSpec {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("raw", &self.raw)?;
        if !self.host.is_empty() {
            map.serialize_entry("host", &self.host)?;
        }
        if !self.path.is_empty() {
            map.serialize_entry("path", &self.path)?;
        }
        if !self.query.is_empty() {
            map.serialize_entry("query", &self.query)?;
        }
        if !self.variable.is_empty() {
            map.serialize_entry("variable", &self.variable)?;
        }
        if let Some(h) = &self.hash {
            if !h.is_empty() {
                map.serialize_entry("hash", h)?;
            }
        }
        map.end()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Header {
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct KV {
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub value: String,
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub typ: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub src: Option<Value>,
    #[serde(default)]
    pub disabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileRef {
    #[serde(default)]
    pub src: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GraphQL {
    #[serde(default)]
    pub query: String,
    #[serde(default)]
    pub variables: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Body {
    #[serde(default)]
    pub mode: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub urlencoded: Vec<KV>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub formdata: Vec<KV>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<FileRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graphql: Option<GraphQL>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<Value>,
}

impl Body {
    /// Best-effort content type for raw bodies.
    pub fn raw_content_type(&self) -> Option<String> {
        let lang = self
            .options
            .as_ref()
            .and_then(|o| o.get("raw"))
            .and_then(|r| r.get("language"))
            .and_then(|l| l.as_str())
            .unwrap_or("text");
        let ct = match lang {
            "json" => "application/json",
            "xml" => "application/xml",
            "html" => "text/html",
            "javascript" => "application/javascript",
            "text" => "text/plain",
            other => {
                let _ = other;
                "text/plain"
            }
        };
        Some(ct.to_string())
    }
}

/// Postman auth object: `{ "type": "bearer", "bearer": [{"key":"token","value":"..."}] }`
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Auth {
    #[serde(rename = "type", default)]
    pub kind: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bearer: Vec<KV>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub basic: Vec<KV>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub apikey: Vec<KV>,
    #[serde(flatten, default)]
    pub extra: serde_json::Map<String, Value>,
}

impl Auth {
    fn kv<'a>(list: &'a [KV], key: &str) -> Option<&'a str> {
        list.iter()
            .find(|kv| kv.key.eq_ignore_ascii_case(key) && !kv.disabled)
            .map(|kv| kv.value.as_str())
    }
    pub fn bearer_token(&self) -> Option<&str> {
        Self::kv(&self.bearer, "token")
    }
    pub fn basic_user(&self) -> Option<&str> {
        Self::kv(&self.basic, "username")
    }
    pub fn basic_pass(&self) -> Option<&str> {
        Self::kv(&self.basic, "password")
    }
    pub fn apikey_key(&self) -> Option<&str> {
        Self::kv(&self.apikey, "key")
    }
    pub fn apikey_value(&self) -> Option<&str> {
        Self::kv(&self.apikey, "value")
    }
    pub fn apikey_in(&self) -> &str {
        Self::kv(&self.apikey, "in").unwrap_or("header")
    }
    /// Extract OAuth2 fields from the flattened `oauth2` array (Postman stores it as KV list).
    pub fn oauth2_params(&self) -> std::collections::HashMap<String, String> {
        let mut m = std::collections::HashMap::new();
        if let Some(Value::Array(arr)) = self.extra.get("oauth2") {
            for item in arr {
                if let (Some(k), Some(v)) = (
                    item.get("key").and_then(|x| x.as_str()),
                    item.get("value"),
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
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Script {
    #[serde(default)]
    pub exec: Vec<String>,
    #[serde(rename = "type", default)]
    pub kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Event {
    #[serde(default)]
    pub listen: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script: Option<Script>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Variable {
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub value: Value,
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub typ: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Environment {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub values: Vec<Variable>,
    #[serde(rename = "_postman_variable_scope", default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

/// Declarative assertion, evaluated by `tests_engine`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TestAssertion {
    #[serde(rename = "type", default)]
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contains: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub equals: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_time_ms: Option<u64>,
}

impl Variable {
    pub fn is_enabled(&self) -> bool {
        self.enabled.unwrap_or(true)
    }
}

#[cfg(test)]
mod url_tests {
    use super::*;

    #[test]
    fn effective_url_applies_path_vars_disabled_query_and_hash() {
        let u = UrlSpec {
            raw: "https://api.test/users/:id?keep=1&drop=2".into(),
            query: vec![
                KV { key: "keep".into(), value: "1".into(), ..Default::default() },
                KV { key: "drop".into(), value: "2".into(), disabled: true, ..Default::default() },
            ],
            variable: vec![KV { key: "id".into(), value: "42".into(), ..Default::default() }],
            hash: Some("sec".into()),
            ..Default::default()
        };
        let eff = u.effective_url();
        assert_eq!(eff, "https://api.test/users/42?keep=1#sec");
    }

    #[test]
    fn url_object_roundtrip_preserves_structure() {
        let json = r#"{"method":"GET","url":{"raw":"https://h/x?a=1","query":[{"key":"a","value":"1"},{"key":"b","value":"2","disabled":true}],"variable":[{"key":"id","value":"7"}],"hash":"frag"}}"#;
        let req: Request = serde_json::from_str(json).unwrap();
        let u: UrlSpec = req.url;
        assert_eq!(u.query.len(), 2);
        assert!(u.query[1].disabled);
        assert_eq!(u.variable[0].value, "7");
        assert_eq!(u.hash.as_deref(), Some("frag"));
        let out = serde_json::to_string(&Request { url: u, ..Default::default() }).unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["url"]["query"][1]["disabled"], true);
    }
}
