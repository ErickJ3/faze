use glint::{Storage, get_data_dir, get_project_db_path};
use std::path::PathBuf;

pub async fn run(db_path: Option<PathBuf>, all: bool) -> Result<(), Box<dyn std::error::Error>> {
    if all {
        let data_dir = get_data_dir()?;
        println!("Cleaning all databases in: {}", data_dir.display());

        let entries = std::fs::read_dir(&data_dir)?;
        let mut count = 0;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("db") {
                if let Err(e) = std::fs::remove_file(&path) {
                    eprintln!("Failed to delete {}: {}", path.display(), e);
                } else {
                    println!("Deleted: {}", path.file_name().unwrap().to_string_lossy());
                    count += 1;
                }
            }
        }

        println!("Cleaned {} database(s)", count);
    } else {
        let final_path = db_path.unwrap_or_else(|| get_project_db_path().unwrap());
        println!("Deleting database: {}", final_path.display());

        match Storage::delete_database(&final_path) {
            Ok(()) => println!("Database deleted successfully"),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                println!("Database not found (already deleted or never created)")
            }
            Err(e) => {
                eprintln!("❌ Failed to delete database: {}", e);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
