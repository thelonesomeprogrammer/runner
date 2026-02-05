use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Serialize, Deserialize, Default)]
pub struct History {
    pub usage_counts: HashMap<String, u32>,
}

pub fn get_history_path() -> Option<PathBuf> {
    ProjectDirs::from("org", "runner", "runner")
        .map(|dirs| {
            let data_dir = dirs.data_dir();
            let _ = fs::create_dir_all(data_dir);
            data_dir.join("history.json")
        })
}

pub fn load_history() -> History {
    if let Some(path) = get_history_path() {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(history) = serde_json::from_str(&content) {
                return history;
            }
        }
    }
    History::default()
}

pub fn save_history(history: &History) -> Result<()> {
    if let Some(path) = get_history_path() {
        let content = serde_json::to_string_pretty(history)?;
        fs::write(path, content)?;
    }
    Ok(())
}

pub fn increment_usage(id: &str) -> Result<()> {
    let mut history = load_history();
    let count = history.usage_counts.entry(id.to_string()).or_insert(0);
    *count += 1;
    save_history(&history)
}