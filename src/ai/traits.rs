// ── src/ai/traits.rs ──────────────────────────────────────────────────────
//! Trait definitions for AI services: Vertex AI and GenAI.

use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;
use crate::error::RustCloudError;
use super::types::*;

/// Unified interface for Google Vertex AI operations.
///
/// Covers model management, endpoint lifecycle, and prediction (online + batch).
#[async_trait]
pub trait VertexAIService: Send + Sync {
    /// Run online prediction against a deployed model endpoint.
    async fn predict(&self, req: PredictRequest)
        -> Result<PredictResponse, RustCloudError>;

    /// Start a batch prediction job.
    async fn batch_predict(&self, req: BatchPredictRequest)
        -> Result<BatchPredictJob, RustCloudError>;

    /// Deploy a model to an endpoint.
    async fn deploy_model(&self, req: DeployModelRequest)
        -> Result<Endpoint, RustCloudError>;

    /// Undeploy a model from an endpoint.
    async fn undeploy_model(&self, endpoint_id: &str, model_id: &str)
        -> Result<(), RustCloudError>;

    /// List all endpoints in a project/region.
    async fn list_endpoints(&self, project_id: &str, region: &str)
        -> Result<Vec<Endpoint>, RustCloudError>;

    /// List all models in a project/region.
    async fn list_models(&self, project_id: &str, region: &str)
        -> Result<Vec<Model>, RustCloudError>;

    /// Get metadata for a specific model.
    async fn get_model(&self, project_id: &str, region: &str, model_id: &str)
        -> Result<Model, RustCloudError>;
}

/// Provider-agnostic GenAI interface.
///
/// Works for **AWS Bedrock** and **Azure OpenAI Service**. Implementations
/// translate provider-agnostic request types into the native REST calls.
///
/// # Streaming
///
/// The [`GenAIService::chat_stream`] method returns a `Pin<Box<dyn Stream>>`
/// that yields [`StreamChunk`] values token-by-token. Callers consume it via:
///
/// ```ignore
/// use futures::StreamExt;
/// let mut stream = ai.chat_stream(req).await?;
/// while let Some(chunk) = stream.next().await {
///     print!("{}", chunk?.delta);
/// }
/// ```
#[async_trait]
pub trait GenAIService: Send + Sync {
    /// Chat completion (synchronous).
    async fn chat(&self, req: ChatRequest)
        -> Result<ChatResponse, RustCloudError>;

    /// Text completion (non-chat, synchronous).
    async fn complete(&self, req: CompletionRequest)
        -> Result<CompletionResponse, RustCloudError>;

    /// Generate embedding vectors for a batch of inputs.
    async fn embed(&self, req: EmbeddingRequest)
        -> Result<EmbeddingResponse, RustCloudError>;

    /// List available models on this provider.
    async fn list_models(&self)
        -> Result<Vec<ModelInfo>, RustCloudError>;

    /// Streaming chat completion — returns a pinned async stream of delta chunks.
    async fn chat_stream(
        &self,
        req: ChatRequest,
    ) -> Result<
        Pin<Box<dyn Stream<Item = Result<StreamChunk, RustCloudError>> + Send>>,
        RustCloudError,
    >;
}
