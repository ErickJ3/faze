//! OTLP/HTTP collector: `/v1/traces`, `/v1/logs`, and `/v1/metrics`.
//!
//! Supports `application/x-protobuf` and `application/json` (OTLP/JSON)
//! request bodies, optionally gzip-compressed, and returns spec-conformant
//! `ExportXServiceResponse` bodies and `google.rpc.Status` errors in the
//! request's encoding.

use crate::convert::logs::convert_resource_logs;
use crate::convert::metrics::convert_resource_metrics;
use crate::convert::traces::convert_resource_spans;
use crate::proto::google::rpc::Status as RpcStatus;
use crate::proto::opentelemetry::proto::collector::{
    logs::v1::{ExportLogsServiceRequest, ExportLogsServiceResponse},
    metrics::v1::{ExportMetricsServiceRequest, ExportMetricsServiceResponse},
    trace::v1::{ExportTraceServiceRequest, ExportTraceServiceResponse},
};
use axum::{
    Router,
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::Response,
    routing::post,
};
use faze::Storage;
use prost::Message;
use serde::{Serialize, de::DeserializeOwned};
use std::sync::Arc;
use tracing::error;

/// Body content negotiation and decompression.
mod body;
/// OTLP/JSON quirk handling (hex ID normalization).
mod json;

use body::{DecompressError, Encoding, decompress, negotiate};

/// gRPC status code for malformed requests.
const CODE_INVALID_ARGUMENT: i32 = 3;
/// gRPC status code for server-side failures.
const CODE_INTERNAL: i32 = 13;

fn build_response(status: StatusCode, encoding: Encoding, payload: Vec<u8>) -> Response {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, encoding.content_type())
        .body(payload.into())
        .unwrap_or_default()
}

/// Serialize a `google.rpc.Status` error in the request's encoding.
fn status_response(
    encoding: Encoding,
    http_status: StatusCode,
    code: i32,
    message: String,
) -> Response {
    let status = RpcStatus { code, message };
    let payload = match encoding {
        Encoding::Protobuf => status.encode_to_vec(),
        Encoding::Json => serde_json::to_vec(&status).unwrap_or_default(),
    };
    build_response(http_status, encoding, payload)
}

/// Serialize an empty (full success) export response.
fn success_response<Res: Message + Default + Serialize>(encoding: Encoding) -> Response {
    let response = Res::default();
    let payload = match encoding {
        Encoding::Protobuf => response.encode_to_vec(),
        Encoding::Json => serde_json::to_vec(&response).unwrap_or_default(),
    };
    build_response(StatusCode::OK, encoding, payload)
}

fn decode_request<Req: Message + Default + DeserializeOwned>(
    encoding: Encoding,
    payload: &[u8],
) -> Result<Req, String> {
    match encoding {
        Encoding::Protobuf => {
            Req::decode(payload).map_err(|e| format!("invalid protobuf payload: {e}"))
        }
        Encoding::Json => {
            let mut value: serde_json::Value = serde_json::from_slice(payload)
                .map_err(|e| format!("invalid JSON payload: {e}"))?;
            json::normalize_otlp_json(&mut value);
            serde_json::from_value(value).map_err(|e| format!("invalid OTLP/JSON payload: {e}"))
        }
    }
}

