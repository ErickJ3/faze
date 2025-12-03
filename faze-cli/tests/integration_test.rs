use faze::Storage;
use faze_collector::proto::opentelemetry::proto::collector::trace::v1::ExportTraceServiceRequest;
use faze_collector::proto::opentelemetry::proto::collector::trace::v1::trace_service_client::TraceServiceClient;
use faze_collector::proto::opentelemetry::proto::common::v1::{AnyValue, KeyValue, any_value};
use faze_collector::proto::opentelemetry::proto::resource::v1::Resource;
use faze_collector::proto::opentelemetry::proto::trace::v1::{
    ResourceSpans, ScopeSpans, Span as OtlpSpan, SpanKind as OtlpSpanKind, Status as OtlpStatus,
    StatusCode as OtlpStatusCode,
};
use std::time::Duration;
use tokio::time::sleep;

/// Helper to create a test OTLP span
fn create_test_otlp_span(
    trace_id: &[u8],
    span_id: &[u8],
    name: &str,
    start_time_nanos: u64,
    duration_nanos: u64,
) -> OtlpSpan {
    OtlpSpan {
        trace_id: trace_id.to_vec(),
        span_id: span_id.to_vec(),
        trace_state: String::new(),
        parent_span_id: vec![],
        name: name.to_string(),
        kind: OtlpSpanKind::Server as i32,
        start_time_unix_nano: start_time_nanos,
        end_time_unix_nano: start_time_nanos + duration_nanos,
        attributes: vec![
            KeyValue {
                key: "http.method".to_string(),
                value: Some(AnyValue {
                    value: Some(any_value::Value::StringValue("GET".to_string())),
                }),
            },
            KeyValue {
                key: "http.status_code".to_string(),
                value: Some(AnyValue {
                    value: Some(any_value::Value::IntValue(200)),
                }),
            },
        ],
        dropped_attributes_count: 0,
        events: vec![],
        dropped_events_count: 0,
        links: vec![],
        dropped_links_count: 0,
        status: Some(OtlpStatus {
            message: String::new(),
            code: OtlpStatusCode::Ok as i32,
        }),
    }
}

/// Test that we can send a trace via gRPC and it gets stored in DuckDB
#[tokio::test]
#[ignore] // Run with: cargo test --test integration_test -- --ignored
async fn test_send_trace_via_grpc() {
    // Note: This test requires a running faze server
    // Start server first: cargo run -p faze-cli -- serve --db-path /tmp/test-integration.db

    // Wait a bit for server to be ready
    sleep(Duration::from_millis(500)).await;

    // Connect to the gRPC server
    let mut client = TraceServiceClient::connect("http://localhost:4317")
        .await
        .expect("Failed to connect to gRPC server");

    // Create a test trace
    let trace_id = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    let span_id = [1u8, 2, 3, 4, 5, 6, 7, 8];

    let request = ExportTraceServiceRequest {
        resource_spans: vec![ResourceSpans {
            resource: Some(Resource {
                attributes: vec![KeyValue {
                    key: "service.name".to_string(),
                    value: Some(AnyValue {
                        value: Some(any_value::Value::StringValue(
                            "integration-test-service".to_string(),
                        )),
                    }),
                }],
                dropped_attributes_count: 0,
            }),
            scope_spans: vec![ScopeSpans {
                scope: None,
                spans: vec![create_test_otlp_span(
                    &trace_id,
                    &span_id,
                    "GET /api/test",
                    1_700_000_000_000_000_000,
                    150_000_000, // 150ms
                )],
                schema_url: String::new(),
            }],
            schema_url: String::new(),
        }],
    };

    // Send the trace
    let response = client.export(request).await.expect("Failed to send trace");

    println!("Response: {:?}", response);

    // Give it time to persist
    sleep(Duration::from_millis(100)).await;

    // Note: We can't query the database here because of DuckDB locking
    // In a real test, we would either:
    // 1. Use the REST API to query (once implemented)
    // 2. Stop the server and then query
    // 3. Use a separate database file for each test

    assert!(response.into_inner().partial_success.is_none());
}

