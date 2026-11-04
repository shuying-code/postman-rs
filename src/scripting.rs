//! Minimal Postman-style `pm.*` script support (pre-request + tests) without an
//! external JS runtime. Supports the most common patterns:
//!   pm.environment.set / pm.globals.set / pm.collectionVariables.set / pm.variables.set
//!   pm.test("name", function () { ... })
//!   pm.response.to.have.status(200)
//!   pm.expect(pm.response.code).to.eql(200)
//!   pm.expect(pm.response.json().a.b).to.eql("x")
//! Values: pm.response.code / .status / .responseTime / .json() / .text()
use crate::http::ResponseRecord; // 引入依赖
use crate::model::Event; // 引入依赖
use serde_json::Value; // 引入依赖
use std::collections::HashMap; // 引入依赖

#[derive(Clone)] // 语句
pub struct TestResult { // 结构体 TestResult
    pub name: String, // 字段 name
    pub passed: bool, // 字段 passed
    pub error: Option<String>, // 字段 error
} // 块结束

#[derive(Default)] // 语句
pub struct ScriptOutput { // 结构体 ScriptOutput
    pub tests: Vec<TestResult>, // 字段 tests
    /// (scope, key, value) writes to apply to the variable store.
    pub sets: Vec<(String, String, String)>, // 字段 sets
} // 块结束

/// Extra context available to the script evaluator so `pm.*` getters work.
#[derive(Default, Clone)] // 语句
pub struct ScriptEnv { // 结构体 ScriptEnv
    /// Merged variable scopes (variables / environment / collection / globals).
    pub vars: HashMap<String, String>, // 字段 vars
    /// Current data-file iteration row.
    pub iter: HashMap<String, String>, // 字段 iter
    /// Current request method/url (for `pm.request.method` / `pm.request.url`).
    pub method: String, // 字段 method
    pub url: String, // 字段 url
} // 块结束

impl ScriptEnv { // 实现
    pub fn empty() -> Self { // 函数 empty
        ScriptEnv::default() // 调用 default
    } // 块结束
} // 块结束

fn matching(s: &str, open: usize, oc: char, cc: char) -> Option<usize> { // 函数 matching
    let mut depth = 0i32; // 变量 depth
    let mut in_q: Option<char> = None; // 变量 in_q
    for (i, c) in s.char_indices().skip_while(|(i, _)| *i < open) { // 循环
        if let Some(q) = in_q { // 条件分支
            if c == q { // 条件分支
                in_q = None; // 语句
            } // 块结束
            continue; // 语句
        } // 块结束
        match c { // 分支匹配
            '\'' | '"' => in_q = Some(c), // 匹配分支
            x if x == oc => depth += 1, // 匹配分支
            x if x == cc => { // 匹配分支
                depth -= 1; // 语句
                if depth == 0 { // 条件分支
                    return Some(i); // 返回
                } // 块结束
            } // 块结束
            _ => {} // 匹配分支
        } // 块结束
    } // 块结束
    None // 语句
} // 块结束

fn unquote(s: &str) -> String { // 函数 unquote
    let t = s.trim(); // 变量 t
    if t.len() >= 2 // 条件分支
        && ((t.starts_with('"') && t.ends_with('"')) // 语句
            || (t.starts_with('\'') && t.ends_with('\''))) // 语句
    { // 语句
        t[1..t.len() - 1].to_string() // 语句
    } else { // 语句
        t.to_string() // 语句
    } // 块结束
} // 块结束

#[derive(Debug, Clone, PartialEq)] // 语句
enum Val { // 枚举 Val
    Num(f64), // 调用 Num
    Str(String), // 调用 Str
    Bool(bool), // 调用 Bool
    Json(Value), // 调用 Json
} // 块结束

