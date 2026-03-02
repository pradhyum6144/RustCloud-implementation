// ── src/error.rs ──────────────────────────────────────────────────────────
//! Unified error type for all RustCloud operations.
//!
//! Every provider-specific error is normalized into [`RustCloudError`] via
//! `From<T>` implementations so callers never interact with provider types.

use thiserror::Error;

/// Unified error type covering all failure modes across providers.
///
/// # Variants
///
/// | Variant      | Typical HTTP status | Meaning                              |
/// |--------------|---------------------|--------------------------------------|
/// | `Auth`       | 401, 403            | Credentials invalid or insufficient  |
/// | `Http`       | —                   | Network / transport failure          |
/// | `Parse`      | —                   | JSON decode or schema mismatch       |
/// | `NotFound`   | 404                 | Requested resource does not exist    |
/// | `RateLimit`  | 429                 | Too many requests; retry after `u64` |
/// | `Quota`      | 429                 | Provider quota exceeded              |
/// | `Provider`   | 4xx / 5xx           | Any other provider-side error        |
/// | `Timeout`    | —                   | Request exceeded configured timeout  |
#[derive(Debug, Error)]
pub enum RustCloudError {
    /// Authentication or authorization failure (HTTP 401/403).
    #[error("Authentication failed: {0}")]
    Auth(String),

    /// Low-level HTTP / network error.
    #[error("HTTP error: {0}")]
    Http(String),

    /// Failed to parse a response body (JSON decode, missing field, etc.).
    #[error("Parse error: {0}")]
    Parse(String),

    /// The requested resource was not found (HTTP 404).
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Rate limit exceeded — the inner value is the suggested retry-after in seconds.
    #[error("Rate limit exceeded — retry after {0}s")]
    RateLimit(u64),

    /// Provider quota exceeded.
    #[error("Quota exceeded: {0}")]
    Quota(String),

    /// Catch-all for other provider-side errors (4xx/5xx not covered above).
    #[error("Provider error: {0}")]
    Provider(String),

    /// The request timed out before the server responded.
    #[error("Request timed out")]
    Timeout,
}

// ── Convenience conversions ───────────────────────────────────────────────

impl From<reqwest::Error> for RustCloudError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            RustCloudError::Timeout
        } else {
            RustCloudError::Http(err.to_string())
        }
    }
}

impl From<serde_json::Error> for RustCloudError {
    fn from(err: serde_json::Error) -> Self {
        RustCloudError::Parse(err.to_string())
    }
}
