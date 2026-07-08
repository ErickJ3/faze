//! HTTP API and embedded web UI for browsing collected observability data.

mod routes;
mod server;
mod ui;

pub use server::ApiServer;
