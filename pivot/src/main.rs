use clap::{Parser, Subcommand};
use pivot::PivotDb; // <-- Change 'pivot' to your actual package name if different
use std::fs;
/// A command line interface for logging task transitions
#[derive(Parser)]
#[command(name = "ticklog")]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to the JSON database
    #[arg(long, default_value = "db.json")]
    db: String,
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    /// Log a new task transition
    Log {
        /// Tasks that just ended (comma-separated)
        #[arg(short, long, value_delimiter = ',')]
        end: Option<Vec<String>>,

        /// Tasks that are starting now (comma-separated)
        #[arg(short, long, value_delimiter = ',')]
        start: Option<Vec<String>>,
    },
    /// Export the database to the legacy markdown format
    Export {
        /// Path to output the legacy text file
        #[arg(short, long, default_value = "legacy.md")]
        output: String,
    },
}
fn main() {
    // Parse the command line arguments
    let cli = Cli::parse();

    // Load the database (calls your untouched lib.rs)
    let mut db = PivotDb::load(cli.db.clone());

    match &cli.command {
        Commands::Log { end, start } => {
            // Safely unwrap the options or default to empty vectors
            let ended_tasks = end.clone().unwrap_or_default();
            let started_tasks = start.clone().unwrap_or_default();

            if ended_tasks.is_empty() && started_tasks.is_empty() {
                println!("Nothing to log! Please provide --end or --start tasks.");
                return;
            }

            db.log_transition(ended_tasks, started_tasks);
            db.save();
            println!("✅ Successfully logged transition to {}", cli.db);
        }
        Commands::Export { output } => {
            let legacy_str = db.convert_to_legacy();
            match fs::write(output, legacy_str) {
                Ok(_) => println!("✅ Successfully exported legacy file to {}", output),
                Err(e) => eprintln!("❌ Failed to export legacy file: {}", e),
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use std::fs;
    // This imports everything from your lib.rs into the test module
    use chrono::Utc;
    use pivot::{LogEntry, PivotDb};

    #[test]
    fn legacy_main() {
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
}
