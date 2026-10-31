//! Minimal, dependency-free assertion engine.
//!
//! Supports two sources:
//!  * declarative `tests` on a request (`TestAssertion`);
//!  * best-effort extraction of common `pm.test(... pm.response.to.have.status(N))` patterns.
use crate::http::ResponseRecord;
use crate::model::{Event, TestAssertion};
use regex::Regex;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

/// Follow a simple dotted/bracket JSON path like `data.items[0].id`.
fn json_path<'a>(root: &'a Value, path: &str) -> Option<&'a Value> {
    let mut cur = root;
    for raw_seg in path.split('.') {
        if raw_seg.is_empty() {
            continue;
        }
        let mut seg = raw_seg;
        // Handle trailing [index] pieces (possibly repeated).
        while let Some(bracket) = seg.find('[') {
            let name = &seg[..bracket];
            if !name.is_empty() {
                cur = cur.get(name)?;
            }
            let close = seg[bracket..].find(']')? + bracket;
            let idx: usize = seg[bracket + 1..close].parse().ok()?;
            cur = cur.get(idx)?;
            seg = &seg[close + 1..];
            if seg.is_empty() {
                break;
            }
        }
        if !seg.is_empty() {
            cur = cur.get(seg)?;
        }
    }
    Some(cur)
}

pub fn evaluate(assertions: &[TestAssertion], resp: &ResponseRecord) -> Vec<TestResult> {
    let json_body: Option<Value> = serde_json::from_str(&resp.body).ok();
    let mut results = Vec::new();
    for a in assertions {
        let (name, passed, detail) = match a.kind.as_str() {
            "status" => {
                let expected = a.equals.as_ref().and_then(|v| v.as_u64()).unwrap_or(200) as u16;
                (
                    format!("status == {}", expected),
                    resp.code == expected,
                    format!("got {}", resp.code),
                )
            }
            "json_path" => {
                let path = a.path.clone().unwrap_or_default();
                let cur = json_body.as_ref().and_then(|j| json_path(j, &path));
                let want = a.equals.clone();
                match (cur, want) {
                    (Some(actual), Some(w)) => (
                        format!("json_path {} equals", path),
                        actual == &w,
                        format!("actual={} want={}", actual, w),
                    ),
                    (Some(actual), None) => (
                        format!("json_path {} exists", path),
                        true,
                        format!("actual={}", actual),
                    ),
                    (None, _) => (
                        format!("json_path {} exists", path),
                        false,
                        "path not found".to_string(),
                    ),
                }
            }
            "body_contains" => {
                let needle = a
                    .value
                    .as_ref()
                    .and_then(|v| v.as_str())
                    .or(a.contains.as_deref())
                    .unwrap_or("");
                (
                    format!("body contains {:?}", needle),
                    resp.body.contains(needle),
                    "substring check".to_string(),
                )
            }
            "header" => {
                let key = a.key.clone().unwrap_or_default();
                let want = a
                    .value
                    .as_ref()
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let found = resp
                    .headers
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case(&key))
                    .map(|(_, v)| v.clone());
                match (found, want) {
                    (Some(v), Some(w)) => (
                        format!("header {} equals", key),
                        v.contains(&w),
                        format!("actual={}", v),
                    ),
                    (Some(v), None) => (format!("header {} exists", key), true, v),
                    (None, _) => (format!("header {} exists", key), false, "missing".to_string()),
                }
            }
            "max_time_ms" => {
                let max = a.max_time_ms.unwrap_or(u64::MAX);
                (
                    format!("elapsed <= {}ms", max),
                    (resp.elapsed_ms as u64) <= max,
                    format!("{}ms", resp.elapsed_ms),
                )
            }
            "response_time" => {
                let max = a
                    .max_time_ms
                    .or_else(|| a.equals.as_ref().and_then(|v| v.as_u64()))
                    .unwrap_or(u64::MAX);
                (
                    format!("response_time <= {}ms", max),
                    (resp.elapsed_ms as u64) <= max,
                    format!("{}ms", resp.elapsed_ms),
                )
            }
            "body_equals" => {
                let want = a
                    .equals
                    .as_ref()
                    .map(|v| v.as_str().map(|s| s.to_string()).unwrap_or_else(|| v.to_string()))
                    .or_else(|| a.value.as_ref().and_then(|v| v.as_str()).map(|s| s.to_string()))
                    .unwrap_or_default();
                (
                    "body equals".to_string(),
                    resp.body == want,
                    format!("got {} bytes", resp.body.len()),
                )
            }
            "regex" => {
                let pat = a
                    .contains
                    .as_deref()
                    .or_else(|| a.value.as_ref().and_then(|v| v.as_str()))
                    .unwrap_or("");
                match Regex::new(pat) {
                    Ok(re) => (
                        format!("body matches /{}/", pat),
                        re.is_match(&resp.body),
                        "regex".to_string(),
                    ),
                    Err(e) => (
                        format!("invalid regex /{}/", pat),
                        false,
                        e.to_string(),
                    ),
                }
            }
            "json_path_exists" => {
                let path = a.path.clone().unwrap_or_default();
                let ok = json_body.as_ref().and_then(|j| json_path(j, &path)).is_some();
                (
                    format!("json_path {} exists", path),
                    ok,
                    if ok { "found".to_string() } else { "not found".to_string() },
                )
            }
            other => (
                format!("unknown assertion {}", other),
                false,
                "unknown assertion type".to_string(),
            ),
        };
        results.push(TestResult {
            name,
            passed,
            detail,
        });
    }
    results
}