/// Shared OTLP/HTTP export flow: negotiate, decompress, decode, store.
fn export<Req, Res>(
    storage: &Storage,
    headers: &HeaderMap,
    body: Bytes,
    store: impl FnOnce(&Storage, Req) -> Result<(), faze::StorageError>,
) -> Response
where
    Req: Message + Default + DeserializeOwned,
    Res: Message + Default + Serialize,
{
    let Some(encoding) = negotiate(headers) else {
        return status_response(
            Encoding::Json,
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            CODE_INVALID_ARGUMENT,
            "unsupported content-type; use application/x-protobuf or application/json".to_string(),
        );
    };

    let payload = match decompress(headers, body) {
        Ok(payload) => payload,
        Err(DecompressError::UnsupportedEncoding(enc)) => {
            return status_response(
                encoding,
                StatusCode::BAD_REQUEST,
                CODE_INVALID_ARGUMENT,
                format!("unsupported content-encoding: {enc}"),
            );
        }
        Err(DecompressError::Corrupt) => {
            return status_response(
                encoding,
                StatusCode::BAD_REQUEST,
                CODE_INVALID_ARGUMENT,
                "failed to decompress request body".to_string(),
            );
        }
        Err(DecompressError::TooLarge) => {
            return status_response(
                encoding,
                StatusCode::PAYLOAD_TOO_LARGE,
                CODE_INVALID_ARGUMENT,
                "decompressed request body too large".to_string(),
            );
        }
    };

    let request = match decode_request::<Req>(encoding, &payload) {
        Ok(request) => request,
        Err(message) => {
            error!("Failed to decode OTLP/HTTP request: {message}");
            return status_response(
                encoding,
                StatusCode::BAD_REQUEST,
                CODE_INVALID_ARGUMENT,
                message,
            );
        }
    };

    match store(storage, request) {
        Ok(()) => success_response::<Res>(encoding),
        Err(e) => {
            error!("Failed to store OTLP/HTTP batch: {e}");
            status_response(
                encoding,
                StatusCode::INTERNAL_SERVER_ERROR,
                CODE_INTERNAL,
                e.to_string(),
            )
        }
    }
}

/// HTTP handler for OTLP trace export
async fn export_traces(
    State(storage): State<Arc<Storage>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    // Each batch is inserted as one transaction; a failure rejects it whole.
    export::<ExportTraceServiceRequest, ExportTraceServiceResponse>(
        &storage,
        &headers,
        body,
        |s, req| s.insert_spans(&convert_resource_spans(&req.resource_spans)),
    )
}

/// HTTP handler for OTLP logs export
async fn export_logs(
    State(storage): State<Arc<Storage>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    export::<ExportLogsServiceRequest, ExportLogsServiceResponse>(
        &storage,
        &headers,
        body,
        |s, req| s.insert_logs(&convert_resource_logs(&req.resource_logs)),
    )
}

/// HTTP handler for OTLP metrics export
async fn export_metrics(
    State(storage): State<Arc<Storage>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    export::<ExportMetricsServiceRequest, ExportMetricsServiceResponse>(
        &storage,
        &headers,
        body,
        |s, req| s.insert_metrics(&convert_resource_metrics(req.resource_metrics)),
    )
}

