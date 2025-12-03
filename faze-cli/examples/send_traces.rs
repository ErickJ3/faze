use faze_collector::proto::opentelemetry::proto::collector::trace::v1::ExportTraceServiceRequest;
/// Example client that sends OTLP traces to Faze
///
/// Usage:
///   1. Start Faze server: cargo run -p faze-cli -- serve --db-path faze.db
///   2. Run this example: cargo run --example send_traces
///   3. Query traces in another terminal (after stopping the server):
///      cargo run -p faze-cli -- traces --db-path faze.db
use faze_collector::proto::opentelemetry::proto::collector::trace::v1::trace_service_client::TraceServiceClient;
use faze_collector::proto::opentelemetry::proto::common::v1::{AnyValue, KeyValue, any_value};
use faze_collector::proto::opentelemetry::proto::resource::v1::Resource;
use faze_collector::proto::opentelemetry::proto::trace::v1::{
    ResourceSpans, ScopeSpans, Span as OtlpSpan, SpanKind as OtlpSpanKind, Status as OtlpStatus,
    StatusCode as OtlpStatusCode,
};
use std::time::{SystemTime, UNIX_EPOCH};

fn current_time_nanos() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

fn create_span(
    trace_id: &[u8],
    span_id: &[u8],
    parent_span_id: Option<&[u8]>,
    name: &str,
    start_offset_ms: u64,
    duration_ms: u64,
    status_code: OtlpStatusCode,
) -> OtlpSpan {
    let now = current_time_nanos();
    let start = now - (start_offset_ms * 1_000_000);
    let end = start + (duration_ms * 1_000_000);

    OtlpSpan {
        trace_id: trace_id.to_vec(),
        span_id: span_id.to_vec(),
        trace_state: String::new(),
        parent_span_id: parent_span_id.map(|p| p.to_vec()).unwrap_or_default(),
        name: name.to_string(),
        kind: OtlpSpanKind::Server as i32,
        start_time_unix_nano: start,
        end_time_unix_nano: end,
        attributes: vec![
            KeyValue {
                key: "http.method".to_string(),
                value: Some(AnyValue {
                    value: Some(any_value::Value::StringValue("GET".to_string())),
                }),
            },
            KeyValue {
                key: "http.url".to_string(),
                value: Some(AnyValue {
                    value: Some(any_value::Value::StringValue(
                        "http://localhost/api/users".to_string(),
                    )),
                }),
            },
            KeyValue {
                key: "http.status_code".to_string(),
                value: Some(AnyValue {
                    value: Some(any_value::Value::IntValue(
                        if status_code == OtlpStatusCode::Ok {
                            200
                        } else {
                            500
                        },
                    )),
                }),
            },
        ],
        dropped_attributes_count: 0,
        events: vec![],
        dropped_events_count: 0,
        links: vec![],
        dropped_links_count: 0,
        status: Some(OtlpStatus {
            message: if status_code == OtlpStatusCode::Ok {
                String::new()
            } else {
                "Internal server error".to_string()
            },
            code: status_code as i32,
        }),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("connecting to faze otlp collector at localhost:4317...");

    let mut client = TraceServiceClient::connect("http://localhost:4317").await?;

    println!("connected! sending traces...\n");
    let trace_id_1 = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

    let request1 = ExportTraceServiceRequest {
        resource_spans: vec![ResourceSpans {
            resource: Some(Resource {
                attributes: vec![
                    KeyValue {
                        key: "service.name".to_string(),
                        value: Some(AnyValue {
                            value: Some(any_value::Value::StringValue(
                                "example-web-api".to_string(),
                            )),
                        }),
                    },
                    KeyValue {
                        key: "service.version".to_string(),
                        value: Some(AnyValue {
                            value: Some(any_value::Value::StringValue("1.0.0".to_string())),
                        }),
                    },
                ],
                dropped_attributes_count: 0,
            }),
            scope_spans: vec![ScopeSpans {
                scope: None,
                spans: vec![
                    create_span(
                        &trace_id_1,
                        &[1, 2, 3, 4, 5, 6, 7, 8],
                        None,
                        "GET /api/users",
                        500,
                        250,
                        OtlpStatusCode::Ok,
                    ),
                    create_span(
                        &trace_id_1,
                        &[2, 2, 3, 4, 5, 6, 7, 8],
                        Some(&[1, 2, 3, 4, 5, 6, 7, 8]),
                        "db.query",
                        450,
                        150,
                        OtlpStatusCode::Ok,
                    ),
                ],
                schema_url: String::new(),
            }],
            schema_url: String::new(),
        }],
    };

    let response1 = client.export(request1).await?;
    println!("trace 1 sent: GET /api/users (250ms, 2 spans)");
    println!("status: {:?}\n", response1.into_inner());

    let trace_id_2 = [2u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

    let request2 = ExportTraceServiceRequest {
        resource_spans: vec![ResourceSpans {
            resource: Some(Resource {
                attributes: vec![KeyValue {
                    key: "service.name".to_string(),
                    value: Some(AnyValue {
                        value: Some(any_value::Value::StringValue("example-web-api".to_string())),
                    }),
                }],
                dropped_attributes_count: 0,
            }),
            scope_spans: vec![ScopeSpans {
                scope: None,
                spans: vec![create_span(
                    &trace_id_2,
                    &[3, 2, 3, 4, 5, 6, 7, 8],
                    None,
                    "GET /api/error",
                    200,
                    75,
                    OtlpStatusCode::Error,
                )],
                schema_url: String::new(),
            }],
            schema_url: String::new(),
        }],
    };

    let response2 = client.export(request2).await?;
    println!("trace 2 sent: GET /api/error (75ms, ERROR)");
    println!("status: {:?}\n", response2.into_inner());

    let trace_id_3 = [3u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

    let request3 = ExportTraceServiceRequest {
        resource_spans: vec![ResourceSpans {
            resource: Some(Resource {
                attributes: vec![KeyValue {
                    key: "service.name".to_string(),
                    value: Some(AnyValue {
                        value: Some(any_value::Value::StringValue(
                            "example-background-worker".to_string(),
                        )),
                    }),
                }],
                dropped_attributes_count: 0,
            }),
            scope_spans: vec![ScopeSpans {
                scope: None,
                spans: vec![create_span(
                    &trace_id_3,
                    &[4, 2, 3, 4, 5, 6, 7, 8],
                    None,
                    "process_job",
                    100,
                    1200,
                    OtlpStatusCode::Ok,
                )],
                schema_url: String::new(),
            }],
            schema_url: String::new(),
        }],
    };

    let response3 = client.export(request3).await?;
    println!("trace 3 sent: process_job (1200ms, SLOW)");
    println!("status: {:?}\n", response3.into_inner());

    println!("all traces sent successfully!");
    println!("\n next steps:");
    println!("1 - stop the faze server (Ctrl+C)");
    println!("2 - query traces: cargo run -p faze-cli -- traces --db-path faze.db");
    println!("3 - query slow traces: cargo run -p faze-cli -- traces --slow --db-path faze.db");

    Ok(())
}