/// Best-effort conversion of common `pm.test` snippets into simple assertions.
pub fn extract_from_events(events: &[Event]) -> Vec<TestAssertion> {
    let status_re = Regex::new(r"to\.have\.status\(\s*(\d{3})\s*\)").unwrap();
    let header_re = Regex::new(r#"to\.have\.header\(\s*["']([^"']+)["']"#).unwrap();
    let mut out = Vec::new();
    for ev in events {
        if !ev.listen.eq_ignore_ascii_case("test") {
            continue;
        }
        if let Some(script) = &ev.script {
            let joined = script.exec.join("\n");
            for cap in status_re.captures_iter(&joined) {
                if let Some(code) = cap.get(1).and_then(|m| m.as_str().parse::<u64>().ok()) {
                    out.push(TestAssertion {
                        kind: "status".to_string(),
                        equals: Some(serde_json::json!(code)),
                        ..Default::default()
                    });
                }
            }
            for cap in header_re.captures_iter(&joined) {
                if let Some(name) = cap.get(1) {
                    out.push(TestAssertion {
                        kind: "header".to_string(),
                        key: Some(name.as_str().to_string()),
                        ..Default::default()
                    });
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resp(code: u16, body: &str) -> ResponseRecord {
        ResponseRecord {
            status: "OK".into(),
            code,
            headers: vec![("content-type".into(), "application/json".into())],
            body: body.into(),
            binary: false,
            elapsed_ms: 10,
            final_url: "http://x".into(),
            dns_ms: None,
            tcp_ms: None,
            ttfb_ms: None,
            download_ms: None,
        }
    }

    #[test]
    fn status_and_json_path() {
        let r = resp(200, r#"{"data":{"id":"abc","items":[{"v":1}]}}"#);
        let assertions = vec![
            TestAssertion {
                kind: "status".into(),
                equals: Some(serde_json::json!(200)),
                ..Default::default()
            },
            TestAssertion {
                kind: "json_path".into(),
                path: Some("data.id".into()),
                equals: Some(serde_json::json!("abc")),
                ..Default::default()
            },
            TestAssertion {
                kind: "json_path".into(),
                path: Some("data.items[0].v".into()),
                equals: Some(serde_json::json!(1)),
                ..Default::default()
            },
        ];
        let out = evaluate(&assertions, &r);
        assert!(out.iter().all(|t| t.passed));
    }

    #[test]
    fn extract_status_from_pm_test() {
        let ev = Event {
            listen: "test".into(),
            script: Some(crate::model::Script {
                exec: vec!["pm.test(\"ok\", () => pm.response.to.have.status(201));".into()],
                kind: Some("text/javascript".into()),
            }),
        };
        let a = extract_from_events(&[ev]);
        assert_eq!(a.len(), 1);
        assert_eq!(a[0].equals.as_ref().unwrap().as_u64().unwrap(), 201);
    }
}
