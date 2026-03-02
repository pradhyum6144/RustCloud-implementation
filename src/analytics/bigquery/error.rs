// ── src/analytics/bigquery/error.rs ──────────────────────────────────────
//! BigQuery-specific error type with automatic conversion to
//! [`RustCloudError`].

use crate::error::RustCloudError;
use std::fmt;

/// BigQuery-specific errors, normalized into [`RustCloudError`] via `From`.
#[derive(Debug)]
pub enum BigQueryError {
    /// Invalid request parameters.
    InvalidRequest(String),
    /// Dataset / table / job not found.
    NotFound(String),
    /// Quota or rate-limit exceeded.
    QuotaExceeded(String),
    /// Unexpected response from the BigQuery API.
    UnexpectedResponse(String),
}

impl fmt::Display for BigQueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(msg) => write!(f, "BigQuery invalid request: {msg}"),
            Self::NotFound(msg) => write!(f, "BigQuery not found: {msg}"),
            Self::QuotaExceeded(msg) => write!(f, "BigQuery quota exceeded: {msg}"),
            Self::UnexpectedResponse(msg) => write!(f, "BigQuery unexpected response: {msg}"),
        }
    }
}

impl std::error::Error for BigQueryError {}

impl From<BigQueryError> for RustCloudError {
    fn from(err: BigQueryError) -> Self {
        match err {
            BigQueryError::InvalidRequest(msg) => RustCloudError::Provider(msg),
            BigQueryError::NotFound(msg) => RustCloudError::NotFound(msg),
            BigQueryError::QuotaExceeded(msg) => RustCloudError::Quota(msg),
            BigQueryError::UnexpectedResponse(msg) => RustCloudError::Provider(msg),
        }
    }
}
