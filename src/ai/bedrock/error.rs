// ── src/ai/bedrock/error.rs ──────────────────────────────────────────────
//! AWS Bedrock error types — stub for Weeks 9–10.

use crate::error::RustCloudError;
use std::fmt;

/// Bedrock-specific errors.
#[derive(Debug)]
pub enum BedrockError {
    /// Model not available or not provisioned.
    ModelNotAvailable(String),
    /// Throttling / rate limit from Bedrock.
    Throttled(String),
}

impl fmt::Display for BedrockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ModelNotAvailable(msg) => write!(f, "Bedrock model not available: {msg}"),
            Self::Throttled(msg) => write!(f, "Bedrock throttled: {msg}"),
        }
    }
}

impl std::error::Error for BedrockError {}

impl From<BedrockError> for RustCloudError {
    fn from(err: BedrockError) -> Self {
        match err {
            BedrockError::ModelNotAvailable(msg) => RustCloudError::Provider(msg),
            BedrockError::Throttled(_msg) => RustCloudError::RateLimit(30),
        }
    }
}
