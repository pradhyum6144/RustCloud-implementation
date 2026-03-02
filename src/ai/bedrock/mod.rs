// ── src/ai/bedrock/mod.rs ────────────────────────────────────────────────
//! AWS Bedrock implementation of [`GenAIService`].
//!
//! **Status**: Stub — full implementation planned for Weeks 9–10.

pub mod types;
pub mod error;

/// AWS Bedrock client — implementation pending.
pub struct AwsBedrock {
    // Will hold: reqwest::Client, AwsSigV4Signer, region
    _private: (),
}
