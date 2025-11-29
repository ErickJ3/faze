use crate::{
    routes::{AppState, get_trace, health_check, list_logs, list_services, list_traces},
    ui,
};
use axum::{Router, routing::get};
use glint::Storage;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::info;

/// API server that exposes REST endpoints for querying observability data
pub struct ApiServer {
    storage: Arc<Storage>,
    port: u16,
}

impl ApiServer {
    /// Create a new API server
    pub fn new(storage: Storage, port: u16) -> Self {
        Self {
            storage: Arc::new(storage),
            port,
        }
    }

    /// Build the router with all routes
    fn build_router(&self) -> Router {
        let state = AppState {
            storage: self.storage.clone(),
        };

        Router::new()
            // Health check
            .route("/health", get(health_check))
            // Trace routes
            .route("/api/traces", get(list_traces))
            .route("/api/traces/{id}", get(get_trace))
            // Log routes
            .route("/api/logs", get(list_logs))
            // Service routes
            .route("/api/services", get(list_services))
            // Enable CORS for browser access
            .layer(CorsLayer::permissive())
            .with_state(state)
            .fallback(ui::fallback_service())
    }

    /// Start the API server
    pub async fn serve(self) -> Result<(), Box<dyn std::error::Error>> {
        let app = self.build_router();
        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));

        info!("Starting API server on {}", addr);
        info!("  Health check:  http://localhost:{}/health", self.port);
        info!("  List traces:   http://localhost:{}/api/traces", self.port);
        info!(
            "  Get trace:     http://localhost:{}/api/traces/:id",
            self.port
        );
        info!("  List logs:     http://localhost:{}/api/logs", self.port);
        info!(
            "  List services: http://localhost:{}/api/services",
            self.port
        );

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use glint::models::{Attributes, Span, SpanKind, Status};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_endpoint() {
        let storage = Storage::new_in_memory().unwrap();
        let server = ApiServer::new(storage, 0);
        let app = server.build_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_traces_endpoint() {
        let storage = Storage::new_in_memory().unwrap();
        let span = Span::new(
            "span1".to_string(),
            "trace1".to_string(),
            None,
            "test".to_string(),
            SpanKind::Server,
            1_000_000_000,
            2_000_000_000,
            Attributes::new(),
            Status::ok(),
            Some("test-service".to_string()),
        );
        storage.insert_span(&span).unwrap();

        let server = ApiServer::new(storage, 0);
        let app = server.build_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/traces")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_trace_endpoint() {
        let storage = Storage::new_in_memory().unwrap();
        let span = Span::new(
            "span1".to_string(),
            "trace123".to_string(),
            None,
            "test".to_string(),
            SpanKind::Server,
            1_000_000_000,
            2_000_000_000,
            Attributes::new(),
            Status::ok(),
            Some("test-service".to_string()),
        );
        storage.insert_span(&span).unwrap();

        let server = ApiServer::new(storage, 0);
        let app = server.build_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/traces/trace123")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_trace_not_found() {
        let storage = Storage::new_in_memory().unwrap();
        let server = ApiServer::new(storage, 0);
        let app = server.build_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/traces/nonexistent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
