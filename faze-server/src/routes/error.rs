//! Shared error type for API handlers.

use axum::{Json, http::StatusCode, response::IntoResponse};
use tracing::error;

/// Error type shared by all API handlers; renders as `{"error": ...}` JSON.
#[derive(Debug)]
pub enum ApiError {
    /// Requested entity does not exist.
    NotFound(String),
    /// Unexpected internal failure.
    Internal(String),
}

impl From<faze::StorageError> for ApiError {
    fn from(e: faze::StorageError) -> Self {
        match e {
            faze::StorageError::NotFound(msg) => Self::NotFound(msg),
            other => {
                error!("Storage error: {other}");
                Self::Internal(other.to_string())
            }
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            Self::NotFound(m) => (StatusCode::NOT_FOUND, m),
            Self::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, m),
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}
