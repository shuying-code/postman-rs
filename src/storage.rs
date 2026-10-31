//! Local, file-based workspace (no database, no cloud).
use crate::model::{Collection, Environment};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub ts: String,
    pub method: String,
    pub url: String,
    pub status: u16,
    pub elapsed_ms: u128,
}

pub fn home() -> PathBuf {
    if let Ok(p) = std::env::var("PMRS_HOME") {
        return PathBuf::from(p);
    }
    let base = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(base).join(".postman-rs")
}

pub fn ensure_dirs() -> Result<()> {
    let h = home();
    std::fs::create_dir_all(h.join("collections")).context("create collections dir")?;
    std::fs::create_dir_all(h.join("environments")).context("create environments dir")?;
    Ok(())
}

fn sanitize(name: &str) -> String {
    let mut s: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' { c } else { '_' })
        .collect();
    s = s.trim().to_string();
    if s.is_empty() {
        s = "untitled".to_string();
    }
    s
}

pub fn save_collection(c: &Collection) -> Result<PathBuf> {
    ensure_dirs()?;
    let name = if c.info.name.is_empty() { "untitled" } else { &c.info.name };
    let path = home().join("collections").join(format!("{}.json", sanitize(name)));
    let text = serde_json::to_string_pretty(c)?;
    std::fs::write(&path, text).context("write collection")?;
    Ok(path)
}

pub fn save_environment(e: &Environment) -> Result<PathBuf> {
    ensure_dirs()?;
    let name = if e.name.is_empty() { "untitled" } else { &e.name };
    let path = home()
        .join("environments")
        .join(format!("{}.json", sanitize(name)));
    std::fs::write(&path, serde_json::to_string_pretty(e)?).context("write environment")?;
    Ok(path)
}

pub fn list_collections() -> Result<Vec<PathBuf>> {
    ensure_dirs()?;
    Ok(list_json(home().join("collections")))
}

pub fn list_environments() -> Result<Vec<PathBuf>> {
    ensure_dirs()?;
    Ok(list_json(home().join("environments")))
}

fn list_json(dir: PathBuf) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().map(|x| x == "json").unwrap_or(false))
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

pub fn append_history(entry: &HistoryEntry) -> Result<()> {
    ensure_dirs()?;
    let path = home().join("history.jsonl");
    let mut line = serde_json::to_string(entry)?;
    line.push('\n');
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .context("open history")?;
    f.write_all(line.as_bytes())?;
    Ok(())
}

pub fn read_history() -> Result<Vec<HistoryEntry>> {
    let path = home().join("history.jsonl");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = std::fs::read_to_string(path)?;
    Ok(text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str::<HistoryEntry>(l).ok())
        .collect())
}

pub fn clear_history() -> Result<()> {
    let path = home().join("history.jsonl");
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

fn globals_path() -> PathBuf {
    home().join("globals.json")
}

/// Load global variables (stored as an Environment with scope="globals").
pub fn load_globals() -> Result<Environment> {
    let path = globals_path();
    if !path.exists() {
        return Ok(Environment {
            id: None,
            name: "globals".to_string(),
            values: Vec::new(),
            scope: Some("globals".to_string()),
        });
    }
    let text = std::fs::read_to_string(&path)?;
    let mut e: Environment = serde_json::from_str(&text)?;
    if e.scope.is_none() {
        e.scope = Some("globals".to_string());
    }
    Ok(e)
}

pub fn save_globals(e: &Environment) -> Result<PathBuf> {
    ensure_dirs()?;
    let path = globals_path();
    std::fs::write(&path, serde_json::to_string_pretty(e)?).context("write globals")?;
    Ok(path)
}