fn json_path(v: &Value, path: &str) -> Value { // 函数 json_path
    let mut cur = v.clone(); // 变量 cur
    for seg in path.split('.').filter(|s| !s.is_empty()) { // 循环
        let (name, mut rest) = match seg.split_once('[') { // 调用 let
            Some((n, r)) => (n, Some(r.trim_end_matches(']').to_string())), // 匹配分支
            None => (seg, None), // 匹配分支
        }; // 块结束
        if !name.is_empty() { // 条件分支
            cur = cur.get(name).cloned().unwrap_or(Value::Null); // 语句
        } // 块结束
        while let Some(idx) = rest.take() { // 循环
            if let Ok(i) = idx.parse::<usize>() { // 条件分支
                cur = cur.get(i).cloned().unwrap_or(Value::Null); // 语句
            } // 块结束
            break; // 语句
        } // 块结束
    } // 块结束
    cur // 语句
} // 块结束

fn eval(expr: &str, resp: &ResponseRecord, env: &ScriptEnv) -> Val { // 函数 eval
    let e = expr.trim(); // 变量 e
    if let Some(rest) = e.strip_prefix("pm.response.json()") { // 条件分支
        let body: Value = serde_json::from_str(&resp.body).unwrap_or(Value::Null); // 变量 body
        let path = rest.trim_start_matches('.'); // 变量 path
        let v = json_path(&body, path); // 变量 v
        return match v { // 返回
            Value::String(s) => Val::Str(s), // 匹配分支
            Value::Number(n) => Val::Num(n.as_f64().unwrap_or(0.0)), // 匹配分支
            Value::Bool(b) => Val::Bool(b), // 匹配分支
            other => Val::Json(other), // 匹配分支
        }; // 块结束
    } // 块结束
    // pm.response.headers.get("Name")
    if let Some(rest) = e.strip_prefix("pm.response.headers.get(") { // 条件分支
        if let Some(end) = rest.rfind(')') { // 条件分支
            let name = unquote(&rest[..end]); // 变量 name
            for (k, v) in &resp.headers { // 循环
                if k.eq_ignore_ascii_case(&name) { // 条件分支
                    return Val::Str(v.clone()); // 返回
                } // 块结束
            } // 块结束
            return Val::Str(String::new()); // 返回
        } // 块结束
    } // 块结束
    // pm.<scope>.get("key") — variables / environment / globals / collectionVariables
    for scope in ["variables", "environment", "globals", "collectionVariables"] { // 循环
        let prefix = format!("pm.{}.get(", scope); // 变量 prefix
        if let Some(rest) = e.strip_prefix(&prefix) { // 条件分支
            if let Some(end) = rest.rfind(')') { // 条件分支
                let key = unquote(&rest[..end]); // 变量 key
                return Val::Str(env.vars.get(&key).cloned().unwrap_or_default()); // 返回
            } // 块结束
        } // 块结束
    } // 块结束
    // pm.iterationData.get("key")
    if let Some(rest) = e.strip_prefix("pm.iterationData.get(") { // 条件分支
        if let Some(end) = rest.rfind(')') { // 条件分支
            let key = unquote(&rest[..end]); // 变量 key
            return Val::Str(env.iter.get(&key).cloned().unwrap_or_default()); // 返回
        } // 块结束
    } // 块结束
    if e == "pm.request.method" { // 条件分支
        return Val::Str(env.method.clone()); // 返回
    } // 块结束
    if e == "pm.request.url" { // 条件分支
        return Val::Str(env.url.clone()); // 返回
    } // 块结束
    match e { // 分支匹配
        "pm.response.code" => return Val::Num(resp.code as f64), // 匹配分支
        "pm.response.status" => return Val::Str(resp.status.clone()), // 匹配分支
        "pm.response.responseTime" => return Val::Num(resp.elapsed_ms as f64), // 匹配分支
        "pm.response.text()" => return Val::Str(resp.body.clone()), // 匹配分支
        _ => {} // 匹配分支
    } // 块结束
    if e == "true" { // 条件分支
        return Val::Bool(true); // 返回
    } // 块结束
    if e == "false" { // 条件分支
        return Val::Bool(false); // 返回
    } // 块结束
    if let Ok(n) = e.parse::<f64>() { // 条件分支
        return Val::Num(n); // 返回
    } // 块结束
    if e.starts_with('"') || e.starts_with('\'') { // 条件分支
        return Val::Str(unquote(e)); // 返回
    } // 块结束
    Val::Str(e.to_string()) // 调用 Str
} // 块结束

