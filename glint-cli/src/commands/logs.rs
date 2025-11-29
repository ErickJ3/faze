use glint::Storage;
use std::path::PathBuf;

pub async fn run(
    service: Option<String>,
    db_path: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let storage = if let Some(path) = db_path {
        Storage::new_with_path(&path)?
    } else {
        Storage::new()?
    };

    let logs = storage.list_logs(service.as_deref(), Some(100))?;

    for log in logs {
        println!(
            "[{}] {} - {}",
            log.severity_level.as_str(),
            log.service_name.as_deref().unwrap_or("unknown"),
            log.body
        );
    }

    Ok(())
}
