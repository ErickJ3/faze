//! OTLP collector: HTTP and gRPC entry points plus conversion to Faze domain types.

/// OTLP-to-Faze conversion helpers.
pub mod convert;
/// gRPC service implementations.
pub mod grpc;
/// HTTP route handlers.
pub mod http;

/// Generated OTLP protobuf bindings (see `build.rs`).
#[allow(
    missing_docs,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]
pub mod proto {
    pub mod opentelemetry {
        pub mod proto {
            pub mod collector {
                pub mod trace {
                    pub mod v1 {
                        tonic::include_proto!("opentelemetry.proto.collector.trace.v1");
                        include!(concat!(
                            env!("OUT_DIR"),
                            "/opentelemetry.proto.collector.trace.v1.serde.rs"
                        ));
                    }
                }
                pub mod logs {
                    pub mod v1 {
                        tonic::include_proto!("opentelemetry.proto.collector.logs.v1");
                        include!(concat!(
                            env!("OUT_DIR"),
                            "/opentelemetry.proto.collector.logs.v1.serde.rs"
                        ));
                    }
                }
                pub mod metrics {
                    pub mod v1 {
                        tonic::include_proto!("opentelemetry.proto.collector.metrics.v1");
                        include!(concat!(
                            env!("OUT_DIR"),
                            "/opentelemetry.proto.collector.metrics.v1.serde.rs"
                        ));
                    }
                }
            }
            pub mod trace {
                pub mod v1 {
                    tonic::include_proto!("opentelemetry.proto.trace.v1");
                    include!(concat!(
                        env!("OUT_DIR"),
                        "/opentelemetry.proto.trace.v1.serde.rs"
                    ));
                }
            }
            pub mod logs {
                pub mod v1 {
                    tonic::include_proto!("opentelemetry.proto.logs.v1");
                    include!(concat!(
                        env!("OUT_DIR"),
                        "/opentelemetry.proto.logs.v1.serde.rs"
                    ));
                }
            }
            pub mod metrics {
                pub mod v1 {
                    tonic::include_proto!("opentelemetry.proto.metrics.v1");
                    include!(concat!(
                        env!("OUT_DIR"),
                        "/opentelemetry.proto.metrics.v1.serde.rs"
                    ));
                }
            }
            pub mod resource {
                pub mod v1 {
                    tonic::include_proto!("opentelemetry.proto.resource.v1");
                    include!(concat!(
                        env!("OUT_DIR"),
                        "/opentelemetry.proto.resource.v1.serde.rs"
                    ));
                }
            }
            pub mod common {
                pub mod v1 {
                    tonic::include_proto!("opentelemetry.proto.common.v1");
                    include!(concat!(
                        env!("OUT_DIR"),
                        "/opentelemetry.proto.common.v1.serde.rs"
                    ));
                }
            }
        }
    }
    pub mod google {
        pub mod rpc {
            tonic::include_proto!("google.rpc");
            include!(concat!(env!("OUT_DIR"), "/google.rpc.serde.rs"));
        }
    }
}

pub use grpc::{logs, metrics, traces};
pub use http::create_router;