fn eq(a: &Val, b: &Val) -> bool { // 函数 eq
    match (a, b) { // 分支匹配
        (Val::Num(x), Val::Num(y)) => (x - y).abs() < f64::EPSILON, // 匹配分支
        (Val::Str(x), Val::Str(y)) => x == y, // 匹配分支
        (Val::Bool(x), Val::Bool(y)) => x == y, // 匹配分支
        (Val::Json(x), Val::Json(y)) => x == y, // 匹配分支
        (Val::Str(x), Val::Num(y)) | (Val::Num(y), Val::Str(x)) => { // 匹配分支
            x.parse::<f64>().map(|n| (n - y).abs() < f64::EPSILON).unwrap_or(false) // 语句
        } // 块结束
        (Val::Num(x), Val::Json(y)) | (Val::Json(y), Val::Num(x)) => y // 匹配分支
            .as_f64() // 语句
            .map(|n| (n - x).abs() < f64::EPSILON) // 语句
            .unwrap_or(false), // 语句
        _ => false, // 匹配分支
    } // 块结束
} // 块结束

fn as_num(v: &Val) -> Option<f64> { // 函数 as_num
    match v { // 分支匹配
        Val::Num(n) => Some(*n), // 匹配分支
        Val::Str(s) => s.parse().ok(), // 匹配分支
        _ => None, // 匹配分支
    } // 块结束
} // 块结束

fn as_str(v: &Val) -> String { // 函数 as_str
    match v { // 分支匹配
        Val::Num(n) => n.to_string(), // 匹配分支
        Val::Str(s) => s.clone(), // 匹配分支
        Val::Bool(b) => b.to_string(), // 匹配分支
        Val::Json(j) => j.to_string(), // 匹配分支
    } // 块结束
} // 块结束

fn val_from_json(v: &Value) -> Val { // 函数 val_from_json
    match v { // 分支匹配
        Value::String(s) => Val::Str(s.clone()), // 匹配分支
        Value::Number(n) => Val::Num(n.as_f64().unwrap_or(0.0)), // 匹配分支
        Value::Bool(b) => Val::Bool(*b), // 匹配分支
        other => Val::Json(other.clone()), // 匹配分支
    } // 块结束
} // 块结束

fn lhs_json(v: &Val) -> Value { // 函数 lhs_json
    match v { // 分支匹配
        Val::Json(j) => j.clone(), // 匹配分支
        Val::Num(n) => serde_json::json!(n), // 匹配分支
        Val::Bool(b) => serde_json::json!(b), // 匹配分支
        Val::Str(s) => serde_json::json!(s), // 匹配分支
    } // 块结束
} // 块结束

fn val_len(v: &Val) -> Option<usize> { // 函数 val_len
    match v { // 分支匹配
        Val::Str(s) => Some(s.chars().count()), // 匹配分支
        Val::Json(Value::Array(a)) => Some(a.len()), // 匹配分支
        Val::Json(Value::Object(o)) => Some(o.len()), // 匹配分支
        _ => None, // 匹配分支
    } // 块结束
} // 块结束

fn type_of(v: &Val) -> String { // 函数 type_of
    match v { // 分支匹配
        Val::Str(_) => "string", // 匹配分支
        Val::Num(_) => "number", // 匹配分支
        Val::Bool(_) => "boolean", // 匹配分支
        Val::Json(Value::Null) => "null", // 匹配分支
        Val::Json(Value::Array(_)) => "array", // 匹配分支
        Val::Json(Value::Object(_)) => "object", // 匹配分支
        Val::Json(_) => "object", // 匹配分支
    } // 块结束
    .to_string() // 语句
} // 块结束

