use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs; // 📁 Added for file system operations

/// Represents a single moment in time when tasks transition.
///
/// This is the core data structure of the application. Instead of
/// tracking duration, it logs a "tick" where one or more tasks end,
/// and one or more tasks begin.
#[derive(Debug, Serialize, Deserialize)]
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
    /// Creates a new database linked to a specific file path.
    pub fn new(path: &str) -> Self {
        PivotDb {
            entries: Vec::new(),
            file_path: path.to_string(),
        }
    }
    /// Adds a new log entry to the in-memory list.
    pub fn add_entry(&mut self, entry: LogEntry) {
        self.entries.push(entry);
    }

    /// Saves the current entries to the JSON file.
    pub fn save(&self) {
        // Convert the entries list into a nicely formatted JSON string
        let json_data =
            serde_json::to_string_pretty(&self.entries).expect("Failed to serialize data");

        // Write the string to the file
        fs::write(&self.file_path, json_data).expect("Failed to write to file");
        println!("💾 Database saved to {}", self.file_path);
    }

    /// Attempts to load an existing database, or creates a new one if the file doesn't exist.
    pub fn load(path: &str) -> Self {
        if let Ok(json_data) = fs::read_to_string(path) {
            if let Ok(entries) = serde_json::from_str(&json_data) {
                println!("📂 Loaded existing database from {}", path);
                return PivotDb {
                    entries,
                    file_path: path.to_string(),
                };
            }
        }

        println!("✨ Creating new database at {}", path);
        PivotDb::new(path)
    }
    /// Automatically logs a transition with the current exact time
    pub fn log_transition(&mut self, ended_tasks: Vec<String>, started_tasks: Vec<String>) {
        let entry = LogEntry {
            timestamp: Utc::now(), // Grabs the exact current time ⏱️
            ended_tasks,
            started_tasks,
        };

        self.add_entry(entry);
    }
}
fn main() {
    println!("Ticklog running...");

    // Example usage to ensure the data structure is functioning
    let example_entry = LogEntry {
        timestamp: Utc::now(),
        ended_tasks: vec!["吃早餐".to_string()],
        started_tasks: vec!["rust".to_string(), "mmd_tidyup".to_string()],
    };
    println!("Current transition: {:?}", example_entry);

    let mut db = PivotDb::load("db.json");
    db.log_transition(vec![], vec!["rust".to_string()]);
    db.log_transition(vec!["rust".to_string()], vec![]);
    db.save();
}
