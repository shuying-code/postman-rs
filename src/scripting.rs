//! Minimal Postman-style `pm.*` script support (pre-request + tests) without an
//! external JS runtime. Supports the most common patterns:
//!   pm.environment.set / pm.globals.set / pm.collectionVariables.set / pm.variables.set
//!   pm.test("name", function () { ... })
//!   pm.response.to.have.status(200)
//!   pm.expect(pm.response.code).to.eql(200)
//!   pm.expect(pm.response.json().a.b).to.eql("x")
//! Values: pm.response.code / .status / .responseTime / .json() / .text()
use crate::http::ResponseRecord;
use crate::model::Event;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub error: Option<String>,
}

#[derive(Default)]
pub struct ScriptOutput {
    pub tests: Vec<TestResult>,
    /// (scope, key, value) writes to apply to the variable store.
    pub sets: Vec<(String, String, String)>,
}

/// Extra context available to the script evaluator so `pm.*` getters work.
#[derive(Default, Clone)]
pub struct ScriptEnv {
    /// Merged variable scopes (variables / environment / collection / globals).
    pub vars: HashMap<String, String>,
    /// Current data-file iteration row.
    pub iter: HashMap<String, String>,
    /// Current request method/url (for `pm.request.method` / `pm.request.url`).
    pub method: String,
    pub url: String,
}

impl ScriptEnv {
    pub fn empty() -> Self {
        ScriptEnv::default()
    }
}

