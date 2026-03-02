// ── src/ai/azure_openai/error.rs ─────────────────────────────────────────
//! Azure OpenAI error types — stub for Week 10.

use crate::error::RustCloudError;
use std::fmt;

/// Azure OpenAI-specific errors.
#[derive(Debug)]
pub enum AzureOpenAIError {
    /// Deployment not found.
    DeploymentNotFound(String),
    /// Content filter triggered.
    ContentFiltered(String),
}

impl fmt::Display for AzureOpenAIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeploymentNotFound(msg) => write!(f, "Azure OpenAI deployment not found: {msg}"),
            Self::ContentFiltered(msg) => write!(f, "Azure OpenAI content filtered: {msg}"),
        }
    }
}

impl std::error::Error for AzureOpenAIError {}

impl From<AzureOpenAIError> for RustCloudError {
    fn from(err: AzureOpenAIError) -> Self {
        match err {
            AzureOpenAIError::DeploymentNotFound(msg) => RustCloudError::NotFound(msg),
            AzureOpenAIError::ContentFiltered(msg) => RustCloudError::Provider(msg),
        }
    }
}
