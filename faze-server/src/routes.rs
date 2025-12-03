use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use faze::Storage;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<Storage>,
}

/// Query parameters for listing traces
#[derive(Debug, Deserialize)]
pub struct ListTracesQuery {
    /// Filter by service name
    pub service: Option<String>,
    /// Minimum duration in milliseconds
    pub min_duration: Option<f64>,
    /// Maximum duration in milliseconds
    pub max_duration: Option<f64>,
    /// Maximum number of results
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
}

/// Query parameters for listing logs
#[derive(Debug, Deserialize)]
pub struct ListLogsQuery {
    /// Filter by service name
    pub service: Option<String>,
    /// Filter by severity level
    pub level: Option<String>,
    /// Maximum number of results
    pub limit: Option<usize>,
}

/// Response for trace list
#[derive(Debug, Serialize)]
pub struct TraceListResponse {
    pub traces: Vec<TraceInfo>,
    pub total: usize,
}

/// Trace information for list view
#[derive(Debug, Serialize)]
pub struct TraceInfo {
    pub trace_id: String,
    pub service_name: Option<String>,
    pub duration_ms: f64,
    pub span_count: usize,
    pub has_errors: bool,
    pub start_time: Option<i64>,
    pub root_span_name: Option<String>,
    pub root_span_kind: Option<faze::SpanKind>,
}

#[derive(Deserialize)]
pub struct ListParams {
    service: Option<String>,
    limit: Option<usize>,
}

impl From<&faze::Trace> for TraceInfo {
    fn from(trace: &faze::Trace) -> Self {
        let root_span = trace.root_span();

        Self {
            trace_id: trace.trace_id.clone(),
            service_name: trace.service_name.clone(),
            duration_ms: trace.duration_ms(),
            span_count: trace.span_count(),
            has_errors: trace.has_errors(),
            start_time: trace
                .start_time()
                .map(|dt| dt.timestamp_nanos_opt().unwrap_or(0)),
            root_span_name: root_span.map(|s| s.name.clone()),
            root_span_kind: root_span.map(|s| s.kind),
        }
    }
}

/// GET /api/traces - List all traces
pub async fn list_traces(
    State(state): State<AppState>,
    Query(params): Query<ListTracesQuery>,
) -> impl IntoResponse {
    info!("GET /api/traces - params: {:?}", params);

    let limit = params.limit.unwrap_or(100).min(1000); // Max 1000 traces

    match state
        .storage
        .list_traces(params.service.as_deref(), Some(limit))
    {
        Ok(traces) => {
            let mut filtered_traces: Vec<_> = traces
                .iter()
                .filter(|t| {
                    let duration = t.duration_ms();

                    if let Some(min) = params.min_duration
                        && duration < min
                    {
                        return false;
                    }

                    if let Some(max) = params.max_duration
                        && duration > max
                    {
                        return false;
                    }

                    true
                })
                .map(TraceInfo::from)
                .collect();

            if let Some(offset) = params.offset {
                filtered_traces = filtered_traces.into_iter().skip(offset).collect();
            }

            let total = filtered_traces.len();

            Json(TraceListResponse {
                traces: filtered_traces,
                total,
            })
            .into_response()
        }
        Err(e) => {
            error!("Failed to list traces: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to list traces: {}", e)
                })),
            )
                .into_response()
        }
    }
}

/// GET /api/traces/:id - Get a specific trace with all spans
pub async fn get_trace(
    State(state): State<AppState>,
    Path(trace_id): Path<String>,
) -> impl IntoResponse {
    info!("GET /api/traces/{}", trace_id);

    match state.storage.get_trace_by_id(&trace_id) {
        Ok(trace) => Json(trace).into_response(),
        Err(faze::StorageError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": format!("Trace not found: {}", trace_id)
            })),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to get trace {}: {}", trace_id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to get trace: {}", e)
                })),
            )
                .into_response()
        }
    }
}

/// GET /api/logs - List logs
pub async fn list_logs(
    State(state): State<AppState>,
    Query(params): Query<ListLogsQuery>,
) -> impl IntoResponse {
    info!("GET /api/logs - params: {:?}", params);

    let limit = params.limit.unwrap_or(100).min(1000);

    match state
        .storage
        .list_logs(params.service.as_deref(), Some(limit))
    {
        Ok(logs) => Json(logs).into_response(),
        Err(e) => {
            error!("Failed to list logs: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to list logs: {}", e)
                })),
            )
                .into_response()
        }
    }
}

/// GET /api/services - List unique service names
pub async fn list_services(State(state): State<AppState>) -> impl IntoResponse {
    info!("GET /api/services");

    match state.storage.list_traces(None, Some(1000)) {
        Ok(traces) => {
            let mut services: Vec<String> = traces
                .iter()
                .filter_map(|t| t.service_name.clone())
                .collect();

            services.sort();
            services.dedup();

            Json(serde_json::json!({
                "services": services
            }))
            .into_response()
        }
        Err(e) => {
            error!("Failed to list services: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to list services: {}", e)
                })),
            )
                .into_response()
        }
    }
}

