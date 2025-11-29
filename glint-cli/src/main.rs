use clap::{Parser, Subcommand};
use glint::{Storage, detect_project_root, get_config_dir, get_data_dir, get_project_db_path};
use glint_collector::OtlpCollector;
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "glint")]
#[command(about = "Local-first observability for developers")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the collector and web UI
    Serve {
        /// HTTP/UI port
        #[arg(short, long, default_value = "7070")]
        port: u16,

        /// gRPC collector port
        #[arg(long, default_value = "4317")]
        grpc_port: u16,

        /// Custom database file path (auto-detected by default)
        #[arg(long)]
        db_path: Option<PathBuf>,
    },
    /// Query traces
    Traces {
        #[arg(long)]
        slow: bool,

        /// Custom database file path (auto-detected by default)
        #[arg(long)]
        db_path: Option<PathBuf>,
    },
    /// Query logs
    Logs {
        #[arg(long)]
        service: Option<String>,

        /// Custom database file path (auto-detected by default)
        #[arg(long)]
        db_path: Option<PathBuf>,
    },
    /// Clean database (delete the .db file for current project)
    Clean {
        /// Custom database file path to delete (auto-detected by default)
        #[arg(long)]
        db_path: Option<PathBuf>,

        /// Clean all databases in config directory
        #[arg(long)]
        all: bool,
    },
    /// Show database information
    Info,
    /// Open TUI
    Tui,
}

async fn serve(
    port: u16,
    grpc_port: u16,
    db_path: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::sync::Arc;

    let storage = if let Some(path) = db_path {
        info!("Using custom database at: {}", path.display());
        Storage::new_with_path(&path)?
    } else {
        let detected_path = get_project_db_path()?;
        let project_root = detect_project_root();
        info!("Project detected: {}", project_root.display());
        info!("Database: {}", detected_path.display());
        Storage::new()?
    };
    info!("Storage initialized");

    let storage_arc = Arc::new(storage.clone());
    let collector = OtlpCollector::new(storage.clone());
    let grpc_service = collector.into_service();
    let grpc_addr = format!("0.0.0.0:{}", grpc_port).parse()?;
    info!("Starting OTLP gRPC collector on {}", grpc_addr);

    let grpc_server = tonic::transport::Server::builder()
        .add_service(grpc_service)
        .serve(grpc_addr);

    let http_collector_router = glint_collector::create_router(storage_arc);
    let http_collector_addr = "0.0.0.0:4318";
    info!("Starting OTLP HTTP collector on {}", http_collector_addr);

    let http_collector_listener = tokio::net::TcpListener::bind(http_collector_addr).await?;
    let http_collector_task = tokio::spawn(async move {
        if let Err(e) = axum::serve(http_collector_listener, http_collector_router).await {
            tracing::error!("HTTP collector server error: {}", e);
        }
    });

    let api_server = glint_server::ApiServer::new(storage, port);
    let api_task = tokio::spawn(async move {
        if let Err(e) = api_server.serve().await {
            tracing::error!("API server error: {}", e);
        }
    });

    info!("Glint is ready!");
    info!("  OTLP gRPC endpoint: http://localhost:{}", grpc_port);
    info!("  OTLP HTTP endpoint: http://localhost:4318");
    info!("  REST API:           http://localhost:{}/api", port);
    info!("  Health check:       http://localhost:{}/health", port);
    info!("  Web UI:             http://localhost:{}", port);

    tokio::select! {
        result = grpc_server => {
            if let Err(e) = result {
                tracing::error!("gRPC server error: {}", e);
            }
        }
        result = http_collector_task => {
            if let Err(e) = result {
                tracing::error!("HTTP collector task error: {}", e);
            }
        }
        result = api_task => {
            if let Err(e) = result {
                tracing::error!("API task error: {}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Shutting down gracefully...");
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Serve {
            port,
            grpc_port,
            db_path,
        } => {
            serve(port, grpc_port, db_path).await?;
        }
        Commands::Traces { slow, db_path } => {
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
        }
        Commands::Logs { service, db_path } => {
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
        }
        Commands::Clean { db_path, all } => {
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
                let db_path = db_path.unwrap_or_else(|| get_project_db_path().unwrap());
                println!("Deleting database: {}", db_path.display());

                match Storage::delete_database(&db_path) {
                    Ok(()) => {
                        println!("Database deleted successfully");
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                        println!("Database not found (already deleted or never created)");
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to delete database: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
        Commands::Info => {
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
        }
        Commands::Tui => {
            println!("TUI not implemented yet");
        }
    }

    Ok(())
}
