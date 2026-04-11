use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs; // 📁 Added for file system operations
/// Represents a single moment in time when tasks transition.
///
/// This is the core data structure of the application. Instead of
/// tracking duration, it logs a "tick" where one or more tasks end,
/// and one or more tasks begin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// The exact UTC timestamp of the transition.
    pub timestamp: DateTime<Utc>,

    /// A list of tasks that were just completed. Can be empty.
    pub ended_tasks: Vec<String>,

    /// A list of tasks that are starting now. Can be empty.
    pub started_tasks: Vec<String>,
}
/// The main database holding all log entries.
#[derive(Debug, Serialize, Deserialize)]
pub struct PivotDb {
    pub entries: Vec<LogEntry>,
    // We add a field to remember where this database is saved
    #[serde(skip)] // We don't need to save the path inside the JSON itself
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