/// `include`/`contain` over strings (substring) and arrays (membership).
fn includes(lhs: &Val, needle: &Val) -> bool { // 函数 includes
    match lhs { // 分支匹配
        Val::Str(s) => s.contains(&as_str(needle)), // 匹配分支
        Val::Json(Value::Array(a)) => a.iter().any(|x| eq(&val_from_json(x), needle)), // 匹配分支
        _ => false, // 匹配分支
    } // 块结束
} // 块结束

/// `match(/regex/)` against a value rendered as a string.
fn match_regex(lhs: &Val, args_text: &str) -> bool { // 函数 match_regex
    let t = args_text.trim(); // 变量 t
    let pat = if t.len() >= 2 && t.starts_with('/') { // 变量 pat
        let end = t.rfind('/').unwrap_or(t.len() - 1); // 变量 end
        t[1..end].to_string() // 语句
    } else { // 语句
        unquote(t) // 调用 unquote
    }; // 块结束
    regex::Regex::new(&pat) // 调用 new
        .map(|re| re.is_match(&as_str(lhs))) // 语句
        .unwrap_or(false) // 语句
} // 块结束

/// Evaluate one assertion statement; returns Ok(()) if it passes, Err(msg) otherwise.
fn eval_assert(stmt: &str, resp: &ResponseRecord, env: &ScriptEnv) -> Option<Result<(), String>> { // 函数 eval_assert
    let s = stmt.trim(); // 变量 s
    // pm.response.to.have.status(N)
    if let Some(idx) = s.find("pm.response.to.have.status(") { // 条件分支
        let open = s[idx..].find('(').map(|o| idx + o)?; // 变量 open
        let close = matching(s, open, '(', ')')?; // 变量 close
        let n: u16 = s[open + 1..close].trim().parse().ok()?; // 变量 n
        return Some(if resp.code == n { // 返回
            Ok(()) // 调用 Ok
        } else { // 语句
            Err(format!("期望状态 {}，实际 {}", n, resp.code)) // 调用 Err
        }); // 语句
    } // 块结束
    // pm.expect(EXPR).to.xxx(VALUE)
    if let Some(idx) = s.find("pm.expect(") { // 条件分支
        let open = s[idx..].find('(').map(|o| idx + o)?; // 变量 open
        let close = matching(s, open, '(', ')')?; // 变量 close
        let expr = &s[open + 1..close]; // 变量 expr
        let lhs = eval(expr, resp, env); // 变量 lhs
        let raw = s[close + 1..].trim(); // 变量 raw
        // 去掉前导 "." 与 "to." 修饰，识别链式修饰符 have./be. 与 .not 取反。
        let mut chain = raw.trim_start_matches('.'); // 变量 chain
        if let Some(r) = chain.strip_prefix("to.") { // 条件分支
            chain = r; // 语句
        } // 块结束
        let negate = chain.starts_with("not.") || chain.contains(".not."); // 变量 negate
        let mut cleaned = chain.replace(".not.", "."); // 变量 cleaned
        if let Some(r) = cleaned.strip_prefix("not.") { // 条件分支
            cleaned = r.to_string(); // 语句
        } // 块结束
        let (matcher_chain, args_text) = if let Some(o) = cleaned.find('(') { // 调用 let
            let c = matching(&cleaned, o, '(', ')')?; // 变量 c
            ( // 语句
                cleaned[..o].trim().trim_end_matches('.').to_string(), // 语句
                cleaned[o + 1..c].to_string(), // 语句
            ) // 块结束
        } else { // 语句
            (cleaned.trim_end_matches('.').to_string(), String::new()) // 语句
        }; // 块结束
        let matcher = matcher_chain.rsplit('.').next().unwrap_or("").to_string(); // 变量 matcher
        let args: Vec<&str> = if args_text.trim().is_empty() { // 变量 args
            Vec::new() // 调用 new
        } else { // 语句
            split_args(&args_text) // 调用 split_args
        }; // 块结束
        let arg_val = |i: usize| -> Val { // 变量 arg_val
            args.get(i) // 语句
                .map(|a| eval(a, resp, env)) // 语句
                .unwrap_or(Val::Bool(true)) // 语句
        }; // 块结束
        let mut ok = match matcher.as_str() { // 变量 ok
            "eql" | "equal" | "equals" | "eq" => eq(&lhs, &arg_val(0)), // 匹配分支
            "above" | "greaterThan" | "gt" => as_num(&lhs) // 匹配分支
                .zip(as_num(&arg_val(0))) // 语句
                .map(|(a, b)| a > b) // 语句
                .unwrap_or(false), // 语句
            "below" | "lessThan" | "lt" => as_num(&lhs) // 匹配分支
                .zip(as_num(&arg_val(0))) // 语句
                .map(|(a, b)| a < b) // 语句
                .unwrap_or(false), // 语句
            "least" | "gte" | "atLeast" => as_num(&lhs) // 匹配分支
                .zip(as_num(&arg_val(0))) // 语句
                .map(|(a, b)| a >= b) // 语句
                .unwrap_or(false), // 语句
            "most" | "lte" | "atMost" => as_num(&lhs) // 匹配分支
                .zip(as_num(&arg_val(0))) // 语句
                .map(|(a, b)| a <= b) // 语句
                .unwrap_or(false), // 语句
            "closeTo" | "approximately" => { // 匹配分支
                match (as_num(&lhs), as_num(&arg_val(0)), as_num(&arg_val(1))) { // 分支匹配
                    (Some(a), Some(b), Some(d)) => (a - b).abs() <= d, // 匹配分支
                    _ => false, // 匹配分支
                } // 块结束
            } // 块结束
            "include" | "contain" | "contains" | "includes" => includes(&lhs, &arg_val(0)), // 匹配分支
            "lengthOf" | "length" => val_len(&lhs) // 匹配分支
                .zip(as_num(&arg_val(0))) // 语句
                .map(|(l, n)| l as f64 == n) // 语句
                .unwrap_or(false), // 语句
            "property" => { // 匹配分支
                let name = as_str(&arg_val(0)); // 变量 name
                match lhs_json(&lhs).get(name.as_str()).cloned() { // 分支匹配
                    Some(v) => { // 匹配分支
                        if args.len() >= 2 { // 条件分支
                            eq(&val_from_json(&v), &arg_val(1)) // 调用 eq
                        } else { // 语句
                            !v.is_null() // 语句
                        } // 块结束
                    } // 块结束
                    None => false, // 匹配分支
                } // 块结束
            } // 块结束
            "a" | "an" => type_of(&lhs) == as_str(&arg_val(0)).to_ascii_lowercase(), // 匹配分支
            "match" => match_regex(&lhs, &args_text), // 匹配分支
            "oneOf" => serde_json::from_str::<Value>(args_text.trim()) // 匹配分支
                .ok() // 语句
                .and_then(|v| v.as_array().cloned()) // 语句
                .map(|arr| arr.iter().any(|x| eq(&lhs, &val_from_json(x)))) // 语句
                .unwrap_or(false), // 语句
            "empty" => val_len(&lhs).map(|l| l == 0).unwrap_or(false), // 匹配分支
            "null" => matches!(lhs_json(&lhs), Value::Null), // 匹配分支
            "exist" => !matches!(lhs_json(&lhs), Value::Null), // 匹配分支
            "true" => matches!(lhs, Val::Bool(true)), // 匹配分支
            "false" => matches!(lhs, Val::Bool(false)), // 匹配分支
            "ok" => match &lhs { // 匹配分支
                Val::Bool(b) => *b, // 匹配分支
                Val::Num(n) => *n != 0.0, // 匹配分支
                Val::Str(s) => !s.is_empty(), // 匹配分支
                Val::Json(j) => !j.is_null(), // 匹配分支
            }, // 块结束
            _ => false, // 匹配分支
        }; // 块结束
        if negate { // 条件分支
            ok = !ok; // 语句
        } // 块结束
        return Some(if ok { // 返回
            Ok(()) // 调用 Ok
        } else { // 语句
            Err(format!( // 调用 Err
                "断言失败：{} {}{} {}", // 语句
                expr.trim(), // 语句
                if negate { "not " } else { "" }, // 条件分支
                matcher, // 语句
                args_text.trim() // 语句
            )) // 语句
        }); // 语句
    } // 块结束
    None // 语句
} // 块结束

