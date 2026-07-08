//! The API route handlers.

use super::AppState;
use super::dto::{
    ListLogsQuery, ListMetricsQuery, ListTracesQuery, StatsResponse, TraceInfo, TraceListResponse,
};
use super::error::ApiError;
use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use tracing::info;

/// GET /api/traces - List all traces
pub async fn list_traces(
    State(state): State<AppState>,
    Query(params): Query<ListTracesQuery>,
) -> Result<impl IntoResponse, ApiError> {
    info!("GET /api/traces - params: {:?}", params);

    let limit = params.limit.unwrap_or(100).min(1000); // Max 1000 traces
    let traces = state
        .storage
        .list_traces(params.service.as_deref(), Some(limit))?;

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

    Ok(Json(TraceListResponse {
        traces: filtered_traces,
        total,
    }))
}

/// GET /api/traces/:id - Get a specific trace with all spans
pub async fn get_trace(
    State(state): State<AppState>,
    Path(trace_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    info!("GET /api/traces/{}", trace_id);

    let trace = state.storage.get_trace_by_id(&trace_id)?;
    Ok(Json(trace))
}

/// GET /api/logs - List logs
pub async fn list_logs(
    State(state): State<AppState>,
    Query(params): Query<ListLogsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    info!("GET /api/logs - params: {:?}", params);

    if let Some(trace_id) = params.trace_id.as_deref() {
        let logs = state.storage.get_logs_by_trace_id(trace_id)?;
        return Ok(Json(logs));
    }

    let limit = params.limit.unwrap_or(100).min(1000);
    let logs = state
        .storage
        .list_logs(params.service.as_deref(), Some(limit))?;

    Ok(Json(logs))
}

/// GET /api/services - List unique service names
pub async fn list_services(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    info!("GET /api/services");

    let traces = state.storage.list_traces(None, Some(1000))?;
    let mut services: Vec<String> = traces.into_iter().filter_map(|t| t.service_name).collect();

    services.sort();
    services.dedup();

    Ok(Json(serde_json::json!({
        "services": services
    })))
}

/// GET /api/metrics - List metrics
pub async fn list_metrics(
    State(state): State<AppState>,
    Query(params): Query<ListMetricsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    info!("GET /api/metrics - params: {:?}", params);

    let limit = params.limit.unwrap_or(100).min(1000);
    let metrics = state
        .storage
        .list_metrics(params.service.as_deref(), Some(limit))?;

    Ok(Json(serde_json::json!({
        "metrics": metrics
    })))
}

/// Number of buckets returned by the stats activity timeline.
const ACTIVITY_BUCKETS: usize = 30;

/// GET /api/stats - Aggregate statistics across all stored telemetry
pub async fn get_stats(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    info!("GET /api/stats");

    let storage = &state.storage;
    let response = StatsResponse {
        spans: storage.count_spans()?,
        logs: storage.count_logs()?,
        metrics: storage.count_metrics()?,
        traces: storage.trace_stats()?.into(),
        services: storage
            .service_stats()?
            .into_iter()
            .map(Into::into)
            .collect(),
        activity: storage
            .trace_time_buckets(ACTIVITY_BUCKETS)?
            .into_iter()
            .map(Into::into)
            .collect(),
    };

    Ok(Json(response))
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

    let project_path = project_root.to_string_lossy().into_owned();

    Json(serde_json::json!({
        "name": project_name,
        "path": project_path,
    }))
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use faze::Storage;
    use faze::models::{Attributes, Span, SpanKind, Status};
    use std::sync::Arc;

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
    async fn test_get_trace_exposes_fidelity_fields() {
        use faze::models::{SpanEvent, SpanLink};

        let storage = Storage::new_in_memory().unwrap();
        let mut resource_attrs = Attributes::new();
        resource_attrs.insert("service.version", "1.0");
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
        )
        .with_events(vec![SpanEvent {
            time_unix_nano: 1_500_000_000,
            name: "exception".to_string(),
            attributes: Attributes::new(),
            dropped_attributes_count: 0,
        }])
        .with_links(vec![SpanLink {
            trace_id: "other".to_string(),
            span_id: "other-span".to_string(),
            trace_state: None,
            attributes: Attributes::new(),
            dropped_attributes_count: 0,
        }])
        .with_resource_attributes(resource_attrs);
        storage.insert_span(&span).unwrap();

        let state = AppState {
            storage: Arc::new(storage),
        };

        let response = get_trace(State(state), Path("trace1".to_string()))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let span_json = &json["spans"][0];
        assert_eq!(span_json["events"][0]["name"], "exception");
        assert_eq!(span_json["links"][0]["trace_id"], "other");
        assert_eq!(span_json["resource_attributes"]["service.version"], "1.0");
    }

    #[tokio::test]
    async fn test_list_traces_with_filters() {
        let storage = Storage::new_in_memory().unwrap();
        for i in 0..5 {
            let span = Span::new(
                format!("span{i}"),
                format!("trace{i}"),
                None,
                format!("operation{i}"),
                SpanKind::Server,
                1_000_000_000,
                1_000_000_000 + (i64::from(i) * 100_000_000), // Different durations
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
                format!("span{i}"),
                format!("trace{i}"),
                None,
                format!("operation{i}"),
                SpanKind::Server,
                1_000_000_000 + (i64::from(i) * 1000),
                2_000_000_000 + (i64::from(i) * 1000),
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
                format!("span{i}"),
                format!("trace{i}"),
                None,
                format!("operation{i}"),
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
    async fn test_get_stats_empty() {
        let storage = Storage::new_in_memory().unwrap();
        let state = AppState {
            storage: Arc::new(storage),
        };

        let response = get_stats(State(state)).await.into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_stats_with_data() {
        let storage = Storage::new_in_memory().unwrap();
        let ok_span = Span::new(
            "span1".to_string(),
            "trace1".to_string(),
            None,
            "ok operation".to_string(),
            SpanKind::Server,
            1_000_000_000,
            2_000_000_000,
            Attributes::new(),
            Status::ok(),
            Some("service-a".to_string()),
        );
        let error_span = Span::new(
            "span2".to_string(),
            "trace2".to_string(),
            None,
            "failing operation".to_string(),
            SpanKind::Server,
            3_000_000_000,
            4_000_000_000,
            Attributes::new(),
            Status::error("boom"),
            Some("service-b".to_string()),
        );
        storage.insert_spans(&[ok_span, error_span]).unwrap();

        let state = AppState {
            storage: Arc::new(storage),
        };

        let response = get_stats(State(state)).await.into_response();

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
            trace_id: None,
            limit: None,
        };

        let response = list_logs(State(state), Query(query)).await.into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_logs_by_trace_id() {
        use faze::models::{Log, SeverityLevel};

        let storage = Storage::new_in_memory().unwrap();
        let log = Log::new(
            1_000_000_000,
            SeverityLevel::Info,
            None,
            "correlated log".to_string(),
            Attributes::new(),
            Some("trace1".to_string()),
            Some("span1".to_string()),
            Some("test-service".to_string()),
        );
        storage.insert_log(&log).unwrap();

        let state = AppState {
            storage: Arc::new(storage),
        };

        let query = ListLogsQuery {
            service: None,
            trace_id: Some("trace1".to_string()),
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
