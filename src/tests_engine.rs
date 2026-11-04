//! Minimal, dependency-free assertion engine.
//!
//! Supports two sources:
//!  * declarative `tests` on a request (`TestAssertion`);
//!  * best-effort extraction of common `pm.test(... pm.response.to.have.status(N))` patterns.
use crate::http::ResponseRecord; // 引入依赖
use crate::model::{Event, TestAssertion}; // 引入依赖
use regex::Regex; // 引入依赖
use serde_json::Value; // 引入依赖

#[derive(Debug, Clone)] // 语句
pub struct TestResult { // 结构体 TestResult
    pub name: String, // 字段 name
    pub passed: bool, // 字段 passed
    pub detail: String, // 字段 detail
} // 块结束

/// Follow a simple dotted/bracket JSON path like `data.items[0].id`.
fn json_path<'a>(root: &'a Value, path: &str) -> Option<&'a Value> { // 函数 json_path
    let mut cur = root; // 变量 cur
    for raw_seg in path.split('.') { // 循环
        if raw_seg.is_empty() { // 条件分支
            continue; // 语句
        } // 块结束
        let mut seg = raw_seg; // 变量 seg
        // Handle trailing [index] pieces (possibly repeated).
        while let Some(bracket) = seg.find('[') { // 循环
            let name = &seg[..bracket]; // 变量 name
            if !name.is_empty() { // 条件分支
                cur = cur.get(name)?; // 语句
            } // 块结束
            let close = seg[bracket..].find(']')? + bracket; // 变量 close
            let idx: usize = seg[bracket + 1..close].parse().ok()?; // 变量 idx
            cur = cur.get(idx)?; // 语句
            seg = &seg[close + 1..]; // 语句
            if seg.is_empty() { // 条件分支
                break; // 语句
            } // 块结束
        } // 块结束
        if !seg.is_empty() { // 条件分支
            cur = cur.get(seg)?; // 语句
        } // 块结束
    } // 块结束
    Some(cur) // 调用 Some
} // 块结束

