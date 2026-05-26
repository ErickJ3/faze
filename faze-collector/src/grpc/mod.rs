//! gRPC service implementations for the OTLP collector endpoints.

/// Log-record gRPC service.
pub mod logs;
/// Metric gRPC service.
pub mod metrics;
/// Trace gRPC service.
pub mod traces;
