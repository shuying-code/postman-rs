//! Collection Runner: iterate requests, resolve variables, execute, assert.
use crate::http::{self, ResponseRecord}; // 引入依赖
use crate::model::{Auth, Collection, Environment, Event, Item, Request, Variable}; // 引入依赖
use crate::storage; // 引入依赖
use crate::scripting; // 引入依赖
use crate::tests_engine::{self, TestResult}; // 引入依赖
use crate::variables::VarContext; // 引入依赖
use std::collections::HashMap; // 引入依赖
use std::time::Duration; // 引入依赖

pub struct RunOutcome { // 结构体 RunOutcome
    pub name: String, // 字段 name
    pub method: String, // 字段 method
    pub url: String, // 字段 url
    pub code: Option<u16>, // 字段 code
    pub error: Option<String>, // 字段 error
    pub elapsed_ms: u128, // 字段 elapsed_ms
    pub tests: Vec<TestResult>, // 字段 tests
    pub passed: bool, // 字段 passed
} // 块结束
pub struct RunSummary { // 结构体 RunSummary
    pub results: Vec<RunOutcome>, // 字段 results
    pub total: usize, // 字段 total
    pub passed: usize, // 字段 passed
    pub failed: usize, // 字段 failed
} // 块结束
#[derive(Default)] // 语句
pub struct RunOptions { // 结构体 RunOptions
    pub folder: Option<String>, // 字段 folder
    pub bail: bool, // 字段 bail
    pub report: Option<String>, // 字段 report
} // 块结束
fn scope_from_variables(vars: &[Variable]) -> HashMap<String, String> { // 函数 scope_from_variables
    vars.iter() // 语句
        .filter(|v| v.is_enabled()) // 语句
        .map(|v| (v.key.clone(), value_to_string(&v.value))) // 语句
        .collect() // 语句
} // 块结束
fn value_to_string(v: &serde_json::Value) -> String { // 函数 value_to_string
    match v { // 分支匹配
        serde_json::Value::String(s) => s.clone(), // 匹配分支
        serde_json::Value::Null => String::new(), // 匹配分支
        other => other.to_string(), // 匹配分支
    } // 块结束
} // 块结束
fn base_context(c: &Collection, env: Option<&Environment>) -> VarContext { // 函数 base_context
    let mut ctx = VarContext::new(); // 变量 ctx
    let coll = scope_from_variables(&c.variable); // 变量 coll
    match env { // 分支匹配
        Some(e) => { // 匹配分支
            ctx.scopes.push(scope_from_variables(&e.values)); // 语句
            ctx.scopes.push(coll); // 语句
        } // 块结束
        None => ctx.scopes.push(coll), // 匹配分支
    } // 块结束
    if let Ok(g) = storage::load_globals() { // 条件分支
        let gs = scope_from_variables(&g.values); // 变量 gs
        if !gs.is_empty() { // 条件分支
            ctx.scopes.push(gs); // 语句
        } // 块结束
    } // 块结束
    ctx // 语句
} // 块结束
fn collect_assertions( // 函数 collect_assertions
    req: &Request, // 字段 req
    item_event: &[Event], // 字段 item_event
    extra_events: &[Event], // 字段 extra_events
) -> Vec<crate::model::TestAssertion> { // 语句
    let mut a = req.tests.clone(); // 变量 a
    a.extend(tests_engine::extract_from_events(&req.event)); // 语句
    a.extend(tests_engine::extract_from_events(item_event)); // 语句
    a.extend(tests_engine::extract_from_events(extra_events)); // 语句
    a // 语句
} // 块结束
fn eval( // 函数 eval
    req: &Request, // 字段 req
    item_event: &[Event], // 字段 item_event
    extra_events: &[Event], // 字段 extra_events
    resp: &ResponseRecord, // 字段 resp
    ctx: &VarContext, // 字段 ctx
) -> Vec<TestResult> { // 语句
    let assertions = collect_assertions(req, item_event, extra_events); // 变量 assertions
    let mut results = tests_engine::evaluate(&assertions, resp); // 变量 results
    // Run any Postman-style `pm.test(...)` scripts attached to the test event.
    let mut events: Vec<Event> = req.event.clone(); // 变量 events
    events.extend(extra_events.iter().cloned()); // 语句
    events.extend(item_event.iter().cloned()); // 语句
    let script = scripting::scripts_of(&events, "test"); // 变量 script
    if !script.trim().is_empty() { // 条件分支
        let senv = scripting::ScriptEnv { // 变量 senv
            vars: ctx.merged(), // 字段 vars
            iter: std::collections::HashMap::new(), // 字段 iter
            method: req.method.clone(), // 字段 method
            url: req.url.effective_url(), // 字段 url
        }; // 块结束
        let out = scripting::run_env(&script, Some(resp), &senv); // 变量 out
        for t in out.tests { // 循环
            results.push(TestResult { // 语句
                name: t.name, // 字段 name
                passed: t.passed, // 字段 passed
                detail: t.error.unwrap_or_default(), // 字段 detail
            }); // 语句
        } // 块结束
    } // 块结束
    results // 语句
} // 块结束
/// Folders/collection inherited down to a request: auth, variables, events.
#[derive(Clone, Default)] // 语句
struct Inherited { // 结构体 Inherited
    auth: Option<Auth>, // 字段 auth
    vars: HashMap<String, String>, // 字段 vars
    events: Vec<Event>, // 字段 events
} // 块结束