/// GET /api/metrics - List metrics
pub async fn list_metrics(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> impl IntoResponse {
    let metrics = state
        .storage
        .list_metrics(params.service.as_deref(), params.limit)
        .unwrap_or_default();

    Json(serde_json::json!({
        "metrics": metrics
    }))
}

/// GET /health - Health check endpoint
pub async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "service": "faze-api"
    }))
}

/// GET /api/project - Get current project information
pub async fn get_project_info() -> impl IntoResponse {
    use faze::storage::detect_project_root;

    let project_root = detect_project_root();
    let project_name = project_root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    let project_path = project_root.to_string_lossy().to_string();

    Json(serde_json::json!({
        "name": project_name,
        "path": project_path,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use faze::models::{Attributes, Span, SpanKind, Status};

    #[tokio::test]
    async fn test_health_check() {
        let response = health_check().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_traces_empty() {
        let storage = Storage::new_in_memory().unwrap();
        let state = AppState {
            storage: Arc::new(storage),
        };

        let query = ListTracesQuery {
            service: None,
            min_duration: None,
            max_duration: None,
            limit: None,
            offset: None,
        };

        let response = list_traces(State(state), Query(query))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_trace_not_found() {
        let storage = Storage::new_in_memory().unwrap();
        let state = AppState {
            storage: Arc::new(storage),
        };

        let response = get_trace(State(state), Path("nonexistent".to_string()))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_trace_success() {
        let storage = Storage::new_in_memory().unwrap();
        let span = Span::new(
            "span1".to_string(),
            "trace1".to_string(),
            None,
            "test operation".to_string(),
            SpanKind::Server,
            1_000_000_000,
            2_000_000_000,
            Attributes::new(),
            Status::ok(),
            Some("test-service".to_string()),
        );
        storage.insert_span(&span).unwrap();

        let state = AppState {
            storage: Arc::new(storage),
        };

        let response = get_trace(State(state), Path("trace1".to_string()))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_traces_with_filters() {
        let storage = Storage::new_in_memory().unwrap();
        for i in 0..5 {
            let span = Span::new(
                format!("span{}", i),
                format!("trace{}", i),
                None,
                format!("operation{}", i),
                SpanKind::Server,
                1_000_000_000,
                1_000_000_000 + (i as i64 * 100_000_000), // Different durations
                Attributes::new(),
                Status::ok(),
                Some("test-service".to_string()),
            );
            storage.insert_span(&span).unwrap();
        }

        let state = AppState {
            storage: Arc::new(storage),
        };
        let query = ListTracesQuery {
            service: None,
            min_duration: Some(200.0),
            max_duration: None,
            limit: None,
            offset: None,
        };

        let response = list_traces(State(state.clone()), Query(query))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_traces_with_service_filter() {
        let storage = Storage::new_in_memory().unwrap();
        let span1 = Span::new(
            "span1".to_string(),
            "trace1".to_string(),
            None,
            "op1".to_string(),
            SpanKind::Server,
            1_000_000_000,
            2_000_000_000,
            Attributes::new(),
            Status::ok(),
            Some("service-a".to_string()),
        );
        storage.insert_span(&span1).unwrap();

        let span2 = Span::new(
            "span2".to_string(),
            "trace2".to_string(),
            None,
            "op2".to_string(),
            SpanKind::Server,
            1_000_000_000,
            2_000_000_000,
            Attributes::new(),
            Status::ok(),
            Some("service-b".to_string()),
        );
        storage.insert_span(&span2).unwrap();

        let state = AppState {
            storage: Arc::new(storage),
        };

        let query = ListTracesQuery {
            service: Some("service-a".to_string()),
            min_duration: None,
            max_duration: None,
            limit: None,
            offset: None,
        };

        let response = list_traces(State(state), Query(query))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_traces_with_pagination() {
        let storage = Storage::new_in_memory().unwrap();
        for i in 0..20 {
            let span = Span::new(
                format!("span{}", i),
                format!("trace{}", i),
                None,
                format!("operation{}", i),
                SpanKind::Server,
                1_000_000_000 + (i as i64 * 1000),
                2_000_000_000 + (i as i64 * 1000),
                Attributes::new(),
                Status::ok(),
                Some("test-service".to_string()),
            );
            storage.insert_span(&span).unwrap();
        }

        let state = AppState {
            storage: Arc::new(storage),
        };

        let query = ListTracesQuery {
            service: None,
            min_duration: None,
            max_duration: None,
            limit: Some(10),
            offset: None,
        };

        let response = list_traces(State(state.clone()), Query(query))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let query = ListTracesQuery {
            service: None,
            min_duration: None,
            max_duration: None,
            limit: Some(5),
            offset: Some(5),
        };

        let response = list_traces(State(state), Query(query))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_services() {
        let storage = Storage::new_in_memory().unwrap();
        let services = ["service-a", "service-b", "service-c", "service-a"];
        for (i, service) in services.iter().enumerate() {
            let span = Span::new(
                format!("span{}", i),
                format!("trace{}", i),
                None,
                format!("operation{}", i),
                SpanKind::Server,
                1_000_000_000,
                2_000_000_000,
                Attributes::new(),
                Status::ok(),
                Some(service.to_string()),
            );
            storage.insert_span(&span).unwrap();
        }

        let state = AppState {
            storage: Arc::new(storage),
        };

        let response = list_services(State(state)).await.into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_logs_empty() {
        let storage = Storage::new_in_memory().unwrap();
        let state = AppState {
            storage: Arc::new(storage),
        };

        let query = ListLogsQuery {
            service: None,
            level: None,
            limit: None,
        };

        let response = list_logs(State(state), Query(query)).await.into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_trace_with_multiple_spans() {
        let storage = Storage::new_in_memory().unwrap();
        let parent_span = Span::new(
            "parent-span".to_string(),
            "multi-span-trace".to_string(),
            None,
            "parent operation".to_string(),
            SpanKind::Server,
            1_000_000_000,
            3_000_000_000,
            Attributes::new(),
            Status::ok(),
            Some("test-service".to_string()),
        );
        storage.insert_span(&parent_span).unwrap();

        let child_span = Span::new(
            "child-span".to_string(),
            "multi-span-trace".to_string(),
            Some("parent-span".to_string()),
            "child operation".to_string(),
            SpanKind::Client,
            1_500_000_000,
            2_500_000_000,
            Attributes::new(),
            Status::ok(),
            Some("test-service".to_string()),
        );
        storage.insert_span(&child_span).unwrap();

        let state = AppState {
            storage: Arc::new(storage),
        };

        let response = get_trace(State(state), Path("multi-span-trace".to_string()))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_trace_with_error_span() {
        let storage = Storage::new_in_memory().unwrap();
        let error_span = Span::new(
            "error-span".to_string(),
            "error-trace".to_string(),
            None,
            "failing operation".to_string(),
            SpanKind::Server,
            1_000_000_000,
            2_000_000_000,
            Attributes::new(),
            Status::error("Something went wrong"),
            Some("test-service".to_string()),
        );
        storage.insert_span(&error_span).unwrap();

        let state = AppState {
            storage: Arc::new(storage),
        };

        let response = get_trace(State(state), Path("error-trace".to_string()))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_traces_max_duration_filter() {
        let storage = Storage::new_in_memory().unwrap();
        let fast_span = Span::new(
            "fast".to_string(),
            "fast-trace".to_string(),
            None,
            "fast op".to_string(),
            SpanKind::Server,
            1_000_000_000,
            1_050_000_000, // 50ms
            Attributes::new(),
            Status::ok(),
            Some("test-service".to_string()),
        );
        storage.insert_span(&fast_span).unwrap();

        let slow_span = Span::new(
            "slow".to_string(),
            "slow-trace".to_string(),
            None,
            "slow op".to_string(),
            SpanKind::Server,
            1_000_000_000,
            1_500_000_000, // 500ms
            Attributes::new(),
            Status::ok(),
            Some("test-service".to_string()),
        );
        storage.insert_span(&slow_span).unwrap();

        let state = AppState {
            storage: Arc::new(storage),
        };

        let query = ListTracesQuery {
            service: None,
            min_duration: None,
            max_duration: Some(100.0),
            limit: None,
            offset: None,
        };

        let response = list_traces(State(state), Query(query))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_trace_info_conversion() {
        let storage = Storage::new_in_memory().unwrap();
        let mut attrs = Attributes::new();
        attrs.insert("http.method", "POST");
        attrs.insert("http.status_code", 201i64);

        let span = Span::new(
            "test-span".to_string(),
            "test-trace".to_string(),
            None,
            "test operation".to_string(),
            SpanKind::Server,
            1_000_000_000_000_000_000,
            1_000_000_000_100_000_000,
            attrs,
            Status::ok(),
            Some("test-service".to_string()),
        );
        storage.insert_span(&span).unwrap();

        let trace = storage.get_trace_by_id("test-trace").unwrap();
        let trace_info = TraceInfo::from(&trace);

        assert_eq!(trace_info.trace_id, "test-trace");
        assert_eq!(trace_info.service_name, Some("test-service".to_string()));
        assert_eq!(trace_info.span_count, 1);
        assert!(!trace_info.has_errors);
        assert_eq!(trace_info.duration_ms, 100.0);
        assert_eq!(
            trace_info.root_span_name,
            Some("test operation".to_string())
        );
        assert_eq!(trace_info.root_span_kind, Some(SpanKind::Server));
    }

    #[tokio::test]
    async fn test_list_traces_limit_enforcement() {
        let storage = Storage::new_in_memory().unwrap();
        let state = AppState {
            storage: Arc::new(storage),
        };
        let query = ListTracesQuery {
            service: None,
            min_duration: None,
            max_duration: None,
            limit: Some(5000),
            offset: None,
        };
        let response = list_traces(State(state), Query(query))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
