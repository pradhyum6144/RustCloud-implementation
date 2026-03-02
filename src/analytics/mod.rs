// ── src/analytics/mod.rs ──────────────────────────────────────────────────
//! Analytics service abstractions and provider implementations.
//!
//! - [`traits::BigQueryService`] — unified trait for BigQuery operations
//! - [`bigquery`] — GCP BigQuery implementation

pub mod traits;
pub mod bigquery;