fn split_statements(body: &str) -> Vec<String> { // 函数 split_statements
    let mut out = Vec::new(); // 变量 out
    let mut cur = String::new(); // 变量 cur
    let mut in_q: Option<char> = None; // 变量 in_q
    let mut depth = 0i32; // 变量 depth
    let chars: Vec<char> = body.chars().collect(); // 变量 chars
    let mut i = 0; // 变量 i
    while i < chars.len() { // 循环
        let c = chars[i]; // 变量 c
        if let Some(q) = in_q { // 条件分支
            cur.push(c); // 语句
            if c == q { // 条件分支
                in_q = None; // 语句
            } // 块结束
            i += 1; // 语句
            continue; // 语句
        } // 块结束
        match c { // 分支匹配
            '\'' | '"' => { // 匹配分支
                in_q = Some(c); // 语句
                cur.push(c); // 语句
            } // 块结束
            '{' | '(' | '[' => { // 匹配分支
                depth += 1; // 语句
                cur.push(c); // 语句
            } // 块结束
            '}' | ')' | ']' => { // 匹配分支
                depth -= 1; // 语句
                cur.push(c); // 语句
            } // 块结束
            ';' | '\n' if depth == 0 => { // 匹配分支
                if !cur.trim().is_empty() { // 条件分支
                    out.push(cur.trim().to_string()); // 语句
                } // 块结束
                cur.clear(); // 语句
            } // 块结束
            _ => cur.push(c), // 匹配分支
        } // 块结束
        i += 1; // 语句
    } // 块结束
    if !cur.trim().is_empty() { // 条件分支
        out.push(cur.trim().to_string()); // 语句
    } // 块结束
    out // 语句
} // 块结束

