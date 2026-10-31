//! Lightweight performance/load test (latency percentiles, error rate, RPS).
use crate::http;
use crate::model::Request;
use crate::variables::VarContext;
use std::sync::{Arc, Mutex};
use std::time::Instant;
pub struct PerfReport {
    pub total: usize,
    pub ok: usize,
    pub err: usize,
    pub rps: f64,
    pub mean_ms: f64,
    pub p50: u128,
    pub p90: u128,
    pub p99: u128,
    pub max_ms: u128,
}
fn pct(sorted: &[u128], p: f64) -> u128 {
    if sorted.is_empty() {
        return 0;
    }
    let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}
pub fn run(
    req: &Request,
    ctx: &VarContext,
    iterations: usize,
    concurrency: usize,
    timeout_secs: u64,
) -> PerfReport {
    let req = Arc::new(req.clone());
    let ctx = Arc::new(ctx.clone());
    let conc = concurrency.max(1);
    let lat = Arc::new(Mutex::new(Vec::<u128>::new()));
    let errs = Arc::new(Mutex::new(0usize));
    let start = Instant::now();
    let mut handles = Vec::new();
    let base = iterations / conc;
    let rem = iterations % conc;
    for t in 0..conc {
        let count = base + if t < rem { 1 } else { 0 };
        let req = req.clone();
        let ctx = ctx.clone();
        let lat = lat.clone();
        let errs = errs.clone();
        handles.push(std::thread::spawn(move || {
            for _ in 0..count {
                let s = Instant::now();
                match http::execute(&req, &ctx, timeout_secs) {
                    Ok(_) => lat.lock().unwrap().push(s.elapsed().as_millis()),
                    Err(_) => *errs.lock().unwrap() += 1,
                }
            }
        }));
    }
    for h in handles {
        let _ = h.join();
    }
    let elapsed = start.elapsed().as_secs_f64();
    let mut lats = Arc::try_unwrap(lat).map(|m| m.into_inner().unwrap()).unwrap_or_default();
    lats.sort_unstable();
    let ok = lats.len();
    let err = Arc::try_unwrap(errs).map(|m| m.into_inner().unwrap()).unwrap_or(0);
    let mean = if ok > 0 {
        lats.iter().sum::<u128>() as f64 / ok as f64
    } else {
        0.0
    };
    PerfReport {
        total: iterations,
        ok,
        err,
        rps: if elapsed > 0.0 { ok as f64 / elapsed } else { 0.0 },
        mean_ms: mean,
        p50: pct(&lats, 0.50),
        p90: pct(&lats, 0.90),
        p99: pct(&lats, 0.99),
        max_ms: lats.last().copied().unwrap_or(0),
    }
}
