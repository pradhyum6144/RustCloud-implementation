// ── src/ai/mod.rs ─────────────────────────────────────────────────────────
//! AI service abstractions and provider implementations.
//!
//! - [`traits::VertexAIService`] — unified trait for Vertex AI operations
//! - [`traits::GenAIService`] — provider-agnostic GenAI (LLM) trait
//! - [`types`] — shared types used across all AI providers
//! - [`vertex`] — GCP Vertex AI implementation (stub)
//! - [`bedrock`] — AWS Bedrock implementation (stub)
//! - [`azure_openai`] — Azure OpenAI implementation (stub)

pub mod traits;
pub mod types;
pub mod vertex;
pub mod bedrock;
pub mod azure_openai;