fn process_sets(stmt: &str, out: &mut ScriptOutput) { // 函数 process_sets
    for scope in [ // 循环
        "environment", // 语句
        "globals", // 语句
        "collectionVariables", // 语句
        "variables", // 语句
    ] { // 语句
        let needle = format!("pm.{}.set(", scope); // 变量 needle
        if let Some(idx) = stmt.find(&needle) { // 条件分支
            let open = idx + needle.len() - 1; // 变量 open
            if let Some(close) = matching(stmt, open, '(', ')') { // 条件分支
                let args = &stmt[open + 1..close]; // 变量 args
                let parts: Vec<&str> = split_args(args); // 变量 parts
                if parts.len() >= 2 { // 条件分支
                    let k = unquote(parts[0]); // 变量 k
                    let v = unquote(parts[1]); // 变量 v
                    out.sets.push((scope.to_string(), k, v)); // 语句
                } // 块结束
            } // 块结束
        } // 块结束
    } // 块结束
} // 块结束

fn split_args(args: &str) -> Vec<&str> { // 函数 split_args
    let mut out = Vec::new(); // 变量 out
    let mut in_q: Option<char> = None; // 变量 in_q
    let mut depth = 0i32; // 变量 depth
    let mut start = 0usize; // 变量 start
    for (i, c) in args.char_indices() { // 循环
        if let Some(q) = in_q { // 条件分支
            if c == q { // 条件分支
                in_q = None; // 语句
            } // 块结束
            continue; // 语句
        } // 块结束
        match c { // 分支匹配
            '\'' | '"' => in_q = Some(c), // 匹配分支
            '(' | '[' | '{' => depth += 1, // 匹配分支
            ')' | ']' | '}' => depth -= 1, // 匹配分支
            ',' if depth == 0 => { // 匹配分支
                out.push(args[start..i].trim()); // 语句
                start = i + 1; // 语句
            } // 块结束
            _ => {} // 匹配分支
        } // 块结束
    } // 块结束
    out.push(args[start..].trim()); // 语句
    out // 语句
} // 块结束

