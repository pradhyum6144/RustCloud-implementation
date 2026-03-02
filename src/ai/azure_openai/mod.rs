// ── src/ai/azure_openai/mod.rs ───────────────────────────────────────────
//! Azure OpenAI implementation of [`GenAIService`].
//!
//! **Status**: Stub — full implementation planned for Week 10.

pub mod types;
pub mod error;

/// Azure OpenAI client — implementation pending.
pub struct AzureOpenAI {
    // Will hold: reqwest::Client, AzureTokenProvider, endpoint, api_version
    _private: (),
}