/// Create HTTP router for OTLP collector
pub fn create_router(storage: Arc<Storage>) -> Router {
    Router::new()
        .route("/v1/traces", post(export_traces))
        .route("/v1/logs", post(export_logs))
        .route("/v1/metrics", post(export_metrics))
        .with_state(storage)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::opentelemetry::proto::{
        common::v1::{AnyValue, KeyValue, any_value},
        logs::v1::{LogRecord, ResourceLogs, ScopeLogs},
        metrics::v1::{
            Gauge, Metric as OtlpMetric, NumberDataPoint, ResourceMetrics, ScopeMetrics, metric,
            number_data_point,
        },
        resource::v1::Resource,
        trace::v1::{
            ResourceSpans, ScopeSpans, Span as OtlpSpan, SpanKind as OtlpSpanKind,
            Status as OtlpStatus, StatusCode as OtlpStatusCode,
        },
    };
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use std::io::Write;
    use tower::ServiceExt;

    fn create_test_request() -> ExportTraceServiceRequest {
        ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".to_string(),
                        value: Some(AnyValue {
                            value: Some(any_value::Value::StringValue("test-service".to_string())),
                        }),
                    }],
                    dropped_attributes_count: 0,
                }),
                scope_spans: vec![ScopeSpans {
                    scope: None,
                    spans: vec![OtlpSpan {
                        trace_id: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                        span_id: vec![1, 2, 3, 4, 5, 6, 7, 8],
                        name: "test-span".to_string(),
                        kind: OtlpSpanKind::Server as i32,
                        start_time_unix_nano: 1_000_000_000_000_000_000,
                        end_time_unix_nano: 1_000_000_000_100_000_000,
                        status: Some(OtlpStatus {
                            message: String::new(),
                            code: OtlpStatusCode::Ok as i32,
                        }),
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        }
    }

    fn protobuf_request(uri: &str, payload: Vec<u8>) -> Request<Body> {
        Request::builder()
            .uri(uri)
            .method("POST")
            .header("content-type", "application/x-protobuf")
            .body(Body::from(payload))
            .unwrap()
    }

    async fn response_bytes(response: Response) -> Bytes {
        axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_export_traces_http() {
        let storage = Arc::new(Storage::new_in_memory().unwrap());
        let app = create_router(storage.clone());
        let request_data = create_test_request();
        let mut buf = Vec::new();
        request_data.encode(&mut buf).unwrap();

        let response = app
            .oneshot(protobuf_request("/v1/traces", buf))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        // The success body is a serialized ExportTraceServiceResponse.
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/x-protobuf"
        );
        let payload = response_bytes(response).await;
        let decoded = ExportTraceServiceResponse::decode(&payload[..]).unwrap();
        assert_eq!(decoded, ExportTraceServiceResponse::default());

        assert_eq!(storage.count_spans().unwrap(), 1);
    }

    #[tokio::test]
    async fn test_export_traces_invalid_protobuf() {
        let storage = Arc::new(Storage::new_in_memory().unwrap());
        let app = create_router(storage.clone());
        let invalid_data = b"not a valid protobuf message";

        let response = app
            .oneshot(protobuf_request("/v1/traces", invalid_data.to_vec()))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        // Error body is a google.rpc.Status in the request encoding.
        let payload = response_bytes(response).await;
        let status = RpcStatus::decode(&payload[..]).unwrap();
        assert_eq!(status.code, CODE_INVALID_ARGUMENT);
    }

    #[tokio::test]
    async fn test_export_traces_empty_body() {
        let storage = Arc::new(Storage::new_in_memory().unwrap());
        let app = create_router(storage.clone());
        let empty_request = ExportTraceServiceRequest {
            resource_spans: vec![],
        };
        let mut buf = Vec::new();
        empty_request.encode(&mut buf).unwrap();

        let response = app
            .oneshot(protobuf_request("/v1/traces", buf))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(storage.count_spans().unwrap(), 0);
    }

    #[tokio::test]
    async fn test_export_traces_multiple_spans() {
        let storage = Arc::new(Storage::new_in_memory().unwrap());
        let app = create_router(storage.clone());

        let mut request_data = create_test_request();
        request_data.resource_spans[0].scope_spans[0]
            .spans
            .push(OtlpSpan {
                trace_id: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                span_id: vec![9, 10, 11, 12, 13, 14, 15, 16],
                name: "span2".to_string(),
                kind: OtlpSpanKind::Client as i32,
                start_time_unix_nano: 1_000_000_000_000_000_000,
                end_time_unix_nano: 1_000_000_000_100_000_000,
                ..Default::default()
            });
        let mut buf = Vec::new();
        request_data.encode(&mut buf).unwrap();

        let response = app
            .oneshot(protobuf_request("/v1/traces", buf))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(storage.count_spans().unwrap(), 2);
    }

    #[tokio::test]
    async fn test_export_traces_with_attributes() {
        let storage = Arc::new(Storage::new_in_memory().unwrap());
        let app = create_router(storage.clone());

        let mut request_data = create_test_request();
        request_data.resource_spans[0].scope_spans[0].spans[0].attributes = vec![
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
        ];
        let mut buf = Vec::new();
        request_data.encode(&mut buf).unwrap();

        let response = app
            .oneshot(protobuf_request("/v1/traces", buf))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let trace = storage
            .get_trace_by_id("0102030405060708090a0b0c0d0e0f10")
            .unwrap();
        let span = &trace.spans[0];
        assert_eq!(span.attributes.get_string("http.method"), Some("GET"));
        assert_eq!(span.attributes.get_int("http.status_code"), Some(200));
    }

    #[tokio::test]
    async fn test_export_traces_http_concurrent() {
        let storage = Arc::new(Storage::new_in_memory().unwrap());

        let mut handles = vec![];
        for i in 0..10u8 {
            let storage_clone = storage.clone();
            let handle = tokio::spawn(async move {
                let app = create_router(storage_clone);
                let mut request_data = create_test_request();
                let span = &mut request_data.resource_spans[0].scope_spans[0].spans[0];
                span.trace_id[0] = i;
                span.span_id[0] = i;
                span.name = format!("span-{i}");
                let mut buf = Vec::new();
                request_data.encode(&mut buf).unwrap();

                app.oneshot(protobuf_request("/v1/traces", buf))
                    .await
                    .unwrap()
            });
            handles.push(handle);
        }

        for handle in handles {
            let resp = handle.await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
        }
        assert_eq!(storage.count_spans().unwrap(), 10);
    }

    #[tokio::test]
    async fn test_export_traces_large_payload() {
        let storage = Arc::new(Storage::new_in_memory().unwrap());
        let app = create_router(storage.clone());
        let mut spans = vec![];
        for i in 0..100u8 {
            spans.push(OtlpSpan {
                trace_id: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                span_id: vec![i, i, i, i, 5, 6, 7, 8],
                name: format!("large-span-{i}"),
                kind: OtlpSpanKind::Internal as i32,
                start_time_unix_nano: 1_000_000_000_000_000_000 + u64::from(i),
                end_time_unix_nano: 1_000_000_000_100_000_000 + u64::from(i),
                ..Default::default()
            });
        }

        let request_data = ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: None,
                scope_spans: vec![ScopeSpans {
                    scope: None,
                    spans,
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };
        let mut buf = Vec::new();
        request_data.encode(&mut buf).unwrap();

        let response = app
            .oneshot(protobuf_request("/v1/traces", buf))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(storage.count_spans().unwrap(), 100);
    }

    #[tokio::test]
    async fn test_export_logs_http() {
        let storage = Arc::new(Storage::new_in_memory().unwrap());
        let app = create_router(storage.clone());

        let request_data = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    scope: None,
                    log_records: vec![LogRecord {
                        time_unix_nano: 1_000,
                        severity_number: 9,
                        body: Some(AnyValue {
                            value: Some(any_value::Value::StringValue("hello".to_string())),
                        }),
                        ..Default::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };
        let mut buf = Vec::new();
        request_data.encode(&mut buf).unwrap();

        let response = app
            .oneshot(protobuf_request("/v1/logs", buf))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let payload = response_bytes(response).await;
        ExportLogsServiceResponse::decode(&payload[..]).unwrap();
        assert_eq!(storage.count_logs().unwrap(), 1);
        assert_eq!(storage.list_logs(None, None).unwrap()[0].body, "hello");
    }

    #[tokio::test]
    async fn test_export_metrics_http() {
        let storage = Arc::new(Storage::new_in_memory().unwrap());
        let app = create_router(storage.clone());

        let request_data = ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: None,
                scope_metrics: vec![ScopeMetrics {
                    scope: None,
                    metrics: vec![OtlpMetric {
                        name: "cpu.usage".to_string(),
                        description: String::new(),
                        unit: String::new(),
                        metadata: vec![],
                        data: Some(metric::Data::Gauge(Gauge {
                            data_points: vec![NumberDataPoint {
                                time_unix_nano: 1_000,
                                value: Some(number_data_point::Value::AsDouble(0.5)),
                                ..Default::default()
                            }],
                        })),
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };
        let mut buf = Vec::new();
        request_data.encode(&mut buf).unwrap();

        let response = app
            .oneshot(protobuf_request("/v1/metrics", buf))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let payload = response_bytes(response).await;
        ExportMetricsServiceResponse::decode(&payload[..]).unwrap();
        assert_eq!(storage.count_metrics().unwrap(), 1);
    }

    #[tokio::test]
    async fn test_export_traces_otlp_json() {
        let storage = Arc::new(Storage::new_in_memory().unwrap());
        let app = create_router(storage.clone());

        // Spec-shaped OTLP/JSON: hex IDs, u64 timestamps as strings, enum as
        // integer. Pins pbjson's name-or-int enum decoding.
        let json_body = r#"{
            "resourceSpans": [{
                "resource": {
                    "attributes": [{
                        "key": "service.name",
                        "value": {"stringValue": "json-service"}
                    }]
                },
                "scopeSpans": [{
                    "scope": {"name": "json-lib", "version": "1.0"},
                    "spans": [{
                        "traceId": "0102030405060708090a0b0c0d0e0f10",
                        "spanId": "0102030405060708",
                        "parentSpanId": "",
                        "name": "json-span",
                        "kind": 2,
                        "startTimeUnixNano": "1000000000000000000",
                        "endTimeUnixNano": "1000000000100000000",
                        "status": {"code": "STATUS_CODE_OK"}
                    }]
                }]
            }]
        }"#;

        let request = Request::builder()
            .uri("/v1/traces")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(json_body))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/json"
        );
        let payload = response_bytes(response).await;
        assert_eq!(&payload[..], b"{}");

        let trace = storage
            .get_trace_by_id("0102030405060708090a0b0c0d0e0f10")
            .unwrap();
        let span = &trace.spans[0];
        assert_eq!(span.name, "json-span");
        assert_eq!(span.kind, faze::SpanKind::Server);
        assert_eq!(span.service_name, Some("json-service".to_string()));
        assert_eq!(span.start_time_unix_nano, 1_000_000_000_000_000_000);
        assert_eq!(span.scope.as_ref().unwrap().name, "json-lib");
    }

    #[tokio::test]
    async fn test_export_logs_otlp_json() {
        let storage = Arc::new(Storage::new_in_memory().unwrap());
        let app = create_router(storage.clone());

        let json_body = r#"{
            "resourceLogs": [{
                "scopeLogs": [{
                    "logRecords": [{
                        "timeUnixNano": "1000",
                        "severityNumber": 13,
                        "severityText": "warning",
                        "body": {"stringValue": "json log"},
                        "traceId": "0102030405060708090a0b0c0d0e0f10",
                        "spanId": "0102030405060708"
                    }]
                }]
            }]
        }"#;

        let request = Request::builder()
            .uri("/v1/logs")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(json_body))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let logs = storage.list_logs(None, None).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].body, "json log");
        assert_eq!(logs[0].severity_text, Some("warning".to_string()));
        assert_eq!(
            logs[0].trace_id,
            Some("0102030405060708090a0b0c0d0e0f10".to_string())
        );
    }

    #[tokio::test]
    async fn test_export_traces_gzip() {
        let storage = Arc::new(Storage::new_in_memory().unwrap());
        let app = create_router(storage.clone());
        let request_data = create_test_request();
        let mut buf = Vec::new();
        request_data.encode(&mut buf).unwrap();

        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(&buf).unwrap();
        let compressed = encoder.finish().unwrap();

        let request = Request::builder()
            .uri("/v1/traces")
            .method("POST")
            .header("content-type", "application/x-protobuf")
            .header("content-encoding", "gzip")
            .body(Body::from(compressed))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(storage.count_spans().unwrap(), 1);
    }

    #[tokio::test]
    async fn test_export_traces_unsupported_content_type() {
        let storage = Arc::new(Storage::new_in_memory().unwrap());
        let app = create_router(storage);

        let request = Request::builder()
            .uri("/v1/traces")
            .method("POST")
            .header("content-type", "text/plain")
            .body(Body::from("hi"))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn test_export_traces_invalid_json_returns_status() {
        let storage = Arc::new(Storage::new_in_memory().unwrap());
        let app = create_router(storage);

        let request = Request::builder()
            .uri("/v1/traces")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from("{not json"))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let payload = response_bytes(response).await;
        let status: serde_json::Value = serde_json::from_slice(&payload).unwrap();
        assert_eq!(status["code"], CODE_INVALID_ARGUMENT);
    }
}