fn matching(s: &str, open: usize, oc: char, cc: char) -> Option<usize> {
    let mut depth = 0i32;
    let mut in_q: Option<char> = None;
    for (i, c) in s.char_indices().skip_while(|(i, _)| *i < open) {
        if let Some(q) = in_q {
            if c == q {
                in_q = None;
            }
            continue;
        }
        match c {
            '\'' | '"' => in_q = Some(c),
            x if x == oc => depth += 1,
            x if x == cc => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

fn unquote(s: &str) -> String {
    let t = s.trim();
    if t.len() >= 2
        && ((t.starts_with('"') && t.ends_with('"'))
            || (t.starts_with('\'') && t.ends_with('\'')))
    {
        t[1..t.len() - 1].to_string()
    } else {
        t.to_string()
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Val {
    Num(f64),
    Str(String),
    Bool(bool),
    Json(Value),
}

fn json_path(v: &Value, path: &str) -> Value {
    let mut cur = v.clone();
    for seg in path.split('.').filter(|s| !s.is_empty()) {
        let (name, mut rest) = match seg.split_once('[') {
            Some((n, r)) => (n, Some(r.trim_end_matches(']').to_string())),
            None => (seg, None),
        };
        if !name.is_empty() {
            cur = cur.get(name).cloned().unwrap_or(Value::Null);
        }
        while let Some(idx) = rest.take() {
            if let Ok(i) = idx.parse::<usize>() {
                cur = cur.get(i).cloned().unwrap_or(Value::Null);
            }
            break;
        }
    }
    cur
}

fn eval(expr: &str, resp: &ResponseRecord, env: &ScriptEnv) -> Val {
    let e = expr.trim();
    if let Some(rest) = e.strip_prefix("pm.response.json()") {
        let body: Value = serde_json::from_str(&resp.body).unwrap_or(Value::Null);
        let path = rest.trim_start_matches('.');
        let v = json_path(&body, path);
        return match v {
            Value::String(s) => Val::Str(s),
            Value::Number(n) => Val::Num(n.as_f64().unwrap_or(0.0)),
            Value::Bool(b) => Val::Bool(b),
            other => Val::Json(other),
        };
    }
    // pm.response.headers.get("Name")
    if let Some(rest) = e.strip_prefix("pm.response.headers.get(") {
        if let Some(end) = rest.rfind(')') {
            let name = unquote(&rest[..end]);
            for (k, v) in &resp.headers {
                if k.eq_ignore_ascii_case(&name) {
                    return Val::Str(v.clone());
                }
            }
            return Val::Str(String::new());
        }
    }
    // pm.<scope>.get("key") — variables / environment / globals / collectionVariables
    for scope in ["variables", "environment", "globals", "collectionVariables"] {
        let prefix = format!("pm.{}.get(", scope);
        if let Some(rest) = e.strip_prefix(&prefix) {
            if let Some(end) = rest.rfind(')') {
                let key = unquote(&rest[..end]);
                return Val::Str(env.vars.get(&key).cloned().unwrap_or_default());
            }
        }
    }
    // pm.iterationData.get("key")
    if let Some(rest) = e.strip_prefix("pm.iterationData.get(") {
        if let Some(end) = rest.rfind(')') {
            let key = unquote(&rest[..end]);
            return Val::Str(env.iter.get(&key).cloned().unwrap_or_default());
        }
    }
    if e == "pm.request.method" {
        return Val::Str(env.method.clone());
    }
    if e == "pm.request.url" {
        return Val::Str(env.url.clone());
    }
    match e {
        "pm.response.code" => return Val::Num(resp.code as f64),
        "pm.response.status" => return Val::Str(resp.status.clone()),
        "pm.response.responseTime" => return Val::Num(resp.elapsed_ms as f64),
        "pm.response.text()" => return Val::Str(resp.body.clone()),
        _ => {}
    }
    if e == "true" {
        return Val::Bool(true);
    }
    if e == "false" {
        return Val::Bool(false);
    }
    if let Ok(n) = e.parse::<f64>() {
        return Val::Num(n);
    }
    if e.starts_with('"') || e.starts_with('\'') {
        return Val::Str(unquote(e));
    }
    Val::Str(e.to_string())
}

fn eq(a: &Val, b: &Val) -> bool {
    match (a, b) {
        (Val::Num(x), Val::Num(y)) => (x - y).abs() < f64::EPSILON,
        (Val::Str(x), Val::Str(y)) => x == y,
        (Val::Bool(x), Val::Bool(y)) => x == y,
        (Val::Json(x), Val::Json(y)) => x == y,
        (Val::Str(x), Val::Num(y)) | (Val::Num(y), Val::Str(x)) => {
            x.parse::<f64>().map(|n| (n - y).abs() < f64::EPSILON).unwrap_or(false)
        }
        (Val::Num(x), Val::Json(y)) | (Val::Json(y), Val::Num(x)) => y
            .as_f64()
            .map(|n| (n - x).abs() < f64::EPSILON)
            .unwrap_or(false),
        _ => false,
    }
}

fn as_num(v: &Val) -> Option<f64> {
    match v {
        Val::Num(n) => Some(*n),
        Val::Str(s) => s.parse().ok(),
        _ => None,
    }
}

fn as_str(v: &Val) -> String {
    match v {
        Val::Num(n) => n.to_string(),
        Val::Str(s) => s.clone(),
        Val::Bool(b) => b.to_string(),
        Val::Json(j) => j.to_string(),
    }
}

fn val_from_json(v: &Value) -> Val {
    match v {
        Value::String(s) => Val::Str(s.clone()),
        Value::Number(n) => Val::Num(n.as_f64().unwrap_or(0.0)),
        Value::Bool(b) => Val::Bool(*b),
        other => Val::Json(other.clone()),
    }
}

fn lhs_json(v: &Val) -> Value {
    match v {
        Val::Json(j) => j.clone(),
        Val::Num(n) => serde_json::json!(n),
        Val::Bool(b) => serde_json::json!(b),
        Val::Str(s) => serde_json::json!(s),
    }
}

fn val_len(v: &Val) -> Option<usize> {
    match v {
        Val::Str(s) => Some(s.chars().count()),
        Val::Json(Value::Array(a)) => Some(a.len()),
        Val::Json(Value::Object(o)) => Some(o.len()),
        _ => None,
    }
}

fn type_of(v: &Val) -> String {
    match v {
        Val::Str(_) => "string",
        Val::Num(_) => "number",
        Val::Bool(_) => "boolean",
        Val::Json(Value::Null) => "null",
        Val::Json(Value::Array(_)) => "array",
        Val::Json(Value::Object(_)) => "object",
        Val::Json(_) => "object",
    }
    .to_string()
}

/// `include`/`contain` over strings (substring) and arrays (membership).
fn includes(lhs: &Val, needle: &Val) -> bool {
    match lhs {
        Val::Str(s) => s.contains(&as_str(needle)),
        Val::Json(Value::Array(a)) => a.iter().any(|x| eq(&val_from_json(x), needle)),
        _ => false,
    }
}

/// `match(/regex/)` against a value rendered as a string.
fn match_regex(lhs: &Val, args_text: &str) -> bool {
    let t = args_text.trim();
    let pat = if t.len() >= 2 && t.starts_with('/') {
        let end = t.rfind('/').unwrap_or(t.len() - 1);
        t[1..end].to_string()
    } else {
        unquote(t)
    };
    regex::Regex::new(&pat)
        .map(|re| re.is_match(&as_str(lhs)))
        .unwrap_or(false)
}

/// Evaluate one assertion statement; returns Ok(()) if it passes, Err(msg) otherwise.
fn eval_assert(stmt: &str, resp: &ResponseRecord, env: &ScriptEnv) -> Option<Result<(), String>> {
    let s = stmt.trim();
    // pm.response.to.have.status(N)
    if let Some(idx) = s.find("pm.response.to.have.status(") {
        let open = s[idx..].find('(').map(|o| idx + o)?;
        let close = matching(s, open, '(', ')')?;
        let n: u16 = s[open + 1..close].trim().parse().ok()?;
        return Some(if resp.code == n {
            Ok(())
        } else {
            Err(format!("期望状态 {}，实际 {}", n, resp.code))
        });
    }
    // pm.expect(EXPR).to.xxx(VALUE)
    if let Some(idx) = s.find("pm.expect(") {
        let open = s[idx..].find('(').map(|o| idx + o)?;
        let close = matching(s, open, '(', ')')?;
        let expr = &s[open + 1..close];
        let lhs = eval(expr, resp, env);
        let raw = s[close + 1..].trim();
        // 去掉前导 "." 与 "to." 修饰，识别链式修饰符 have./be. 与 .not 取反。
        let mut chain = raw.trim_start_matches('.');
        if let Some(r) = chain.strip_prefix("to.") {
            chain = r;
        }
        let negate = chain.starts_with("not.") || chain.contains(".not.");
        let mut cleaned = chain.replace(".not.", ".");
        if let Some(r) = cleaned.strip_prefix("not.") {
            cleaned = r.to_string();
        }
        let (matcher_chain, args_text) = if let Some(o) = cleaned.find('(') {
            let c = matching(&cleaned, o, '(', ')')?;
            (
                cleaned[..o].trim().trim_end_matches('.').to_string(),
                cleaned[o + 1..c].to_string(),
            )
        } else {
            (cleaned.trim_end_matches('.').to_string(), String::new())
        };
        let matcher = matcher_chain.rsplit('.').next().unwrap_or("").to_string();
        let args: Vec<&str> = if args_text.trim().is_empty() {
            Vec::new()
        } else {
            split_args(&args_text)
        };
        let arg_val = |i: usize| -> Val {
            args.get(i)
                .map(|a| eval(a, resp, env))
                .unwrap_or(Val::Bool(true))
        };
        let mut ok = match matcher.as_str() {
            "eql" | "equal" | "equals" | "eq" => eq(&lhs, &arg_val(0)),
            "above" | "greaterThan" | "gt" => as_num(&lhs)
                .zip(as_num(&arg_val(0)))
                .map(|(a, b)| a > b)
                .unwrap_or(false),
            "below" | "lessThan" | "lt" => as_num(&lhs)
                .zip(as_num(&arg_val(0)))
                .map(|(a, b)| a < b)
                .unwrap_or(false),
            "least" | "gte" | "atLeast" => as_num(&lhs)
                .zip(as_num(&arg_val(0)))
                .map(|(a, b)| a >= b)
                .unwrap_or(false),
            "most" | "lte" | "atMost" => as_num(&lhs)
                .zip(as_num(&arg_val(0)))
                .map(|(a, b)| a <= b)
                .unwrap_or(false),
            "closeTo" | "approximately" => {
                match (as_num(&lhs), as_num(&arg_val(0)), as_num(&arg_val(1))) {
                    (Some(a), Some(b), Some(d)) => (a - b).abs() <= d,
                    _ => false,
                }
            }
            "include" | "contain" | "contains" | "includes" => includes(&lhs, &arg_val(0)),
            "lengthOf" | "length" => val_len(&lhs)
                .zip(as_num(&arg_val(0)))
                .map(|(l, n)| l as f64 == n)
                .unwrap_or(false),
            "property" => {
                let name = as_str(&arg_val(0));
                match lhs_json(&lhs).get(name.as_str()).cloned() {
                    Some(v) => {
                        if args.len() >= 2 {
                            eq(&val_from_json(&v), &arg_val(1))
                        } else {
                            !v.is_null()
                        }
                    }
                    None => false,
                }
            }
            "a" | "an" => type_of(&lhs) == as_str(&arg_val(0)).to_ascii_lowercase(),
            "match" => match_regex(&lhs, &args_text),
            "oneOf" => serde_json::from_str::<Value>(args_text.trim())
                .ok()
                .and_then(|v| v.as_array().cloned())
                .map(|arr| arr.iter().any(|x| eq(&lhs, &val_from_json(x))))
                .unwrap_or(false),
            "empty" => val_len(&lhs).map(|l| l == 0).unwrap_or(false),
            "null" => matches!(lhs_json(&lhs), Value::Null),
            "exist" => !matches!(lhs_json(&lhs), Value::Null),
            "true" => matches!(lhs, Val::Bool(true)),
            "false" => matches!(lhs, Val::Bool(false)),
            "ok" => match &lhs {
                Val::Bool(b) => *b,
                Val::Num(n) => *n != 0.0,
                Val::Str(s) => !s.is_empty(),
                Val::Json(j) => !j.is_null(),
            },
            _ => false,
        };
        if negate {
            ok = !ok;
        }
        return Some(if ok {
            Ok(())
        } else {
            Err(format!(
                "断言失败：{} {}{} {}",
                expr.trim(),
                if negate { "not " } else { "" },
                matcher,
                args_text.trim()
            ))
        });
    }
    None
}

fn split_statements(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_q: Option<char> = None;
    let mut depth = 0i32;
    let chars: Vec<char> = body.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if let Some(q) = in_q {
            cur.push(c);
            if c == q {
                in_q = None;
            }
            i += 1;
            continue;
        }
        match c {
            '\'' | '"' => {
                in_q = Some(c);
                cur.push(c);
            }
            '{' | '(' | '[' => {
                depth += 1;
                cur.push(c);
            }
            '}' | ')' | ']' => {
                depth -= 1;
                cur.push(c);
            }
            ';' | '\n' if depth == 0 => {
                if !cur.trim().is_empty() {
                    out.push(cur.trim().to_string());
                }
                cur.clear();
            }
            _ => cur.push(c),
        }
        i += 1;
    }
    if !cur.trim().is_empty() {
        out.push(cur.trim().to_string());
    }
    out
}

fn process_sets(stmt: &str, out: &mut ScriptOutput) {
    for scope in [
        "environment",
        "globals",
        "collectionVariables",
        "variables",
    ] {
        let needle = format!("pm.{}.set(", scope);
        if let Some(idx) = stmt.find(&needle) {
            let open = idx + needle.len() - 1;
            if let Some(close) = matching(stmt, open, '(', ')') {
                let args = &stmt[open + 1..close];
                let parts: Vec<&str> = split_args(args);
                if parts.len() >= 2 {
                    let k = unquote(parts[0]);
                    let v = unquote(parts[1]);
                    out.sets.push((scope.to_string(), k, v));
                }
            }
        }
    }
}

fn split_args(args: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut in_q: Option<char> = None;
    let mut depth = 0i32;
    let mut start = 0usize;
    for (i, c) in args.char_indices() {
        if let Some(q) = in_q {
            if c == q {
                in_q = None;
            }
            continue;
        }
        match c {
            '\'' | '"' => in_q = Some(c),
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                out.push(args[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(args[start..].trim());
    out
}

/// Run a script that may contain `pm.test(...)` blocks and `pm.<scope>.set(...)`.
pub fn run(script: &str, resp: Option<&ResponseRecord>) -> ScriptOutput {
    run_env(script, resp, &ScriptEnv::empty())
}

/// Like [`run`] but exposes merged variables / iteration data to `pm.*` getters.
pub fn run_env(script: &str, resp: Option<&ResponseRecord>, env: &ScriptEnv) -> ScriptOutput {
    let mut out = ScriptOutput::default();
    let mut i = 0;
    let bytes = script;
    while let Some(rel) = bytes[i..].find("pm.test(") {
        let start = i + rel;
        let open = match bytes[start..].find('(') {
            Some(o) => start + o,
            None => break,
        };
        let close = match matching(bytes, open, '(', ')') {
            Some(c) => c,
            None => break,
        };
        let args = &bytes[open + 1..close];
        // name = first quoted string
        let name = args
            .split(',')
            .next()
            .map(unquote)
            .unwrap_or_else(|| "test".into());
        // body between first '{' and its match
        let body = if let Some(bopen_rel) = args.find('{') {
            let bopen = open + 1 + bopen_rel;
            matching(bytes, bopen, '{', '}')
                .map(|bclose| bytes[bopen + 1..bclose].to_string())
                .unwrap_or_default()
        } else {
            String::new()
        };
        let mut fails = Vec::new();
        for stmt in split_statements(&body) {
            if let Some(r) = resp.and_then(|r| eval_assert(&stmt, r, env)) {
                if let Err(e) = r {
                    fails.push(e);
                }
            } else {
                process_sets(&stmt, &mut out);
            }
        }
        out.tests.push(TestResult {
            name,
            passed: fails.is_empty(),
            error: if fails.is_empty() {
                None
            } else {
                Some(fails.join("; "))
            },
        });
        i = close + 1;
    }
    // Top-level set calls outside pm.test blocks.
    for stmt in split_statements(script) {
        process_sets(&stmt, &mut out);
    }
    out
}

/// Extract pre-request (`listen == "prerequest"`) and test (`listen == "test"`) scripts.
pub fn scripts_of(events: &[Event], listen: &str) -> String {
    let mut parts = Vec::new();
    for e in events {
        if e.listen.eq_ignore_ascii_case(listen) {
            if let Some(s) = &e.script {
                parts.push(s.exec.join("\n"));
            }
        }
    }
    parts.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resp(code: u16, body: &str) -> ResponseRecord {
        ResponseRecord {
            status: "OK".into(),
            code,
            headers: vec![],
            body: body.into(),
            binary: false,
            elapsed_ms: 12,
            final_url: "http://x".into(),
            dns_ms: None,
            tcp_ms: None,
            ttfb_ms: None,
            download_ms: None,
        }
    }

    #[test]
    fn status_and_expect() {
        let r = resp(200, r#"{"a":{"b":7},"name":"z"}"#);
        let s = r#"
            pm.test("状态为 200", function () {
                pm.response.to.have.status(200);
            });
            pm.test("code 是 200", function () {
                pm.expect(pm.response.code).to.eql(200);
            });
            pm.test("字段 b 是 7", function () {
                pm.expect(pm.response.json().a.b).to.eql(7);
            });
            pm.test("错误断言", function () {
                pm.expect(pm.response.code).to.eql(500);
            });
        "#;
        let out = run(s, Some(&r));
        assert_eq!(out.tests.len(), 4);
        assert!(out.tests[0].passed);
        assert!(out.tests[1].passed);
        assert!(out.tests[2].passed);
        assert!(!out.tests[3].passed);
    }

    #[test]
    fn pre_request_sets() {
        let s = r#"pm.environment.set("token", "abc"); pm.globals.set("g", "1");"#;
        let out = run(s, None);
        assert_eq!(out.sets.len(), 2);
        assert_eq!(out.sets[0].0, "environment");
        assert_eq!(out.sets[0].1, "token");
        assert_eq!(out.sets[0].2, "abc");
    }

    #[test]
    fn get_variables_iteration_and_headers() {
        let r = resp(200, "{}");
        let mut env = ScriptEnv::empty();
        env.vars.insert("token".into(), "abc".into());
        env.iter.insert("id".into(), "42".into());
        let mut r = r;
        r.headers.push(("Content-Type".into(), "application/json".into()));
        let s = r#"
            pm.test("token", function () { pm.expect(pm.variables.get("token")).to.eql("abc"); });
            pm.test("iter", function () { pm.expect(pm.iterationData.get("id")).to.eql("42"); });
            pm.test("ct", function () { pm.expect(pm.response.headers.get("content-type")).to.eql("application/json"); });
            pm.test("miss", function () { pm.expect(pm.variables.get("nope")).to.eql("x"); });
        "#;
        let out = run_env(s, Some(&r), &env);
        assert_eq!(out.tests.len(), 4);
        assert!(out.tests[0].passed);
        assert!(out.tests[1].passed);
        assert!(out.tests[2].passed);
        assert!(!out.tests[3].passed);
    }

    #[test]
    fn chai_matchers_not_property_length_match() {
        let r = resp(200, r#"{"name":"z","tags":[1,2,3],"age":30}"#);
        let s = r#"
            pm.test("not", function () { pm.expect(pm.response.code).to.not.eql(500); });
            pm.test("len", function () { pm.expect(pm.response.json().tags).to.have.lengthOf(3); });
            pm.test("prop", function () { pm.expect(pm.response.json()).to.have.property("age", 30); });
            pm.test("type", function () { pm.expect(pm.response.json().name).to.be.a("string"); });
            pm.test("match", function () { pm.expect(pm.response.json().name).to.match(/^z$/); });
            pm.test("oneof", function () { pm.expect(pm.response.json().age).to.be.oneOf([10,20,30]); });
            pm.test("close", function () { pm.expect(pm.response.json().age).to.be.closeTo(29, 1.5); });
            pm.test("include", function () { pm.expect(pm.response.json().tags).to.include(2); });
            pm.test("negfail", function () { pm.expect(pm.response.code).to.not.eql(200); });
        "#;
        let out = run(s, Some(&r));
        assert_eq!(out.tests.len(), 9);
        for t in &out.tests[..8] {
            assert!(t.passed, "{} failed: {:?}", t.name, t.error);
        }
        assert!(!out.tests[8].passed);
    }
}