/// Run a script that may contain `pm.test(...)` blocks and `pm.<scope>.set(...)`.
pub fn run(script: &str, resp: Option<&ResponseRecord>) -> ScriptOutput { // 函数 run
    run_env(script, resp, &ScriptEnv::empty()) // 调用 run_env
} // 块结束

/// Like [`run`] but exposes merged variables / iteration data to `pm.*` getters.
pub fn run_env(script: &str, resp: Option<&ResponseRecord>, env: &ScriptEnv) -> ScriptOutput { // 函数 run_env
    let mut out = ScriptOutput::default(); // 变量 out
    let mut i = 0; // 变量 i
    let bytes = script; // 变量 bytes
    while let Some(rel) = bytes[i..].find("pm.test(") { // 循环
        let start = i + rel; // 变量 start
        let open = match bytes[start..].find('(') { // 变量 open
            Some(o) => start + o, // 匹配分支
            None => break, // 匹配分支
        }; // 块结束
        let close = match matching(bytes, open, '(', ')') { // 变量 close
            Some(c) => c, // 匹配分支
            None => break, // 匹配分支
        }; // 块结束
        let args = &bytes[open + 1..close]; // 变量 args
        // name = first quoted string
        let name = args // 变量 name
            .split(',') // 语句
            .next() // 语句
            .map(unquote) // 语句
            .unwrap_or_else(|| "test".into()); // 语句
        // body between first '{' and its match
        let body = if let Some(bopen_rel) = args.find('{') { // 变量 body
            let bopen = open + 1 + bopen_rel; // 变量 bopen
            matching(bytes, bopen, '{', '}') // 调用 matching
                .map(|bclose| bytes[bopen + 1..bclose].to_string()) // 语句
                .unwrap_or_default() // 语句
        } else { // 语句
            String::new() // 调用 new
        }; // 块结束
        let mut fails = Vec::new(); // 变量 fails
        for stmt in split_statements(&body) { // 循环
            if let Some(r) = resp.and_then(|r| eval_assert(&stmt, r, env)) { // 条件分支
                if let Err(e) = r { // 条件分支
                    fails.push(e); // 语句
                } // 块结束
            } else { // 语句
                process_sets(&stmt, &mut out); // 调用 process_sets
            } // 块结束
        } // 块结束
        out.tests.push(TestResult { // 语句
            name, // 语句
            passed: fails.is_empty(), // 字段 passed
            error: if fails.is_empty() { // 语句
                None // 语句
            } else { // 语句
                Some(fails.join("; ")) // 调用 Some
            }, // 块结束
        }); // 语句
        i = close + 1; // 语句
    } // 块结束
    // Top-level set calls outside pm.test blocks.
    for stmt in split_statements(script) { // 循环
        process_sets(&stmt, &mut out); // 调用 process_sets
    } // 块结束
    out // 语句
} // 块结束

/// Extract pre-request (`listen == "prerequest"`) and test (`listen == "test"`) scripts.
pub fn scripts_of(events: &[Event], listen: &str) -> String { // 函数 scripts_of
    let mut parts = Vec::new(); // 变量 parts
    for e in events { // 循环
        if e.listen.eq_ignore_ascii_case(listen) { // 条件分支
            if let Some(s) = &e.script { // 条件分支
                parts.push(s.exec.join("\n")); // 语句
            } // 块结束
        } // 块结束
    } // 块结束
    parts.join("\n") // 语句
} // 块结束

