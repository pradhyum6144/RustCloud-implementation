// ── src/analytics/bigquery/types.rs ──────────────────────────────────────
//! Data types for BigQuery operations.
//!
//! These types map to the BigQuery REST API v2 request and response
//! structures, normalized into idiomatic Rust.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════════════
//  Dataset
// ═══════════════════════════════════════════════════════════════════════════

/// A BigQuery dataset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    /// Dataset identifier.
    pub dataset_id: String,
    /// Parent project identifier.
    pub project_id: String,
    /// Geographic location (e.g. `"US"`, `"eu-west1"`).
    pub location: String,
    /// Optional human-readable description.
    pub description: Option<String>,
    /// Key-value labels attached to the dataset.
    pub labels: HashMap<String, String>,
    /// Creation timestamp (millis since epoch).
    pub creation_time: Option<i64>,
    /// Last modification timestamp (millis since epoch).
    pub last_modified_time: Option<i64>,
}

/// Parameters for creating a new dataset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDatasetRequest {
    pub project_id: String,
    pub dataset_id: String,
    pub location: String,
    pub description: Option<String>,
    pub labels: HashMap<String, String>,
}

// ═══════════════════════════════════════════════════════════════════════════
//  Table
// ═══════════════════════════════════════════════════════════════════════════

/// A BigQuery table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub table_id: String,
    pub dataset_id: String,
    pub project_id: String,
    pub schema: Option<TableSchema>,
    pub description: Option<String>,
    pub num_rows: Option<u64>,
    pub creation_time: Option<i64>,
    pub last_modified_time: Option<i64>,
}

/// Parameters for creating a new table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTableRequest {
    pub project_id: String,
    pub dataset_id: String,
    pub table_id: String,
    pub schema: TableSchema,
    pub description: Option<String>,
}

/// Table schema — an ordered list of column definitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSchema {
    pub fields: Vec<TableField>,
}

/// A single column in a table schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableField {
    /// Column name.
    pub name: String,
    /// BigQuery type: `STRING`, `INTEGER`, `FLOAT`, `BOOLEAN`, `RECORD`,
    /// `TIMESTAMP`, `DATE`, `BYTES`, `GEOGRAPHY`, `JSON`, etc.
    pub field_type: String,
    /// Mode: `NULLABLE`, `REQUIRED`, `REPEATED`.
    pub mode: String,
    /// Optional description for documentation.
    pub description: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
//  Insert (Streaming)
// ═══════════════════════════════════════════════════════════════════════════

/// Request for streaming row inserts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertRowsRequest {
    pub project_id: String,
    pub dataset_id: String,
    pub table_id: String,
    /// Each value is a JSON object representing a single row.
    pub rows: Vec<InsertRow>,
    /// If `true`, skip invalid rows instead of failing the whole request.
    pub skip_invalid_rows: bool,
    /// If `true`, accept rows that contain values that do not match the
    /// schema; the extra values are ignored.
    pub ignore_unknown_values: bool,
}

/// A single row for insertion, with an optional dedup insert ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertRow {
    /// Optional unique ID for deduplication.
    pub insert_id: Option<String>,
    /// Row data as a JSON object.
    pub json: serde_json::Value,
}

/// Response from a streaming insert.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertRowsResponse {
    /// Errors per row (empty if all rows succeeded).
    pub insert_errors: Vec<InsertError>,
}

/// Error detail for a single failed row insert.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertError {
    pub index: u32,
    pub errors: Vec<ErrorProto>,
}

/// Individual error reason from the BigQuery API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorProto {
    pub reason: String,
    pub location: Option<String>,
    pub message: String,
}

// ═══════════════════════════════════════════════════════════════════════════
//  Query
// ═══════════════════════════════════════════════════════════════════════════

/// Parameters for a synchronous query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryRequest {
    pub project_id: String,
    /// SQL query string (Standard SQL by default).
    pub query: String,
    /// If `true`, interpret the query as Legacy SQL.
    pub use_legacy_sql: bool,
    /// Timeout in milliseconds for the query to complete.
    pub timeout_ms: Option<u64>,
    /// Maximum number of result rows to return.
    pub max_results: Option<u64>,
    /// Processing location hint.
    pub location: Option<String>,
}

/// Result set from a query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResponse {
    /// Job ID that executed this query.
    pub job_id: String,
    /// Schema of the result set.
    pub schema: TableSchema,
    /// Row data — each inner `Vec` corresponds to one row, with values
    /// aligned to the schema fields.
    pub rows: Vec<Vec<serde_json::Value>>,
    /// Total number of rows in the complete result set.
    pub total_rows: Option<u64>,
    /// Pagination token for retrieving subsequent result pages.
    pub page_token: Option<String>,
    /// `true` if the query completed within the timeout.
    pub job_complete: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
//  Job
// ═══════════════════════════════════════════════════════════════════════════

/// Parameters for creating an asynchronous job (query, load, extract, copy).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJobRequest {
    pub project_id: String,
    /// Job configuration as raw JSON — allows query, load, extract, or copy
    /// job configurations without over-constraining the type.
    pub configuration: serde_json::Value,
    /// Optional explicit job ID (auto-generated if omitted).
    pub job_id: Option<String>,
    /// Processing location.
    pub location: Option<String>,
}

/// A BigQuery job (any type).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub job_id: String,
    pub project_id: String,
    pub status: JobStatus,
    pub configuration: Option<serde_json::Value>,
    pub statistics: Option<serde_json::Value>,
    pub creation_time: Option<i64>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
}

/// Job execution status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatus {
    /// Current state: `PENDING`, `RUNNING`, `DONE`.
    pub state: String,
    /// Error result if the job failed.
    pub error_result: Option<ErrorProto>,
}
