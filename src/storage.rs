//! Local, file-based workspace (no database, no cloud).
use crate::model::{Collection, Environment}; // 引入依赖
use anyhow::{Context, Result}; // 引入依赖
use serde::{Deserialize, Serialize}; // 引入依赖
use std::path::PathBuf; // 引入依赖

#[derive(Debug, Clone, Serialize, Deserialize)] // 语句
pub struct HistoryEntry { // 结构体 HistoryEntry
    pub ts: String, // 字段 ts
    pub method: String, // 字段 method
    pub url: String, // 字段 url
    pub status: u16, // 字段 status
    pub elapsed_ms: u128, // 字段 elapsed_ms
} // 块结束

pub fn home() -> PathBuf { // 函数 home
    if let Ok(p) = std::env::var("PMRS_HOME") { // 条件分支
        return PathBuf::from(p); // 返回
    } // 块结束
    let base = std::env::var("USERPROFILE") // 变量 base
        .or_else(|_| std::env::var("HOME")) // 语句
        .unwrap_or_else(|_| ".".to_string()); // 语句
    PathBuf::from(base).join(".postman-rs") // 调用 from
} // 块结束

pub fn ensure_dirs() -> Result<()> { // 函数 ensure_dirs
    let h = home(); // 变量 h
    std::fs::create_dir_all(h.join("collections")).context("create collections dir")?; // 调用 create_dir_all
    std::fs::create_dir_all(h.join("environments")).context("create environments dir")?; // 调用 create_dir_all
    Ok(()) // 调用 Ok
} // 块结束

fn sanitize(name: &str) -> String { // 函数 sanitize
    let mut s: String = name // 变量 s
        .chars() // 语句
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' { c } else { '_' }) // 语句
        .collect(); // 语句
    s = s.trim().to_string(); // 语句
    if s.is_empty() { // 条件分支
        s = "untitled".to_string(); // 语句
    } // 块结束
    s // 语句
} // 块结束

pub fn save_collection(c: &Collection) -> Result<PathBuf> { // 函数 save_collection
    ensure_dirs()?; // 调用 ensure_dirs
    let name = if c.info.name.is_empty() { "untitled" } else { &c.info.name }; // 变量 name
    let path = home().join("collections").join(format!("{}.json", sanitize(name))); // 变量 path
    let text = serde_json::to_string_pretty(c)?; // 变量 text
    std::fs::write(&path, text).context("write collection")?; // 调用 write
    Ok(path) // 调用 Ok
} // 块结束

pub fn save_environment(e: &Environment) -> Result<PathBuf> { // 函数 save_environment
    ensure_dirs()?; // 调用 ensure_dirs
    let name = if e.name.is_empty() { "untitled" } else { &e.name }; // 变量 name
    let path = home() // 变量 path
        .join("environments") // 语句
        .join(format!("{}.json", sanitize(name))); // 语句
    std::fs::write(&path, serde_json::to_string_pretty(e)?).context("write environment")?; // 调用 write
    Ok(path) // 调用 Ok
} // 块结束

pub fn list_collections() -> Result<Vec<PathBuf>> { // 函数 list_collections
    ensure_dirs()?; // 调用 ensure_dirs
    Ok(list_json(home().join("collections"))) // 调用 Ok
} // 块结束

pub fn list_environments() -> Result<Vec<PathBuf>> { // 函数 list_environments
    ensure_dirs()?; // 调用 ensure_dirs
    Ok(list_json(home().join("environments"))) // 调用 Ok
} // 块结束

fn list_json(dir: PathBuf) -> Vec<PathBuf> { // 函数 list_json
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir) // 变量 v
        .map(|rd| { // 语句
            rd.filter_map(|e| e.ok()) // 语句
                .map(|e| e.path()) // 语句
                .filter(|p| p.extension().map(|x| x == "json").unwrap_or(false)) // 语句
                .collect() // 语句
        }) // 语句
        .unwrap_or_default(); // 语句
    v.sort(); // 语句
    v // 语句
} // 块结束

pub fn append_history(entry: &HistoryEntry) -> Result<()> { // 函数 append_history
    ensure_dirs()?; // 调用 ensure_dirs
    let path = home().join("history.jsonl"); // 变量 path
    let mut line = serde_json::to_string(entry)?; // 变量 line
    line.push('\n'); // 语句
    use std::io::Write; // 引入依赖
    let mut f = std::fs::OpenOptions::new() // 变量 f
        .create(true) // 语句
        .append(true) // 语句
        .open(path) // 语句
        .context("open history")?; // 语句
    f.write_all(line.as_bytes())?; // 语句
    Ok(()) // 调用 Ok
} // 块结束

pub fn read_history() -> Result<Vec<HistoryEntry>> { // 函数 read_history
    let path = home().join("history.jsonl"); // 变量 path
    if !path.exists() { // 条件分支
        return Ok(Vec::new()); // 返回
    } // 块结束
    let text = std::fs::read_to_string(path)?; // 变量 text
    Ok(text // 调用 Ok
        .lines() // 语句
        .filter(|l| !l.trim().is_empty()) // 语句
        .filter_map(|l| serde_json::from_str::<HistoryEntry>(l).ok()) // 语句
        .collect()) // 语句
} // 块结束

pub fn clear_history() -> Result<()> { // 函数 clear_history
    let path = home().join("history.jsonl"); // 变量 path
    if path.exists() { // 条件分支
        std::fs::remove_file(path)?; // 调用 remove_file
    } // 块结束
    Ok(()) // 调用 Ok
} // 块结束

fn globals_path() -> PathBuf { // 函数 globals_path
    home().join("globals.json") // 调用 home
} // 块结束

/// Load global variables (stored as an Environment with scope="globals").
pub fn load_globals() -> Result<Environment> { // 函数 load_globals
    let path = globals_path(); // 变量 path
    if !path.exists() { // 条件分支
        return Ok(Environment { // 返回
            id: None, // 字段 id
            name: "globals".to_string(), // 字段 name
            values: Vec::new(), // 字段 values
            scope: Some("globals".to_string()), // 字段 scope
        }); // 语句
    } // 块结束
    let text = std::fs::read_to_string(&path)?; // 变量 text
    let mut e: Environment = serde_json::from_str(&text)?; // 变量 e
    if e.scope.is_none() { // 条件分支
        e.scope = Some("globals".to_string()); // 语句
    } // 块结束
    Ok(e) // 调用 Ok
} // 块结束

pub fn save_globals(e: &Environment) -> Result<PathBuf> { // 函数 save_globals
    ensure_dirs()?; // 调用 ensure_dirs
    let path = globals_path(); // 变量 path
    std::fs::write(&path, serde_json::to_string_pretty(e)?).context("write globals")?; // 调用 write
    Ok(path) // 调用 Ok
} // 块结束
