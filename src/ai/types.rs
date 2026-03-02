// ── src/ai/types.rs ───────────────────────────────────────────────────────
//! Provider-agnostic types shared across all AI service implementations.
//!
//! These types are used in the [`super::traits::GenAIService`] and
//! [`super::traits::VertexAIService`] trait signatures, ensuring that
//! user code is fully decoupled from any specific provider.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════════════
//  GenAI Types (used by Bedrock + Azure OpenAI)
// ═══════════════════════════════════════════════════════════════════════════

/// A single message in a chat conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Role of the message author: `"system"`, `"user"`, or `"assistant"`.
    pub role: String,
    /// Message content.
    pub content: String,
}

/// Request to a chat-completion endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    /// Model identifier (e.g. `"anthropic.claude-3-sonnet"`, `"gpt-4"`).
    pub model: String,
    /// Conversation messages.
    pub messages: Vec<ChatMessage>,
    /// Sampling temperature (0.0–2.0).
    pub temperature: Option<f64>,
    /// Maximum tokens to generate.
    pub max_tokens: Option<u32>,
    /// Top-p nucleus sampling.
    pub top_p: Option<f64>,
    /// Stop sequences.
    pub stop: Option<Vec<String>>,
}

/// Response from a chat-completion endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// Generated message.
    pub message: ChatMessage,
    /// Model used for generation.
    pub model: String,
    /// Usage statistics.
    pub usage: Option<Usage>,
    /// Finish reason (e.g. `"stop"`, `"length"`).
    pub finish_reason: Option<String>,
}

/// Token-by-token streaming delta.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    /// Incremental text content.
    pub delta: String,
    /// Set when the stream is complete.
    pub finish_reason: Option<String>,
    /// Chunk index (for multi-choice responses).
    pub index: u32,
}

/// Request for text completion (non-chat).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub model: String,
    pub prompt: String,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f64>,
    pub stop: Option<Vec<String>>,
}

/// Response from a text completion endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub text: String,
    pub model: String,
    pub usage: Option<Usage>,
    pub finish_reason: Option<String>,
}

/// Request for embedding generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    pub model: String,
    pub input: Vec<String>,
}

/// Response containing embedding vectors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub embeddings: Vec<Vec<f64>>,
    pub model: String,
    pub usage: Option<Usage>,
}

/// Token usage statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Metadata about an available model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Provider-specific model identifier.
    pub model_id: String,
    /// Human-readable model name.
    pub name: Option<String>,
    /// Provider that hosts this model.
    pub provider: String,
    /// Model description.
    pub description: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
//  Vertex AI Types
// ═══════════════════════════════════════════════════════════════════════════

/// Request for Vertex AI online prediction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictRequest {
    /// Project ID.
    pub project_id: String,
    /// GCP region (e.g. `"us-central1"`).
    pub region: String,
    /// Endpoint ID.
    pub endpoint_id: String,
    /// Input instances as JSON values.
    pub instances: Vec<serde_json::Value>,
    /// Optional prediction parameters.
    pub parameters: Option<serde_json::Value>,
}

/// Response from Vertex AI prediction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictResponse {
    /// One prediction per input instance.
    pub predictions: Vec<serde_json::Value>,
    /// Model resource name that served the prediction.
    pub deployed_model_id: Option<String>,
}

/// Request for Vertex AI batch prediction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchPredictRequest {
    pub project_id: String,
    pub region: String,
    pub model_id: String,
    pub input_uri: String,
    pub output_uri: String,
    pub parameters: Option<serde_json::Value>,
}

/// Metadata for a batch prediction job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchPredictJob {
    pub job_id: String,
    pub state: String,
    pub create_time: Option<String>,
    pub update_time: Option<String>,
}

/// Request to deploy a model to an endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployModelRequest {
    pub project_id: String,
    pub region: String,
    pub endpoint_id: String,
    pub model_id: String,
    /// Machine type for the deployed model (e.g. `"n1-standard-4"`).
    pub machine_type: Option<String>,
    /// Minimum replica count for auto-scaling.
    pub min_replica_count: Option<u32>,
    /// Maximum replica count for auto-scaling.
    pub max_replica_count: Option<u32>,
}

/// A Vertex AI endpoint serving one or more models.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    pub endpoint_id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub deployed_models: Vec<DeployedModel>,
    pub create_time: Option<String>,
    pub update_time: Option<String>,
}

/// A model deployed to an endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployedModel {
    pub model_id: String,
    pub display_name: Option<String>,
    pub machine_type: Option<String>,
}

/// Metadata for a Vertex AI model resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub model_id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub create_time: Option<String>,
    pub update_time: Option<String>,
    pub labels: HashMap<String, String>,
}
