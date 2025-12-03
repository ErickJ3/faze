use colored::*;
use faze::Storage;
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

    if logs.is_empty() {
        println!("{}", "No logs found".yellow());
        return Ok(());
    }

    for log in logs {
        let severity = log.severity_level.as_str();
        let colored_severity = match severity {
            "ERROR" | "FATAL" => severity.red().bold(),
            "WARN" => severity.yellow(),
            "INFO" => severity.cyan(),
            "DEBUG" => severity.dimmed(),
            _ => severity.normal(),
        };
        let service = log.service_name.as_deref().unwrap_or("unknown");

        println!(
            "[{}] {} - {}",
            colored_severity,
            service.bright_white(),
            log.body
        );
    }

    Ok(())
}
