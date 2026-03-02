// ── src/auth/mod.rs ───────────────────────────────────────────────────────
//! Authentication modules for each cloud provider.
//!
//! - [`gcp`] — GCP OAuth2 via service-account JWT (RS256 → bearer token)
//! - [`aws`] — AWS Signature Version 4 (HMAC-SHA256 request signing)
//! - [`azure`] — Azure AD client-credentials flow (bearer token)

pub mod gcp;
pub mod aws;
pub mod azure;
