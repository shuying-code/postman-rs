//! Collection Runner: iterate requests, resolve variables, execute, assert.
use crate::http::{self, ResponseRecord};
use crate::model::{Auth, Collection, Environment, Event, Item, Request, Variable};
use crate::storage;
use crate::scripting;
use crate::tests_engine::{self, TestResult};
use crate::variables::VarContext;
use std::collections::HashMap;
use std::time::Duration;

pub struct RunOutcome {
    pub name: String,
    pub method: String,
    pub url: String,
    pub code: Option<u16>,
    pub error: Option<String>,
    pub elapsed_ms: u128,
    pub tests: Vec<TestResult>,
    pub passed: bool,
}
pub struct RunSummary {
    pub results: Vec<RunOutcome>,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
}
#[derive(Default)]
pub struct RunOptions {
    pub folder: Option<String>,
    pub bail: bool,
    pub report: Option<String>,
}
fn scope_from_variables(vars: &[Variable]) -> HashMap<String, String> {
    vars.iter()
        .filter(|v| v.is_enabled())
        .map(|v| (v.key.clone(), value_to_string(&v.value)))
        .collect()
}
fn value_to_string(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    }
}
fn base_context(c: &Collection, env: Option<&Environment>) -> VarContext {
    let mut ctx = VarContext::new();
    let coll = scope_from_variables(&c.variable);
    match env {
        Some(e) => {
            ctx.scopes.push(scope_from_variables(&e.values));
            ctx.scopes.push(coll);
        }
        None => ctx.scopes.push(coll),
    }
    if let Ok(g) = storage::load_globals() {
        let gs = scope_from_variables(&g.values);
        if !gs.is_empty() {
            ctx.scopes.push(gs);
        }
    }
    ctx
}
fn collect_assertions(
    req: &Request,
    item_event: &[Event],
    extra_events: &[Event],
) -> Vec<crate::model::TestAssertion> {
    let mut a = req.tests.clone();
    a.extend(tests_engine::extract_from_events(&req.event));
    a.extend(tests_engine::extract_from_events(item_event));
    a.extend(tests_engine::extract_from_events(extra_events));
    a
}
fn eval(
    req: &Request,
    item_event: &[Event],
    extra_events: &[Event],
    resp: &ResponseRecord,
    ctx: &VarContext,
) -> Vec<TestResult> {
    let assertions = collect_assertions(req, item_event, extra_events);
    let mut results = tests_engine::evaluate(&assertions, resp);
    // Run any Postman-style `pm.test(...)` scripts attached to the test event.
    let mut events: Vec<Event> = req.event.clone();
    events.extend(extra_events.iter().cloned());
    events.extend(item_event.iter().cloned());
    let script = scripting::scripts_of(&events, "test");
    if !script.trim().is_empty() {
        let senv = scripting::ScriptEnv {
            vars: ctx.merged(),
            iter: std::collections::HashMap::new(),
            method: req.method.clone(),
            url: req.url.effective_url(),
        };
        let out = scripting::run_env(&script, Some(resp), &senv);
        for t in out.tests {
            results.push(TestResult {
                name: t.name,
                passed: t.passed,
                detail: t.error.unwrap_or_default(),
            });
        }
    }
    results
}
/// Folders/collection inherited down to a request: auth, variables, events.
#[derive(Clone, Default)]
struct Inherited {
    auth: Option<Auth>,
    vars: HashMap<String, String>,
    events: Vec<Event>,
}

