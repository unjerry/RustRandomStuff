use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

fn main() {
    println!("Ticklog running...");

    // Example usage to ensure the data structure is functioning
    let example_entry = LogEntry {
        timestamp: Utc::now(),
        ended_tasks: vec!["吃早餐".to_string()],
        started_tasks: vec!["rust".to_string(), "mmd_tidyup".to_string()],
    };
    println!("Current transition: {:?}", example_entry);
}