pub fn evaluate(assertions: &[TestAssertion], resp: &ResponseRecord) -> Vec<TestResult> { // 函数 evaluate
    let json_body: Option<Value> = serde_json::from_str(&resp.body).ok(); // 变量 json_body
    let mut results = Vec::new(); // 变量 results
    for a in assertions { // 循环
        let (name, passed, detail) = match a.kind.as_str() { // 调用 let
            "status" => { // 匹配分支
                let expected = a.equals.as_ref().and_then(|v| v.as_u64()).unwrap_or(200) as u16; // 变量 expected
                ( // 语句
                    format!("status == {}", expected), // 语句
                    resp.code == expected, // 语句
                    format!("got {}", resp.code), // 语句
                ) // 块结束
            } // 块结束
            "json_path" => { // 匹配分支
                let path = a.path.clone().unwrap_or_default(); // 变量 path
                let cur = json_body.as_ref().and_then(|j| json_path(j, &path)); // 变量 cur
                let want = a.equals.clone(); // 变量 want
                match (cur, want) { // 分支匹配
                    (Some(actual), Some(w)) => ( // 匹配分支
                        format!("json_path {} equals", path), // 语句
                        actual == &w, // 语句
                        format!("actual={} want={}", actual, w), // 语句
                    ), // 语句
                    (Some(actual), None) => ( // 匹配分支
                        format!("json_path {} exists", path), // 语句
                        true, // 语句
                        format!("actual={}", actual), // 语句
                    ), // 语句
                    (None, _) => ( // 匹配分支
                        format!("json_path {} exists", path), // 语句
                        false, // 语句
                        "path not found".to_string(), // 语句
                    ), // 语句
                } // 块结束
            } // 块结束
            "body_contains" => { // 匹配分支
                let needle = a // 变量 needle
                    .value // 语句
                    .as_ref() // 语句
                    .and_then(|v| v.as_str()) // 语句
                    .or(a.contains.as_deref()) // 语句
                    .unwrap_or(""); // 语句
                ( // 语句
                    format!("body contains {:?}", needle), // 语句
                    resp.body.contains(needle), // 语句
                    "substring check".to_string(), // 语句
                ) // 块结束
            } // 块结束
            "header" => { // 匹配分支
                let key = a.key.clone().unwrap_or_default(); // 变量 key
                let want = a // 变量 want
                    .value // 语句
                    .as_ref() // 语句
                    .and_then(|v| v.as_str()) // 语句
                    .map(|s| s.to_string()); // 语句
                let found = resp // 变量 found
                    .headers // 语句
                    .iter() // 语句
                    .find(|(k, _)| k.eq_ignore_ascii_case(&key)) // 语句
                    .map(|(_, v)| v.clone()); // 语句
                match (found, want) { // 分支匹配
                    (Some(v), Some(w)) => ( // 匹配分支
                        format!("header {} equals", key), // 语句
                        v.contains(&w), // 语句
                        format!("actual={}", v), // 语句
                    ), // 语句
                    (Some(v), None) => (format!("header {} exists", key), true, v), // 匹配分支
                    (None, _) => (format!("header {} exists", key), false, "missing".to_string()), // 匹配分支
                } // 块结束
            } // 块结束
            "max_time_ms" => { // 匹配分支
                let max = a.max_time_ms.unwrap_or(u64::MAX); // 变量 max
                ( // 语句
                    format!("elapsed <= {}ms", max), // 语句
                    (resp.elapsed_ms as u64) <= max, // 语句
                    format!("{}ms", resp.elapsed_ms), // 语句
                ) // 块结束
            } // 块结束
            "response_time" => { // 匹配分支
                let max = a // 变量 max
                    .max_time_ms // 语句
                    .or_else(|| a.equals.as_ref().and_then(|v| v.as_u64())) // 语句
                    .unwrap_or(u64::MAX); // 语句
                ( // 语句
                    format!("response_time <= {}ms", max), // 语句
                    (resp.elapsed_ms as u64) <= max, // 语句
                    format!("{}ms", resp.elapsed_ms), // 语句
                ) // 块结束
            } // 块结束
            "body_equals" => { // 匹配分支
                let want = a // 变量 want
                    .equals // 语句
                    .as_ref() // 语句
                    .map(|v| v.as_str().map(|s| s.to_string()).unwrap_or_else(|| v.to_string())) // 语句
                    .or_else(|| a.value.as_ref().and_then(|v| v.as_str()).map(|s| s.to_string())) // 语句
                    .unwrap_or_default(); // 语句
                ( // 语句
                    "body equals".to_string(), // 语句
                    resp.body == want, // 语句
                    format!("got {} bytes", resp.body.len()), // 语句
                ) // 块结束
            } // 块结束
            "regex" => { // 匹配分支
                let pat = a // 变量 pat
                    .contains // 语句
                    .as_deref() // 语句
                    .or_else(|| a.value.as_ref().and_then(|v| v.as_str())) // 语句
                    .unwrap_or(""); // 语句
                match Regex::new(pat) { // 分支匹配
                    Ok(re) => ( // 匹配分支
                        format!("body matches /{}/", pat), // 语句
                        re.is_match(&resp.body), // 语句
                        "regex".to_string(), // 语句
                    ), // 语句
                    Err(e) => ( // 匹配分支
                        format!("invalid regex /{}/", pat), // 语句
                        false, // 语句
                        e.to_string(), // 语句
                    ), // 语句
                } // 块结束
            } // 块结束
            "json_path_exists" => { // 匹配分支
                let path = a.path.clone().unwrap_or_default(); // 变量 path
                let ok = json_body.as_ref().and_then(|j| json_path(j, &path)).is_some(); // 变量 ok
                ( // 语句
                    format!("json_path {} exists", path), // 语句
                    ok, // 语句
                    if ok { "found".to_string() } else { "not found".to_string() }, // 条件分支
                ) // 块结束
            } // 块结束
            other => ( // 匹配分支
                format!("unknown assertion {}", other), // 语句
                false, // 语句
                "unknown assertion type".to_string(), // 语句
            ), // 语句
        }; // 块结束
        results.push(TestResult { // 语句
            name, // 语句
            passed, // 语句
            detail, // 语句
        }); // 语句
    } // 块结束
    results // 语句
} // 块结束