#[cfg(test)] // 语句
mod tests { // 模块 tests
    use super::*; // 引入依赖

    fn resp(code: u16, body: &str) -> ResponseRecord { // 函数 resp
        ResponseRecord { // 语句
            status: "OK".into(), // 字段 status
            code, // 语句
            headers: vec![], // 字段 headers
            body: body.into(), // 字段 body
            binary: false, // 字段 binary
            elapsed_ms: 12, // 字段 elapsed_ms
            final_url: "http://x".into(), // 字段 final_url
            dns_ms: None, // 字段 dns_ms
            tcp_ms: None, // 字段 tcp_ms
            ttfb_ms: None, // 字段 ttfb_ms
            download_ms: None, // 字段 download_ms
        } // 块结束
    } // 块结束

    #[test] // 语句
    fn status_and_expect() { // 函数 status_and_expect
        let r = resp(200, r#"{"a":{"b":7},"name":"z"}"#); // 变量 r
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
        "#; // 语句
        let out = run(s, Some(&r)); // 变量 out
        assert_eq!(out.tests.len(), 4); // 语句
        assert!(out.tests[0].passed); // 语句
        assert!(out.tests[1].passed); // 语句
        assert!(out.tests[2].passed); // 语句
        assert!(!out.tests[3].passed); // 语句
    } // 块结束

    #[test] // 语句
    fn pre_request_sets() { // 函数 pre_request_sets
        let s = r#"pm.environment.set("token", "abc"); pm.globals.set("g", "1");"#; // 变量 s
        let out = run(s, None); // 变量 out
        assert_eq!(out.sets.len(), 2); // 语句
        assert_eq!(out.sets[0].0, "environment"); // 语句
        assert_eq!(out.sets[0].1, "token"); // 语句
        assert_eq!(out.sets[0].2, "abc"); // 语句
    } // 块结束

    #[test] // 语句
    fn get_variables_iteration_and_headers() { // 函数 get_variables_iteration_and_headers
        let r = resp(200, "{}"); // 变量 r
        let mut env = ScriptEnv::empty(); // 变量 env
        env.vars.insert("token".into(), "abc".into()); // 语句
        env.iter.insert("id".into(), "42".into()); // 语句
        let mut r = r; // 变量 r
        r.headers.push(("Content-Type".into(), "application/json".into())); // 语句
        let s = r#"
            pm.test("token", function () { pm.expect(pm.variables.get("token")).to.eql("abc"); });
            pm.test("iter", function () { pm.expect(pm.iterationData.get("id")).to.eql("42"); });
            pm.test("ct", function () { pm.expect(pm.response.headers.get("content-type")).to.eql("application/json"); });
            pm.test("miss", function () { pm.expect(pm.variables.get("nope")).to.eql("x"); });
        "#; // 语句
        let out = run_env(s, Some(&r), &env); // 变量 out
        assert_eq!(out.tests.len(), 4); // 语句
        assert!(out.tests[0].passed); // 语句
        assert!(out.tests[1].passed); // 语句
        assert!(out.tests[2].passed); // 语句
        assert!(!out.tests[3].passed); // 语句
    } // 块结束

    #[test] // 语句
    fn chai_matchers_not_property_length_match() { // 函数 chai_matchers_not_property_length_match
        let r = resp(200, r#"{"name":"z","tags":[1,2,3],"age":30}"#); // 变量 r
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
        "#; // 语句
        let out = run(s, Some(&r)); // 变量 out
        assert_eq!(out.tests.len(), 9); // 语句
        for t in &out.tests[..8] { // 循环
            assert!(t.passed, "{} failed: {:?}", t.name, t.error); // 语句
        } // 块结束
        assert!(!out.tests[8].passed); // 语句
    } // 块结束
} // 块结束
