use colored::*;
use faze::{Storage, get_data_dir, get_project_db_path};
use std::path::PathBuf;

pub async fn run(db_path: Option<PathBuf>, all: bool) -> Result<(), Box<dyn std::error::Error>> {
    if all {
        let data_dir = get_data_dir()?;
        println!("\n{}", "Cleaning All Databases".yellow().bold());
        println!("  Location: {}", data_dir.display().to_string().dimmed());

        let entries = std::fs::read_dir(&data_dir)?;
        let mut count = 0;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("db") {
                if let Err(e) = std::fs::remove_file(&path) {
                    println!(
                        "  {} {}: {}",
                        "✗".red(),
                        path.file_name().unwrap().to_string_lossy(),
                        e.to_string().dimmed()
                    );
                } else {
                    println!(
                        "  {} {}",
                        "✓".green(),
                        path.file_name().unwrap().to_string_lossy().bright_white()
                    );
                    count += 1;
                }
            }
        }

        println!(
            "\n{} {}",
            "Cleaned".green().bold(),
            format!("{} database(s)", count).cyan()
        );
    } else {
        let final_path = db_path.unwrap_or_else(|| get_project_db_path().unwrap());
        println!("\n{}", "Deleting Database".yellow().bold());
        println!("  Path: {}", final_path.display().to_string().dimmed());

        match Storage::delete_database(&final_path) {
            Ok(()) => println!("\n{}", "Database deleted".green().bold()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                println!(
                    "\n{}",
                    "Database not found (already deleted or never created)".yellow()
                )
            }
            Err(e) => {
                println!("\n{} {}", "Failed to delete database:".red().bold(), e);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
