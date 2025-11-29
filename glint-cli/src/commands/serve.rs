use glint::{Storage, detect_project_root, get_project_db_path};
use glint_collector::OtlpCollector;
use std::path::PathBuf;
use tracing::info;

pub async fn run(
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