fn collect_requests<'a>(
    it: &'a Item,
    parent: &Inherited,
    out: &mut Vec<(&'a Item, Inherited)>,
) {
    let mut inh = parent.clone();
    // Folder/item-level auth overrides ancestors (deeper wins).
    if let Some(a) = &it.auth {
        inh.auth = Some(a.clone());
    }
    for v in &it.variable {
        if v.is_enabled() {
            inh.vars.insert(v.key.clone(), value_to_string(&v.value));
        }
    }
    inh.events.extend(it.event.iter().cloned());
    if it.request.is_some() {
        out.push((it, inh.clone()));
    }
    if let Some(children) = &it.item {
        for c in children {
            collect_requests(c, &inh, out);
        }
    }
}
fn find_item<'a>(items: &'a [Item], name: &str) -> Option<&'a Item> {
    for it in items {
        if it.name == name {
            return Some(it);
        }
        if let Some(children) = &it.item {
            if let Some(found) = find_item(children, name) {
                return Some(found);
            }
        }
    }
    None
}
fn select_items<'a>(c: &'a Collection, folder: &Option<String>) -> Vec<(&'a Item, Inherited)> {
    let mut out = Vec::new();
    let root = Inherited::default();
    match folder {
        None => {
            for it in &c.item {
                collect_requests(it, &root, &mut out);
            }
        }
        Some(name) => {
            if let Some(node) = find_item(&c.item, name) {
                collect_requests(node, &root, &mut out);
            }
        }
    }
    out
}
fn write_report(path: &str, summary: &RunSummary) {
    let results: Vec<serde_json::Value> = summary
        .results
        .iter()
        .map(|r| {
            serde_json::json!({
                "name": r.name,
                "method": r.method,
                "url": r.url,
                "code": r.code,
                "elapsed_ms": r.elapsed_ms,
                "passed": r.passed,
                "error": r.error,
                "tests": r.tests.iter().map(|t| serde_json::json!({"name": t.name, "passed": t.passed, "detail": t.detail})).collect::<Vec<_>>(),
            })
        })
        .collect();
    let doc = serde_json::json!({
        "total": summary.total,
        "passed": summary.passed,
        "failed": summary.failed,
        "results": results,
    });
    if let Ok(t) = serde_json::to_string_pretty(&doc) {
        let _ = std::fs::write(path, t);
    }
}
pub fn run_collection_opts(
    c: &Collection,
    env: Option<&Environment>,
    iterations: usize,
    data_rows: Vec<HashMap<String, String>>,
    delay_ms: u64,
    timeout_secs: u64,
    opts: RunOptions,
) -> RunSummary {
    let base = base_context(c, env);
    let coll_auth = c.auth.clone();
    let selected = select_items(c, &opts.folder);
    let runs: Vec<HashMap<String, String>> = if data_rows.is_empty() {
        vec![HashMap::new(); iterations.max(1)]
    } else {
        data_rows
    };
    let mut results = Vec::new();
    for data in &runs {
        let mut ctx = base.clone();
        if !data.is_empty() {
            ctx.push_high(data.clone());
        }
        run_items(&selected, &coll_auth, &ctx, delay_ms, timeout_secs, opts.bail, &mut results);
        if opts.bail && results.iter().any(|r| !r.passed) {
            break;
        }
    }
    let passed = results.iter().filter(|r| r.passed).count();
    let total = results.len();
    let summary = RunSummary {
        failed: total - passed,
        passed,
        total,
        results,
    };
    if let Some(path) = &opts.report {
        write_report(path, &summary);
    }
    summary
}
pub fn run_collection(
    c: &Collection,
    env: Option<&Environment>,
    iterations: usize,
    data_rows: Vec<HashMap<String, String>>,
    delay_ms: u64,
    timeout_secs: u64,
) -> RunSummary {
    run_collection_opts(
        c,
        env,
        iterations,
        data_rows,
        delay_ms,
        timeout_secs,
        RunOptions::default(),
    )
}
fn run_items(
    items: &[(&Item, Inherited)],
    coll_auth: &Option<Auth>,
    ctx: &VarContext,
    delay_ms: u64,
    timeout_secs: u64,
    bail: bool,
    out: &mut Vec<RunOutcome>,
) {
    for (item, inh) in items {
        if let Some(req0) = &item.request {
            let mut req = req0.clone();
            // Auth: request-level > folder/item inherited > collection-level.
            if req.auth.is_none() {
                req.auth = inh.auth.clone().or_else(|| coll_auth.clone());
            }
            // Per-request context: base scopes + inherited folder variables
            // (folder vars sit just below environment, above collection vars).
            let mut item_ctx = ctx.clone();
            if !inh.vars.is_empty() {
                let idx = 1.min(item_ctx.scopes.len());
                item_ctx.scopes.insert(idx, inh.vars.clone());
            }
            // Pre-request script: apply `pm.<scope>.set(...)` before sending.
            let mut evs: Vec<Event> = inh.events.clone();
            evs.extend(req.event.iter().cloned());
            evs.extend(item.event.iter().cloned());
            let pre = scripting::scripts_of(&evs, "prerequest");
            if !pre.trim().is_empty() {
                let senv = scripting::ScriptEnv {
                    vars: item_ctx.merged(),
                    iter: std::collections::HashMap::new(),
                    method: req.method.clone(),
                    url: req.url.effective_url(),
                };
                let out = scripting::run_env(&pre, None, &senv);
                if !out.sets.is_empty() {
                    let mut map = HashMap::new();
                    for (_scope, k, v) in out.sets {
                        map.insert(k, v);
                    }
                    item_ctx.push_high(map);
                }
            }
            let resolved_url = item_ctx.resolve(&req.url.effective_url());
            match http::execute(&req, &item_ctx, timeout_secs) {
                Ok(resp) => {
                    let tests = eval(&req, &item.event, &inh.events, &resp, &item_ctx);
                    let passed = tests.iter().all(|t| t.passed);
                    out.push(RunOutcome {
                        name: item.name.clone(),
                        method: req.method.to_uppercase(),
                        url: resolved_url,
                        code: Some(resp.code),
                        error: None,
                        elapsed_ms: resp.elapsed_ms,
                        tests,
                        passed,
                    });
                }
                Err(e) => {
                    out.push(RunOutcome {
                        name: item.name.clone(),
                        method: req.method.to_uppercase(),
                        url: resolved_url,
                        code: None,
                        error: Some(e.to_string()),
                        elapsed_ms: 0,
                        tests: Vec::new(),
                        passed: false,
                    });
                }
            }
            if bail && out.last().map(|r| !r.passed).unwrap_or(false) {
                return;
            }
            if delay_ms > 0 {
                std::thread::sleep(Duration::from_millis(delay_ms));
            }
        }
    }
}
pub fn load_data_file(path: &str) -> anyhow::Result<Vec<HashMap<String, String>>> {
    let text = std::fs::read_to_string(path)?;
    if path.to_ascii_lowercase().ends_with(".json") {
        let v: serde_json::Value = serde_json::from_str(&text)?;
        let arr = v
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("JSON data must be an array of objects"))?;
        let mut rows = Vec::new();
        for obj in arr {
            let mut m = HashMap::new();
            if let Some(o) = obj.as_object() {
                for (k, val) in o {
                    m.insert(k.clone(), value_to_string(val));
                }
            }
            rows.push(m);
        }
        Ok(rows)
    } else {
        let mut rdr = csv::Reader::from_reader(text.as_bytes());
        let headers = rdr.headers()?.clone();
        let mut rows = Vec::new();
        for rec in rdr.records() {
            let rec = rec?;
            let mut m = HashMap::new();
            for (h, val) in headers.iter().zip(rec.iter()) {
                m.insert(h.to_string(), val.to_string());
            }
            rows.push(m);
        }
        Ok(rows)
    }
}

