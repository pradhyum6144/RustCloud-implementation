// ── src/ai/vertex/mod.rs ─────────────────────────────────────────────────
//! GCP Vertex AI implementation of [`VertexAIService`].
//!
//! **Status**: Stub — full implementation planned for Weeks 7–8.

pub mod types;
pub mod error;

/// Vertex AI client — implementation pending.
pub struct GcpVertexAI {
    // Will hold: reqwest::Client, GcpTokenProvider, project_id, region
    _private: (),
}
