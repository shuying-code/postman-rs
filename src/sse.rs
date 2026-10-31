//! Server-Sent Events client (consumes a text/event-stream).
use anyhow::Result;
use std::io::{BufRead, BufReader};
pub fn run(url: &str, headers: &[(String, String)], count: usize, timeout_secs: u64) -> Result<()> {
    let resp = crate::http::open_stream(url, headers, timeout_secs)?;
    println!("connected: {} (status {})", url, resp.status());
    let mut reader = BufReader::new(resp);
    let mut line = String::new();
    let mut events = 0usize;
    loop {
        line.clear();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            break;
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            println!("---");
            continue;
        }
        println!("{}", trimmed);
        if trimmed.starts_with("data:") {
            events += 1;
            if count > 0 && events >= count {
                break;
            }
        }
    }
    Ok(())
}
