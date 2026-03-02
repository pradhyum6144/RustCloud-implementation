// ── src/ai/vertex/error.rs ───────────────────────────────────────────────
//! Vertex AI error types — stub for Weeks 7–8.

use crate::error::RustCloudError;
use std::fmt;

/// Vertex AI-specific errors.
#[derive(Debug)]
pub enum VertexAIError {
    /// Model endpoint not found or not ready.
    EndpointNotReady(String),
    /// Prediction failed.
    PredictionFailed(String),
}

impl fmt::Display for VertexAIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EndpointNotReady(msg) => write!(f, "Vertex AI endpoint not ready: {msg}"),
            Self::PredictionFailed(msg) => write!(f, "Vertex AI prediction failed: {msg}"),
        }
    }
}

impl std::error::Error for VertexAIError {}

impl From<VertexAIError> for RustCloudError {
    fn from(err: VertexAIError) -> Self {
        match err {
            VertexAIError::EndpointNotReady(msg) => RustCloudError::Provider(msg),
            VertexAIError::PredictionFailed(msg) => RustCloudError::Provider(msg),
        }
    }
}