/// Best-effort conversion of common `pm.test` snippets into simple assertions.
pub fn extract_from_events(events: &[Event]) -> Vec<TestAssertion> { // 函数 extract_from_events
    let status_re = Regex::new(r"to\.have\.status\(\s*(\d{3})\s*\)").unwrap(); // 变量 status_re
    let header_re = Regex::new(r#"to\.have\.header\(\s*["']([^"']+)["']"#).unwrap(); // 变量 header_re
    let mut out = Vec::new(); // 变量 out
    for ev in events { // 循环
        if !ev.listen.eq_ignore_ascii_case("test") { // 条件分支
            continue; // 语句
        } // 块结束
        if let Some(script) = &ev.script { // 条件分支
            let joined = script.exec.join("\n"); // 变量 joined
            for cap in status_re.captures_iter(&joined) { // 循环
                if let Some(code) = cap.get(1).and_then(|m| m.as_str().parse::<u64>().ok()) { // 条件分支
                    out.push(TestAssertion { // 语句
                        kind: "status".to_string(), // 字段 kind
                        equals: Some(serde_json::json!(code)), // 字段 equals
                        ..Default::default() // 语句
                    }); // 语句
                } // 块结束
            } // 块结束
            for cap in header_re.captures_iter(&joined) { // 循环
                if let Some(name) = cap.get(1) { // 条件分支
                    out.push(TestAssertion { // 语句
                        kind: "header".to_string(), // 字段 kind
                        key: Some(name.as_str().to_string()), // 字段 key
                        ..Default::default() // 语句
                    }); // 语句
                } // 块结束
            } // 块结束
        } // 块结束
    } // 块结束
    out // 语句
} // 块结束

#[cfg(test)] // 语句
mod tests { // 模块 tests
    use super::*; // 引入依赖

    fn resp(code: u16, body: &str) -> ResponseRecord { // 函数 resp
        ResponseRecord { // 语句
            status: "OK".into(), // 字段 status
            code, // 语句
            headers: vec![("content-type".into(), "application/json".into())], // 字段 headers
            body: body.into(), // 字段 body
            binary: false, // 字段 binary
            elapsed_ms: 10, // 字段 elapsed_ms
            final_url: "http://x".into(), // 字段 final_url
            dns_ms: None, // 字段 dns_ms
            tcp_ms: None, // 字段 tcp_ms
            ttfb_ms: None, // 字段 ttfb_ms
            download_ms: None, // 字段 download_ms
        } // 块结束
    } // 块结束

    #[test] // 语句
    fn status_and_json_path() { // 函数 status_and_json_path
        let r = resp(200, r#"{"data":{"id":"abc","items":[{"v":1}]}}"#); // 变量 r
        let assertions = vec![ // 变量 assertions
            TestAssertion { // 语句
                kind: "status".into(), // 字段 kind
                equals: Some(serde_json::json!(200)), // 字段 equals
                ..Default::default() // 语句
            }, // 块结束
            TestAssertion { // 语句
                kind: "json_path".into(), // 字段 kind
                path: Some("data.id".into()), // 字段 path
                equals: Some(serde_json::json!("abc")), // 字段 equals
                ..Default::default() // 语句
            }, // 块结束
            TestAssertion { // 语句
                kind: "json_path".into(), // 字段 kind
                path: Some("data.items[0].v".into()), // 字段 path
                equals: Some(serde_json::json!(1)), // 字段 equals
                ..Default::default() // 语句
            }, // 块结束
        ]; // 语句
        let out = evaluate(&assertions, &r); // 变量 out
        assert!(out.iter().all(|t| t.passed)); // 语句
    } // 块结束

    #[test] // 语句
    fn extract_status_from_pm_test() { // 函数 extract_status_from_pm_test
        let ev = Event { // 变量 ev
            listen: "test".into(), // 字段 listen
            script: Some(crate::model::Script { // 语句
                exec: vec!["pm.test(\"ok\", () => pm.response.to.have.status(201));".into()], // 匹配分支
                kind: Some("text/javascript".into()), // 字段 kind
            }), // 语句
        }; // 块结束
        let a = extract_from_events(&[ev]); // 变量 a
        assert_eq!(a.len(), 1); // 语句
        assert_eq!(a[0].equals.as_ref().unwrap().as_u64().unwrap(), 201); // 语句
    } // 块结束
} // 块结束
