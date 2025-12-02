use colored::*;
use faze::{Storage, detect_project_root, get_project_db_path};
use faze_collector::grpc::{logs, metrics, traces};
use std::path::PathBuf;

pub async fn run(
    port: u16,
    grpc_port: u16,
    db_path: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::sync::Arc;

    let (storage, project_root, db_path_display) = if let Some(path) = db_path {
        let storage = Storage::new_with_path(&path)?;
        let project_root = detect_project_root();
        (storage, project_root, path.display().to_string())
    } else {
        let detected_path = get_project_db_path()?;
        let project_root = detect_project_root();
        let storage = Storage::new()?;
        (storage, project_root, detected_path.display().to_string())
    };

    println!("\n{}", "Starting Faze".bright_cyan().bold());
    println!(
        "  Project:  {}",
        project_root.display().to_string().bright_white()
    );
    println!("  Database: {}", db_path_display.dimmed());
    println!("  Storage:  {}", "ready".green());

    let storage_arc = Arc::new(storage.clone());

    let spans_collector = traces::OtlpSpansCollector::new(storage.clone());
    let spans_grpc_service = spans_collector.into_service();

    let logs_collector = logs::OtlpLogsCollector::new(storage.clone());
    let logs_grpc_service = logs_collector.into_service();

    let metrics_collector = metrics::OtlpMetricsCollector::new(storage.clone());
    let metrics_grpc_service = metrics_collector.into_service();

    let grpc_addr = format!("0.0.0.0:{}", grpc_port).parse()?;
    let grpc_server = tonic::transport::Server::builder()
        .add_service(spans_grpc_service)
        .add_service(logs_grpc_service)
        .add_service(metrics_grpc_service)
        .serve(grpc_addr);

    let http_collector_router = faze_collector::create_router(storage_arc);
    let http_collector_addr = "0.0.0.0:4318";

    let http_collector_listener = tokio::net::TcpListener::bind(http_collector_addr).await?;
    let http_collector_task = tokio::spawn(async move {
        if let Err(e) = axum::serve(http_collector_listener, http_collector_router).await {
            tracing::error!("HTTP collector server error: {}", e);
        }
    });

    let api_server = faze_server::ApiServer::new(storage, port);
    let api_task = tokio::spawn(async move {
        if let Err(e) = api_server.serve().await {
            tracing::error!("API server error: {}", e);
        }
    });

    println!("\n{}", "Listeners".yellow().bold());
    println!("  OTLP gRPC  {}", format!("0.0.0.0:{}", grpc_port).cyan());
    println!("  OTLP HTTP  {}", "0.0.0.0:4318".cyan());
    println!("  API Server {}", format!("0.0.0.0:{}", port).cyan());

    println!("\n{}", "Ready".green().bold());
    println!(
        "  Web UI    {}",
        format!("http://localhost:{}", port).cyan()
    );
    println!(
        "  API       {}",
        format!("http://localhost:{}/api", port).dimmed()
    );
    println!(
        "  Health    {}",
        format!("http://localhost:{}/health", port).dimmed()
    );

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
            println!("\n{}", "Shutting down...".yellow());
        }
    }

    Ok(())
}
