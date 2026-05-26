//! HTTP API and embedded web UI for browsing collected observability data.

/// HTTP route handlers exposing trace, log, and metric APIs.
pub mod routes;
/// Axum-based API server lifecycle.
pub mod server;
/// Embedded static asset handler for the web UI.
pub mod ui;

pub use server::ApiServer;
