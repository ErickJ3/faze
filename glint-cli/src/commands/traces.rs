use glint::Storage;
use std::path::PathBuf;

pub async fn run(slow: bool, db_path: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let storage = if let Some(path) = db_path {
        Storage::new_with_path(&path)?
    } else {
        Storage::new()?
    };

    let traces = storage.list_traces(None, Some(100))?;

    for trace in traces {
        if !slow || trace.duration_ms() > 100.0 {
            println!(
                "[{}] {} - {:.2}ms - {} spans{}",
                trace.trace_id,
                trace.service_name.as_deref().unwrap_or("unknown"),
                trace.duration_ms(),
                trace.span_count(),
                if trace.has_errors() { " [ERROR]" } else { "" }
            );
        }
    }

    Ok(())
}