fn collect_requests<'a>( // 函数 collect_requests
    it: &'a Item, // 字段 it
    parent: &Inherited, // 字段 parent
    out: &mut Vec<(&'a Item, Inherited)>, // 字段 out
) { // 语句
    let mut inh = parent.clone(); // 变量 inh
    // Folder/item-level auth overrides ancestors (deeper wins).
    if let Some(a) = &it.auth { // 条件分支
        inh.auth = Some(a.clone()); // 语句
    } // 块结束
    for v in &it.variable { // 循环
        if v.is_enabled() { // 条件分支
            inh.vars.insert(v.key.clone(), value_to_string(&v.value)); // 语句
        } // 块结束
    } // 块结束
    inh.events.extend(it.event.iter().cloned()); // 语句
    if it.request.is_some() { // 条件分支
        out.push((it, inh.clone())); // 语句
    } // 块结束
    if let Some(children) = &it.item { // 条件分支
        for c in children { // 循环
            collect_requests(c, &inh, out); // 调用 collect_requests
        } // 块结束
    } // 块结束
} // 块结束
fn find_item<'a>(items: &'a [Item], name: &str) -> Option<&'a Item> { // 函数 find_item
    for it in items { // 循环
        if it.name == name { // 条件分支
            return Some(it); // 返回
        } // 块结束
        if let Some(children) = &it.item { // 条件分支
            if let Some(found) = find_item(children, name) { // 条件分支
                return Some(found); // 返回
            } // 块结束
        } // 块结束
    } // 块结束
    None // 语句
} // 块结束
fn select_items<'a>(c: &'a Collection, folder: &Option<String>) -> Vec<(&'a Item, Inherited)> { // 函数 select_items
    let mut out = Vec::new(); // 变量 out
    let root = Inherited::default(); // 变量 root
    match folder { // 分支匹配
        None => { // 匹配分支
            for it in &c.item { // 循环
                collect_requests(it, &root, &mut out); // 调用 collect_requests
            } // 块结束
        } // 块结束
        Some(name) => { // 匹配分支
            if let Some(node) = find_item(&c.item, name) { // 条件分支
                collect_requests(node, &root, &mut out); // 调用 collect_requests
            } // 块结束
        } // 块结束
    } // 块结束
    out // 语句
} // 块结束
fn write_report(path: &str, summary: &RunSummary) { // 函数 write_report
    let results: Vec<serde_json::Value> = summary // 变量 results
        .results // 语句
        .iter() // 语句
        .map(|r| { // 语句
            serde_json::json!({ // 语句
                "name": r.name, // 语句
                "method": r.method, // 语句
                "url": r.url, // 语句
                "code": r.code, // 语句
                "elapsed_ms": r.elapsed_ms, // 语句
                "passed": r.passed, // 语句
                "error": r.error, // 语句
                "tests": r.tests.iter().map(|t| serde_json::json!({"name": t.name, "passed": t.passed, "detail": t.detail})).collect::<Vec<_>>(), // 语句
            }) // 语句
        }) // 语句
        .collect(); // 语句
    let doc = serde_json::json!({ // 变量 doc
        "total": summary.total, // 语句
        "passed": summary.passed, // 语句
        "failed": summary.failed, // 语句
        "results": results, // 语句
    }); // 语句
    if let Ok(t) = serde_json::to_string_pretty(&doc) { // 条件分支
        let _ = std::fs::write(path, t); // 变量 _
    } // 块结束
} // 块结束
pub fn run_collection_opts( // 函数 run_collection_opts
    c: &Collection, // 字段 c
    env: Option<&Environment>, // 字段 env
    iterations: usize, // 字段 iterations
    data_rows: Vec<HashMap<String, String>>, // 字段 data_rows
    delay_ms: u64, // 字段 delay_ms
    timeout_secs: u64, // 字段 timeout_secs
    opts: RunOptions, // 字段 opts
) -> RunSummary { // 语句
    let base = base_context(c, env); // 变量 base
    let coll_auth = c.auth.clone(); // 变量 coll_auth
    let selected = select_items(c, &opts.folder); // 变量 selected
    let runs: Vec<HashMap<String, String>> = if data_rows.is_empty() { // 变量 runs
        vec![HashMap::new(); iterations.max(1)] // 语句
    } else { // 语句
        data_rows // 语句
    }; // 块结束
    let mut results = Vec::new(); // 变量 results
    for data in &runs { // 循环
        let mut ctx = base.clone(); // 变量 ctx
        if !data.is_empty() { // 条件分支
            ctx.push_high(data.clone()); // 语句
        } // 块结束
        run_items(&selected, &coll_auth, &ctx, delay_ms, timeout_secs, opts.bail, &mut results); // 调用 run_items
        if opts.bail && results.iter().any(|r| !r.passed) { // 条件分支
            break; // 语句
        } // 块结束
    } // 块结束
    let passed = results.iter().filter(|r| r.passed).count(); // 变量 passed
    let total = results.len(); // 变量 total
    let summary = RunSummary { // 变量 summary
        failed: total - passed, // 字段 failed
        passed, // 语句
        total, // 语句
        results, // 语句
    }; // 块结束
    if let Some(path) = &opts.report { // 条件分支
        write_report(path, &summary); // 调用 write_report
    } // 块结束
    summary // 语句
} // 块结束
pub fn run_collection( // 函数 run_collection
    c: &Collection, // 字段 c
    env: Option<&Environment>, // 字段 env
    iterations: usize, // 字段 iterations
    data_rows: Vec<HashMap<String, String>>, // 字段 data_rows
    delay_ms: u64, // 字段 delay_ms
    timeout_secs: u64, // 字段 timeout_secs
) -> RunSummary { // 语句
    run_collection_opts( // 调用 run_collection_opts
        c, // 语句
        env, // 语句
        iterations, // 语句
        data_rows, // 语句
        delay_ms, // 语句
        timeout_secs, // 语句
        RunOptions::default(), // 字段 RunOptions
    ) // 块结束
} // 块结束
fn run_items( // 函数 run_items
    items: &[(&Item, Inherited)], // 字段 items
    coll_auth: &Option<Auth>, // 字段 coll_auth
    ctx: &VarContext, // 字段 ctx
    delay_ms: u64, // 字段 delay_ms
    timeout_secs: u64, // 字段 timeout_secs
    bail: bool, // 字段 bail
    out: &mut Vec<RunOutcome>, // 字段 out
) { // 语句
    for (item, inh) in items { // 循环
        if let Some(req0) = &item.request { // 条件分支
            let mut req = req0.clone(); // 变量 req
            // Auth: request-level > folder/item inherited > collection-level.
            if req.auth.is_none() { // 条件分支
                req.auth = inh.auth.clone().or_else(|| coll_auth.clone()); // 语句
            } // 块结束
            // Per-request context: base scopes + inherited folder variables
            // (folder vars sit just below environment, above collection vars).
            let mut item_ctx = ctx.clone(); // 变量 item_ctx
            if !inh.vars.is_empty() { // 条件分支
                let idx = 1.min(item_ctx.scopes.len()); // 变量 idx
                item_ctx.scopes.insert(idx, inh.vars.clone()); // 语句
            } // 块结束
            // Pre-request script: apply `pm.<scope>.set(...)` before sending.
            let mut evs: Vec<Event> = inh.events.clone(); // 变量 evs
            evs.extend(req.event.iter().cloned()); // 语句
            evs.extend(item.event.iter().cloned()); // 语句
            let pre = scripting::scripts_of(&evs, "prerequest"); // 变量 pre
            if !pre.trim().is_empty() { // 条件分支
                let senv = scripting::ScriptEnv { // 变量 senv
                    vars: item_ctx.merged(), // 字段 vars
                    iter: std::collections::HashMap::new(), // 字段 iter
                    method: req.method.clone(), // 字段 method
                    url: req.url.effective_url(), // 字段 url
                }; // 块结束
                let out = scripting::run_env(&pre, None, &senv); // 变量 out
                if !out.sets.is_empty() { // 条件分支
                    let mut map = HashMap::new(); // 变量 map
                    for (_scope, k, v) in out.sets { // 循环
                        map.insert(k, v); // 语句
                    } // 块结束
                    item_ctx.push_high(map); // 语句
                } // 块结束
            } // 块结束
            let resolved_url = item_ctx.resolve(&req.url.effective_url()); // 变量 resolved_url
            match http::execute(&req, &item_ctx, timeout_secs) { // 分支匹配
                Ok(resp) => { // 匹配分支
                    let tests = eval(&req, &item.event, &inh.events, &resp, &item_ctx); // 变量 tests
                    let passed = tests.iter().all(|t| t.passed); // 变量 passed
                    out.push(RunOutcome { // 语句
                        name: item.name.clone(), // 字段 name
                        method: req.method.to_uppercase(), // 字段 method
                        url: resolved_url, // 字段 url
                        code: Some(resp.code), // 字段 code
                        error: None, // 字段 error
                        elapsed_ms: resp.elapsed_ms, // 字段 elapsed_ms
                        tests, // 语句
                        passed, // 语句
                    }); // 语句
                } // 块结束
                Err(e) => { // 匹配分支
                    out.push(RunOutcome { // 语句
                        name: item.name.clone(), // 字段 name
                        method: req.method.to_uppercase(), // 字段 method
                        url: resolved_url, // 字段 url
                        code: None, // 字段 code
                        error: Some(e.to_string()), // 字段 error
                        elapsed_ms: 0, // 字段 elapsed_ms
                        tests: Vec::new(), // 字段 tests
                        passed: false, // 字段 passed
                    }); // 语句
                } // 块结束
            } // 块结束
            if bail && out.last().map(|r| !r.passed).unwrap_or(false) { // 条件分支
                return; // 返回
            } // 块结束
            if delay_ms > 0 { // 条件分支
                std::thread::sleep(Duration::from_millis(delay_ms)); // 调用 sleep
            } // 块结束
        } // 块结束
    } // 块结束
} // 块结束
pub fn load_data_file(path: &str) -> anyhow::Result<Vec<HashMap<String, String>>> { // 函数 load_data_file
    let text = std::fs::read_to_string(path)?; // 变量 text
    if path.to_ascii_lowercase().ends_with(".json") { // 条件分支
        let v: serde_json::Value = serde_json::from_str(&text)?; // 变量 v
        let arr = v // 变量 arr
            .as_array() // 语句
            .ok_or_else(|| anyhow::anyhow!("JSON data must be an array of objects"))?; // 语句
        let mut rows = Vec::new(); // 变量 rows
        for obj in arr { // 循环
            let mut m = HashMap::new(); // 变量 m
            if let Some(o) = obj.as_object() { // 条件分支
                for (k, val) in o { // 循环
                    m.insert(k.clone(), value_to_string(val)); // 语句
                } // 块结束
            } // 块结束
            rows.push(m); // 语句
        } // 块结束
        Ok(rows) // 调用 Ok
    } else { // 语句
        let mut rdr = csv::Reader::from_reader(text.as_bytes()); // 变量 rdr
        let headers = rdr.headers()?.clone(); // 变量 headers
        let mut rows = Vec::new(); // 变量 rows
        for rec in rdr.records() { // 循环
            let rec = rec?; // 变量 rec
            let mut m = HashMap::new(); // 变量 m
            for (h, val) in headers.iter().zip(rec.iter()) { // 循环
                m.insert(h.to_string(), val.to_string()); // 语句
            } // 块结束
            rows.push(m); // 语句
        } // 块结束
        Ok(rows) // 调用 Ok
    } // 块结束
} // 块结束

