use chrono::{DateTime, Utc};
use flutter_rust_bridge::frb; // 1. 🌉 Import the bridge macros
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub ended_tasks: Vec<String>,
    pub started_tasks: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[frb(opaque)] // 2. 🛡️ Tell the bridge to keep this safely in Rust memory
pub struct PivotDb {
    pub entries: Vec<LogEntry>,
    #[serde(skip)]
    pub file_path: String,
}

impl PivotDb {
    pub fn load(path: String) -> Self {
        if let Ok(json_data) = fs::read_to_string(&path) {
            if let Ok(entries) = serde_json::from_str(&json_data) {
                return PivotDb {
                    entries,
                    file_path: path,
                };
            }
        }
        PivotDb {
            entries: Vec::new(),
            file_path: path,
        }
    }

    pub fn log_transition(&mut self, ended_tasks: Vec<String>, started_tasks: Vec<String>) {
        let entry = LogEntry {
            timestamp: Utc::now(),
            ended_tasks,
            started_tasks,
        };
        self.entries.push(entry);
    }

    pub fn save(&self) {
        if let Ok(json_data) = serde_json::to_string_pretty(&self.entries) {
            let _ = fs::write(&self.file_path, json_data);
        }
    }
}
