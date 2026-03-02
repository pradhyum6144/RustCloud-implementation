// ── src/lib.rs ────────────────────────────────────────────────────────────
//! # RustCloud
//!
//! **Unified, provider-agnostic Rust API for modern cloud analytics and AI services.**
//!
//! RustCloud provides composable trait interfaces that abstract over multiple
//! cloud providers, letting you swap backends without rewriting business logic.
//!
//! ## Service Coverage
//!
//! | Category    | Trait             | Providers                     |
//! |-------------|-------------------|-------------------------------|
//! | Analytics   | `BigQueryService` | GCP BigQuery                  |
//! | ML Platform | `VertexAIService` | GCP Vertex AI *(stub)*        |
//! | GenAI       | `GenAIService`    | AWS Bedrock, Azure OpenAI *(stub)* |
//!
//! ## Quick Start
//!
//! ```no_run
//! use rustcloud::analytics::traits::BigQueryService;
//! use rustcloud::analytics::bigquery::GcpBigQuery;
//! use rustcloud::auth::gcp::GcpTokenProvider;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let tp = GcpTokenProvider::from_service_account_file("sa.json").await?;
//!     let bq = GcpBigQuery::new(tp);
//!     let datasets = bq.list_datasets("my-project").await?;
//!     for ds in &datasets {
//!         println!("{}", ds.dataset_id);
//!     }
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod auth;
pub mod analytics;
pub mod ai;
