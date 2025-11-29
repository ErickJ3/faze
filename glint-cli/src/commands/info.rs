use glint::{Storage, detect_project_root, get_config_dir, get_data_dir, get_project_db_path};

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = get_config_dir()?;
    let data_dir = get_data_dir()?;
    let project_root = detect_project_root();
    let db_path = get_project_db_path()?;

    println!("Glint Database Information\n");
    println!("Project root:     {}", project_root.display());
    println!("Database file:    {}", db_path.display());
    println!("Config directory: {}", config_dir.display());
    println!("Data directory:   {}", data_dir.display());

    if db_path.exists() {
        let metadata = std::fs::metadata(&db_path)?;
        let size_mb = metadata.len() as f64 / 1_024_000.0;
        println!("Database size:    {:.2} MB", size_mb);

        if let Ok(storage) = Storage::new() {
            if let Ok(span_count) = storage.count_spans() {
                println!("Total spans:      {}", span_count);
            }
            if let Ok(log_count) = storage.count_logs() {
                println!("Total logs:       {}", log_count);
            }
        }
    } else {
        println!("Database not yet created");
    }

    println!("\nAll databases:");

    let entries = std::fs::read_dir(&data_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("db") {
            let metadata = std::fs::metadata(&path)?;
            let size_mb = metadata.len() as f64 / 1_024_000.0;
            let name = path.file_name().unwrap().to_string_lossy();
            println!("  • {} ({:.2} MB)", name, size_mb);
        }
    }

    Ok(())
}
