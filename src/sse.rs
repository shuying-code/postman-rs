//! Server-Sent Events client (consumes a text/event-stream).
use anyhow::Result; // 引入依赖
use std::io::{BufRead, BufReader}; // 引入依赖
pub fn run(url: &str, headers: &[(String, String)], count: usize, timeout_secs: u64) -> Result<()> { // 函数 run
    let resp = crate::http::open_stream(url, headers, timeout_secs)?; // 变量 resp
    println!("connected: {} (status {})", url, resp.status()); // 语句
    let mut reader = BufReader::new(resp); // 变量 reader
    let mut line = String::new(); // 变量 line
    let mut events = 0usize; // 变量 events
    loop { // 循环
        line.clear(); // 语句
        let n = reader.read_line(&mut line)?; // 变量 n
        if n == 0 { // 条件分支
            break; // 语句
        } // 块结束
        let trimmed = line.trim_end_matches(['\r', '\n']); // 变量 trimmed
        if trimmed.is_empty() { // 条件分支
            println!("---"); // 语句
            continue; // 语句
        } // 块结束
        println!("{}", trimmed); // 语句
        if trimmed.starts_with("data:") { // 条件分支
            events += 1; // 语句
            if count > 0 && events >= count { // 条件分支
                break; // 语句
            } // 块结束
        } // 块结束
    } // 块结束
    Ok(()) // 调用 Ok
} // 块结束
