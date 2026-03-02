// ── src/analytics/traits.rs ───────────────────────────────────────────────
//! Trait definition for BigQuery-compatible analytics services.

use async_trait::async_trait;
use crate::error::RustCloudError;
use super::bigquery::types::*;

/// Unified interface for Google BigQuery operations.
///
/// Any struct implementing this trait can be used interchangeably,
/// enabling provider-agnostic application code via `dyn BigQueryService`.
///
/// # Example
/// ```no_run
/// # use rustcloud::analytics::traits::BigQueryService;
/// # async fn example(bq: &dyn BigQueryService) -> Result<(), Box<dyn std::error::Error>> {
/// let datasets = bq.list_datasets("my-project").await?;
/// for ds in &datasets {
///     println!("{}: {}", ds.dataset_id, ds.location);
/// }
/// # Ok(()) }
/// ```
#[async_trait]
pub trait BigQueryService: Send + Sync {
    /// Create a new dataset in the specified project.
    async fn create_dataset(&self, req: CreateDatasetRequest)
        -> Result<Dataset, RustCloudError>;

    /// Delete an existing dataset.
    async fn delete_dataset(&self, project_id: &str, dataset_id: &str)
        -> Result<(), RustCloudError>;

    /// List all datasets in a project.
    async fn list_datasets(&self, project_id: &str)
        -> Result<Vec<Dataset>, RustCloudError>;

    /// Create a new table within a dataset.
    async fn create_table(&self, req: CreateTableRequest)
        -> Result<Table, RustCloudError>;

    /// Delete an existing table.
    async fn delete_table(
        &self,
        project_id: &str,
        dataset_id: &str,
        table_id: &str,
    ) -> Result<(), RustCloudError>;

    /// List all tables in a dataset.
    async fn list_tables(&self, project_id: &str, dataset_id: &str)
        -> Result<Vec<Table>, RustCloudError>;

    /// Insert rows into a table via the streaming insert API.
    async fn insert_rows(&self, req: InsertRowsRequest)
        -> Result<InsertRowsResponse, RustCloudError>;

    /// Execute a synchronous SQL query.
    async fn run_query(&self, req: QueryRequest)
        -> Result<QueryResponse, RustCloudError>;

    /// Retrieve results from a previously started query job.
    async fn get_query_results(&self, project_id: &str, job_id: &str)
        -> Result<QueryResponse, RustCloudError>;

    /// Create an asynchronous job (load, extract, copy, or query).
    async fn create_job(&self, req: CreateJobRequest)
        -> Result<Job, RustCloudError>;

    /// Get the status and metadata of a job.
    async fn get_job(&self, project_id: &str, job_id: &str)
        -> Result<Job, RustCloudError>;
}
