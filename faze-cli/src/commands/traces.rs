use colored::*;
use faze::Storage;
use std::path::PathBuf;

pub async fn run(slow: bool, db_path: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let storage = if let Some(path) = db_path {
        Storage::new_with_path(&path)?
    } else {
        Storage::new()?
    };

    let traces = storage.list_traces(None, Some(100))?;

    if traces.is_empty() {
        println!("{}", "No traces found".yellow());
        return Ok(());
    }

    for trace in traces {
        if !slow || trace.duration_ms() > 100.0 {
            let duration = trace.duration_ms();
            let duration_colored = if duration > 1000.0 {
                format!("{:.2}ms", duration).red()
            } else if duration > 100.0 {
                format!("{:.2}ms", duration).yellow()
            } else {
                format!("{:.2}ms", duration).green()
            };

            let service = trace
                .service_name
                .as_deref()
                .unwrap_or("unknown")
                .bright_white();
            let span_count = format!("{} spans", trace.span_count()).dimmed();
            let error_badge = if trace.has_errors() {
                format!(" {}", "[ERROR]".red().bold())
            } else {
                String::new()
            };

            println!(
                "[{}] {} - {} - {}{}",
                trace.trace_id[..8].dimmed(),
                service,
                duration_colored,
                span_count,
                error_badge
            );
        }
    }

    Ok(())
}
