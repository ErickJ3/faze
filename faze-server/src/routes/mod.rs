//! HTTP route handlers exposing trace, log, and metric APIs.

mod dto;
mod error;
mod handlers;

pub use handlers::{
    get_project_info, get_stats, get_trace, health_check, list_logs, list_metrics, list_services,
    list_traces,
};

use faze::Storage;
use std::sync::Arc;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    /// Shared storage handle used by all route handlers.
    pub storage: Arc<Storage>,
}
