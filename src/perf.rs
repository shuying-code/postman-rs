//! Lightweight performance/load test (latency percentiles, error rate, RPS).
use crate::http; // 引入依赖
use crate::model::Request; // 引入依赖
use crate::variables::VarContext; // 引入依赖
use std::sync::{Arc, Mutex}; // 引入依赖
use std::time::Instant; // 引入依赖
pub struct PerfReport { // 结构体 PerfReport
    pub total: usize, // 字段 total
    pub ok: usize, // 字段 ok
    pub err: usize, // 字段 err
    pub rps: f64, // 字段 rps
    pub mean_ms: f64, // 字段 mean_ms
    pub p50: u128, // 字段 p50
    pub p90: u128, // 字段 p90
    pub p99: u128, // 字段 p99
    pub max_ms: u128, // 字段 max_ms
} // 块结束
fn pct(sorted: &[u128], p: f64) -> u128 { // 函数 pct
    if sorted.is_empty() { // 条件分支
        return 0; // 返回
    } // 块结束
    let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize; // 变量 idx
    sorted[idx.min(sorted.len() - 1)] // 语句
} // 块结束
pub fn run( // 函数 run
    req: &Request, // 字段 req
    ctx: &VarContext, // 字段 ctx
    iterations: usize, // 字段 iterations
    concurrency: usize, // 字段 concurrency
    timeout_secs: u64, // 字段 timeout_secs
) -> PerfReport { // 语句
    let req = Arc::new(req.clone()); // 变量 req
    let ctx = Arc::new(ctx.clone()); // 变量 ctx
    let conc = concurrency.max(1); // 变量 conc
    let lat = Arc::new(Mutex::new(Vec::<u128>::new())); // 变量 lat
    let errs = Arc::new(Mutex::new(0usize)); // 变量 errs
    let start = Instant::now(); // 变量 start
    let mut handles = Vec::new(); // 变量 handles
    let base = iterations / conc; // 变量 base
    let rem = iterations % conc; // 变量 rem
    for t in 0..conc { // 循环
        let count = base + if t < rem { 1 } else { 0 }; // 变量 count
        let req = req.clone(); // 变量 req
        let ctx = ctx.clone(); // 变量 ctx
        let lat = lat.clone(); // 变量 lat
        let errs = errs.clone(); // 变量 errs
        handles.push(std::thread::spawn(move || { // 语句
            for _ in 0..count { // 循环
                let s = Instant::now(); // 变量 s
                match http::execute(&req, &ctx, timeout_secs) { // 分支匹配
                    Ok(_) => lat.lock().unwrap().push(s.elapsed().as_millis()), // 匹配分支
                    Err(_) => *errs.lock().unwrap() += 1, // 匹配分支
                } // 块结束
            } // 块结束
        })); // 语句
    } // 块结束
    for h in handles { // 循环
        let _ = h.join(); // 变量 _
    } // 块结束
    let elapsed = start.elapsed().as_secs_f64(); // 变量 elapsed
    let mut lats = Arc::try_unwrap(lat).map(|m| m.into_inner().unwrap()).unwrap_or_default(); // 变量 lats
    lats.sort_unstable(); // 语句
    let ok = lats.len(); // 变量 ok
    let err = Arc::try_unwrap(errs).map(|m| m.into_inner().unwrap()).unwrap_or(0); // 变量 err
    let mean = if ok > 0 { // 变量 mean
        lats.iter().sum::<u128>() as f64 / ok as f64 // 语句
    } else { // 语句
        0.0 // 语句
    }; // 块结束
    PerfReport { // 语句
        total: iterations, // 字段 total
        ok, // 语句
        err, // 语句
        rps: if elapsed > 0.0 { ok as f64 / elapsed } else { 0.0 }, // 字段 rps
        mean_ms: mean, // 字段 mean_ms
        p50: pct(&lats, 0.50), // 字段 p50
        p90: pct(&lats, 0.90), // 字段 p90
        p99: pct(&lats, 0.99), // 字段 p99
        max_ms: lats.last().copied().unwrap_or(0), // 字段 max_ms
    } // 块结束
} // 块结束
