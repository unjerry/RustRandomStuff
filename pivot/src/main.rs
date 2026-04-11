use chrono::Utc;
use pivot::{LogEntry, PivotDb};
use std::fs;
fn main() {
    println!("Ticklog running...");

    // Example usage to ensure the data structure is functioning
    let example_entry = LogEntry {
        timestamp: Utc::now(),
        ended_tasks: vec!["吃早餐".to_string()],
        started_tasks: vec!["rust".to_string(), "mmd_tidyup".to_string()],
    };
    println!("Current transition: {:?}", example_entry);

    let mut db = PivotDb::load("db.json".to_string());
    db.log_transition(vec![], vec!["rust".to_string()]);
    db.log_transition(vec!["rust".to_string()], vec![]);
    db.save();
    let legdb = PivotDb::load("pivot_data_backup20260411.json".to_string());
    let str = legdb.convert_to_legacy();
    println!("{}", str);
    fs::write("legacy.md", str).ok();
}
