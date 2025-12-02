use colored::*;
use faze::{Storage, detect_project_root, get_data_dir, get_project_db_path};

fn format_number(n: i64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn format_size(bytes: u64) -> String {
    let mb = bytes as f64 / 1_024_000.0;
    if mb >= 1000.0 {
        format!("{:.2} GB", mb / 1000.0)
    } else {
        format!("{:.2} MB", mb)
    }
}

fn get_size_status(mb: f64) -> (String, &'static str) {
    if mb > 500.0 {
        ("⚠ Large".to_string(), "red")
    } else if mb > 100.0 {
        ("● Growing".to_string(), "yellow")
    } else {
        ("✓ Good".to_string(), "green")
    }
}

fn shorten_path(path: &std::path::Path) -> String {
    let path_str = path.display().to_string();
    if let Ok(home) = std::env::var("HOME") {
        path_str.replace(&home, "~")
    } else {
        path_str
    }
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let project_root = detect_project_root();
    let db_path = get_project_db_path()?;

    println!("\n{}", "Database Info".bright_cyan().bold());
    println!("  Project:  {}", shorten_path(&project_root).bright_white());

    if db_path.exists() {
        let metadata = std::fs::metadata(&db_path)?;
        let size_mb = metadata.len() as f64 / 1_024_000.0;
        let size_str = format_size(metadata.len());
        let (status, status_color) = get_size_status(size_mb);

        let colored_size = match status_color {
            "red" => size_str.red(),
            "yellow" => size_str.yellow(),
            _ => size_str.green(),
        };

        let colored_status = match status_color {
            "red" => status.red().bold(),
            "yellow" => status.yellow(),
            _ => status.green(),
        };

        print!("  {} {} ", "Size:".dimmed(), colored_size);
        println!("{}", format!("({})", colored_status).dimmed());

        if let Ok(storage) = Storage::new() {
            let mut total_items = 0;
            let mut items = vec![];

            if let Ok(span_count) = storage.count_spans() {
                total_items += span_count;
                items.push(format!("{} spans", format_number(span_count)));
            }
            if let Ok(log_count) = storage.count_logs() {
                total_items += log_count;
                items.push(format!("{} logs", format_number(log_count)));
            }
            if let Ok(metric_count) = storage.count_metrics() {
                total_items += metric_count;
                items.push(format!("{} metrics", format_number(metric_count)));
            }

            println!("  {} {}", "Items:".dimmed(), items.join(" • ").cyan());

            if size_mb > 500.0 {
                println!(
                    "\n  {} Database very large. Run {} to clean.",
                    "⚠".yellow(),
                    "faze clean --all".cyan()
                );
            } else if size_mb > 100.0 {
                println!(
                    "\n  {} Database growing. Consider {}",
                    "→".dimmed(),
                    "faze clean".cyan()
                );
            } else if total_items > 100_000 {
                println!(
                    "\n  {} {} items. Consider cleanup.",
                    "→".dimmed(),
                    format_number(total_items).cyan()
                );
            }
        }
    } else {
        println!("  {} {}", "Status:".dimmed(), "not created".yellow());
        println!(
            "\n  {} Run {} and send OTLP data to localhost:4317/4318",
            "→".dimmed(),
            "faze serve".cyan()
        );
    }

    let data_dir = get_data_dir()?;
    let entries = std::fs::read_dir(&data_dir)?;
    let mut databases: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("db"))
        .collect();

    if !databases.is_empty() {
        println!("\n{}", "Other Databases".yellow().bold());

        databases.sort_by(|a, b| {
            let size_a = std::fs::metadata(a.path()).map(|m| m.len()).unwrap_or(0);
            let size_b = std::fs::metadata(b.path()).map(|m| m.len()).unwrap_or(0);
            size_b.cmp(&size_a)
        });

        let current_db_name = db_path.file_name().unwrap().to_string_lossy().to_string();

        for entry in databases.iter().take(5) {
            let path = entry.path();
            if let Ok(metadata) = std::fs::metadata(&path) {
                let name = path.file_name().unwrap().to_string_lossy();

                if name == current_db_name {
                    continue;
                }

                let size_str = format_size(metadata.len());
                let size_mb = metadata.len() as f64 / 1_024_000.0;
                let size_colored = if size_mb > 100.0 {
                    size_str.yellow()
                } else {
                    size_str.dimmed()
                };

                println!("  {}  {}", name.to_string().normal(), size_colored);
            }
        }

        if databases.len() > 6 {
            println!("  {} +{} more...", "...".dimmed(), databases.len() - 6);
        }
    }

    Ok(())
}
