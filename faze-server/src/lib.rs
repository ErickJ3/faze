//! HTTP API and embedded web UI for browsing collected observability data.

pub mod routes;
pub mod server;
pub mod ui;

pub use server::ApiServer;