/// Test storage operations directly (unit test)
#[tokio::test]
async fn test_storage_insert_and_query() {
    use faze::models::{Attributes, Span, SpanKind, Status};

    let storage = Storage::new_in_memory().expect("Failed to create storage");

    // Insert a test span
    let mut attrs = Attributes::new();
    attrs.insert("http.method", "POST");
    attrs.insert("http.status_code", 201i64);

    let span = Span::new(
        "abc123".to_string(),
        "trace456".to_string(),
        None,
        "POST /api/users".to_string(),
        SpanKind::Server,
        1_700_000_000_000_000_000,
        1_700_000_000_150_000_000,
        attrs,
        Status::ok(),
        Some("test-service".to_string()),
    );

    storage.insert_span(&span).expect("Failed to insert span");

    // Query it back
    let trace = storage
        .get_trace_by_id("trace456")
        .expect("Failed to get trace");

    assert_eq!(trace.trace_id, "trace456");
    assert_eq!(trace.spans.len(), 1);
    assert_eq!(trace.spans[0].span_id, "abc123");
    assert_eq!(trace.spans[0].name, "POST /api/users");
    assert_eq!(trace.spans[0].duration_ms(), 150.0);

    // Test list_traces
    let traces = storage
        .list_traces(None, Some(10))
        .expect("Failed to list traces");
    assert_eq!(traces.len(), 1);
}

/// Test inserting multiple spans in the same trace
#[tokio::test]
async fn test_storage_multiple_spans_same_trace() {
    use faze::models::{Attributes, Span, SpanKind, Status};

    let storage = Storage::new_in_memory().expect("Failed to create storage");

    let trace_id = "trace-multi-span";

    // Insert parent span
    let parent_span = Span::new(
        "span-parent".to_string(),
        trace_id.to_string(),
        None,
        "parent operation".to_string(),
        SpanKind::Server,
        1_000_000_000,
        5_000_000_000,
        Attributes::new(),
        Status::ok(),
        Some("test-service".to_string()),
    );

    // Insert child spans
    let child1 = Span::new(
        "span-child1".to_string(),
        trace_id.to_string(),
        Some("span-parent".to_string()),
        "child operation 1".to_string(),
        SpanKind::Internal,
        2_000_000_000,
        3_000_000_000,
        Attributes::new(),
        Status::ok(),
        Some("test-service".to_string()),
    );

    let child2 = Span::new(
        "span-child2".to_string(),
        trace_id.to_string(),
        Some("span-parent".to_string()),
        "child operation 2".to_string(),
        SpanKind::Internal,
        3_500_000_000,
        4_500_000_000,
        Attributes::new(),
        Status::ok(),
        Some("test-service".to_string()),
    );

    storage.insert_span(&parent_span).unwrap();
    storage.insert_span(&child1).unwrap();
    storage.insert_span(&child2).unwrap();

    // Query the trace
    let trace = storage.get_trace_by_id(trace_id).unwrap();

    assert_eq!(trace.span_count(), 3);
    assert_eq!(trace.root_span().unwrap().span_id, "span-parent");

    let children = trace.children_of("span-parent");
    assert_eq!(children.len(), 2);
}

/// Test querying logs
#[tokio::test]
async fn test_storage_logs() {
    use faze::models::{Attributes, Log, SeverityLevel};

    let storage = Storage::new_in_memory().expect("Failed to create storage");

    // Insert logs
    let log1 = Log::new(
        1_000_000_000,
        SeverityLevel::Info,
        Some("INFO".to_string()),
        "User logged in".to_string(),
        Attributes::new(),
        Some("trace1".to_string()),
        Some("span1".to_string()),
        Some("auth-service".to_string()),
    );

    let log2 = Log::new(
        2_000_000_000,
        SeverityLevel::Error,
        Some("ERROR".to_string()),
        "Database connection failed".to_string(),
        Attributes::new(),
        Some("trace2".to_string()),
        Some("span2".to_string()),
        Some("db-service".to_string()),
    );

    storage.insert_log(&log1).unwrap();
    storage.insert_log(&log2).unwrap();

    // Query all logs
    let logs = storage.list_logs(None, Some(10)).unwrap();
    assert_eq!(logs.len(), 2);

    // Query by service
    let auth_logs = storage.list_logs(Some("auth-service"), Some(10)).unwrap();
    assert_eq!(auth_logs.len(), 1);
    assert_eq!(auth_logs[0].body, "User logged in");

    // Count
    let count = storage.count_logs().unwrap();
    assert_eq!(count, 2);
}
