// ── src/analytics/bigquery/mod.rs ────────────────────────────────────────
//! GCP BigQuery implementation of [`BigQueryService`].
//!
//! Connects to the BigQuery REST API v2 at
//! `https://bigquery.googleapis.com/bigquery/v2`.

pub mod types;
pub mod error;

use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde_json::json;

use crate::analytics::traits::BigQueryService;
use crate::auth::gcp::GcpTokenProvider;
use crate::error::RustCloudError;
use types::*;

/// BigQuery client backed by the GCP REST API v2.
///
/// # Example
/// ```no_run
/// # use rustcloud::analytics::bigquery::GcpBigQuery;
/// # use rustcloud::auth::gcp::GcpTokenProvider;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let tp = GcpTokenProvider::from_service_account_file("sa.json").await?;
/// let bq = GcpBigQuery::new(tp);
/// // bq implements BigQueryService — use it via the trait
/// # Ok(()) }
/// ```
pub struct GcpBigQuery {
    client: Client,
    token_provider: GcpTokenProvider,
    base_url: String,
}

impl GcpBigQuery {
    /// Create a new BigQuery client with the given token provider.
    pub fn new(token_provider: GcpTokenProvider) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .tcp_keepalive(std::time::Duration::from_secs(60))
                .build()
                .expect("HTTP client build failed"),
            token_provider,
            base_url: "https://bigquery.googleapis.com/bigquery/v2".into(),
        }
    }

    /// Create with a custom base URL (useful for testing).
    #[doc(hidden)]
    pub fn with_base_url(token_provider: GcpTokenProvider, base_url: &str) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("HTTP client build failed"),
            token_provider,
            base_url: base_url.to_string(),
        }
    }

    /// Obtain a fresh bearer token.
    async fn token(&self) -> Result<String, RustCloudError> {
        self.token_provider.get_token().await
    }

    /// Map a non-OK HTTP status to the appropriate [`RustCloudError`].
    fn map_status(status: StatusCode, text: &str) -> RustCloudError {
        match status {
            StatusCode::UNAUTHORIZED => RustCloudError::Auth(text.to_string()),
            StatusCode::FORBIDDEN => RustCloudError::Auth(
                format!("Insufficient BigQuery permissions: {text}")
            ),
            StatusCode::NOT_FOUND => RustCloudError::NotFound(text.to_string()),
            StatusCode::TOO_MANY_REQUESTS => {
                // Attempt to extract retry-after from the body
                RustCloudError::RateLimit(30)
            }
            _ => RustCloudError::Provider(
                format!("BigQuery {status} — {text}")
            ),
        }
    }
}

#[async_trait]
impl BigQueryService for GcpBigQuery {
    // ── Dataset operations ────────────────────────────────────────────────

    async fn create_dataset(&self, req: CreateDatasetRequest)
        -> Result<Dataset, RustCloudError>
    {
        let token = self.token().await?;
        let url = format!("{}/projects/{}/datasets", self.base_url, req.project_id);

        let body = json!({
            "datasetReference": {
                "datasetId": req.dataset_id,
                "projectId": req.project_id,
            },
            "location": req.location,
            "friendlyName": req.description,
            "labels": req.labels,
        });

        let resp = self.client
            .post(&url)
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await
            .map_err(|e| RustCloudError::Http(e.to_string()))?;

        match resp.status() {
            StatusCode::OK | StatusCode::CREATED => {
                let raw: serde_json::Value = resp.json().await
                    .map_err(|e| RustCloudError::Parse(e.to_string()))?;
                Ok(Dataset {
                    dataset_id: raw["datasetReference"]["datasetId"]
                        .as_str().unwrap_or("").to_string(),
                    project_id: raw["datasetReference"]["projectId"]
                        .as_str().unwrap_or("").to_string(),
                    location: raw["location"]
                        .as_str().unwrap_or("").to_string(),
                    description: raw["friendlyName"]
                        .as_str().map(|s| s.to_string()),
                    labels: serde_json::from_value(
                        raw["labels"].clone()
                    ).unwrap_or_default(),
                    creation_time: raw["creationTime"]
                        .as_str().and_then(|s| s.parse().ok()),
                    last_modified_time: raw["lastModifiedTime"]
                        .as_str().and_then(|s| s.parse().ok()),
                })
            }
            status => {
                let text = resp.text().await.unwrap_or_default();
                Err(Self::map_status(status, &text))
            }
        }
    }

    async fn delete_dataset(&self, project_id: &str, dataset_id: &str)
        -> Result<(), RustCloudError>
    {
        let token = self.token().await?;
        let url = format!(
            "{}/projects/{}/datasets/{}?deleteContents=true",
            self.base_url, project_id, dataset_id,
        );

        let resp = self.client
            .delete(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| RustCloudError::Http(e.to_string()))?;

        match resp.status() {
            StatusCode::OK | StatusCode::NO_CONTENT => Ok(()),
            status => {
                let text = resp.text().await.unwrap_or_default();
                Err(Self::map_status(status, &text))
            }
        }
    }

    async fn list_datasets(&self, project_id: &str)
        -> Result<Vec<Dataset>, RustCloudError>
    {
        let token = self.token().await?;
        let url = format!("{}/projects/{}/datasets", self.base_url, project_id);

        let resp = self.client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| RustCloudError::Http(e.to_string()))?;

        match resp.status() {
            StatusCode::OK => {
                let body: serde_json::Value = resp.json().await
                    .map_err(|e| RustCloudError::Parse(e.to_string()))?;
                let arr = body["datasets"].as_array().cloned().unwrap_or_default();
                arr.iter()
                    .map(|d| {
                        Ok(Dataset {
                            dataset_id: d["datasetReference"]["datasetId"]
                                .as_str().unwrap_or("").to_string(),
                            project_id: d["datasetReference"]["projectId"]
                                .as_str().unwrap_or("").to_string(),
                            location: d["location"]
                                .as_str().unwrap_or("").to_string(),
                            description: d["friendlyName"]
                                .as_str().map(|s| s.to_string()),
                            labels: serde_json::from_value(
                                d["labels"].clone()
                            ).unwrap_or_default(),
                            creation_time: d["creationTime"]
                                .as_str().and_then(|s| s.parse().ok()),
                            last_modified_time: d["lastModifiedTime"]
                                .as_str().and_then(|s| s.parse().ok()),
                        })
                    })
                    .collect()
            }
            status => {
                let text = resp.text().await.unwrap_or_default();
                Err(Self::map_status(status, &text))
            }
        }
    }

    // ── Table operations ──────────────────────────────────────────────────

    async fn create_table(&self, req: CreateTableRequest)
        -> Result<Table, RustCloudError>
    {
        let token = self.token().await?;
        let url = format!(
            "{}/projects/{}/datasets/{}/tables",
            self.base_url, req.project_id, req.dataset_id,
        );

        let fields: Vec<serde_json::Value> = req.schema.fields.iter().map(|f| {
            json!({
                "name": f.name,
                "type": f.field_type,
                "mode": f.mode,
                "description": f.description,
            })
        }).collect();

        let body = json!({
            "tableReference": {
                "projectId": req.project_id,
                "datasetId": req.dataset_id,
                "tableId":   req.table_id,
            },
            "schema": { "fields": fields },
            "description": req.description,
        });

        let resp = self.client
            .post(&url)
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await
            .map_err(|e| RustCloudError::Http(e.to_string()))?;

        match resp.status() {
            StatusCode::OK | StatusCode::CREATED => {
                let raw: serde_json::Value = resp.json().await
                    .map_err(|e| RustCloudError::Parse(e.to_string()))?;
                Ok(parse_table(&raw))
            }
            status => {
                let text = resp.text().await.unwrap_or_default();
                Err(Self::map_status(status, &text))
            }
        }
    }

    async fn delete_table(
        &self,
        project_id: &str,
        dataset_id: &str,
        table_id: &str,
    ) -> Result<(), RustCloudError> {
        let token = self.token().await?;
        let url = format!(
            "{}/projects/{}/datasets/{}/tables/{}",
            self.base_url, project_id, dataset_id, table_id,
        );

        let resp = self.client
            .delete(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| RustCloudError::Http(e.to_string()))?;

        match resp.status() {
            StatusCode::OK | StatusCode::NO_CONTENT => Ok(()),
            status => {
                let text = resp.text().await.unwrap_or_default();
                Err(Self::map_status(status, &text))
            }
        }
    }

    async fn list_tables(&self, project_id: &str, dataset_id: &str)
        -> Result<Vec<Table>, RustCloudError>
    {
        let token = self.token().await?;
        let url = format!(
            "{}/projects/{}/datasets/{}/tables",
            self.base_url, project_id, dataset_id,
        );

        let resp = self.client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| RustCloudError::Http(e.to_string()))?;

        match resp.status() {
            StatusCode::OK => {
                let body: serde_json::Value = resp.json().await
                    .map_err(|e| RustCloudError::Parse(e.to_string()))?;
                let arr = body["tables"].as_array().cloned().unwrap_or_default();
                Ok(arr.iter().map(parse_table).collect())
            }
            status => {
                let text = resp.text().await.unwrap_or_default();
                Err(Self::map_status(status, &text))
            }
        }
    }

    // ── Insert ────────────────────────────────────────────────────────────

    async fn insert_rows(&self, req: InsertRowsRequest)
        -> Result<InsertRowsResponse, RustCloudError>
    {
        let token = self.token().await?;
        let url = format!(
            "{}/projects/{}/datasets/{}/tables/{}/insertAll",
            self.base_url, req.project_id, req.dataset_id, req.table_id,
        );

        let rows: Vec<serde_json::Value> = req.rows.iter().map(|r| {
            let mut row = json!({ "json": r.json });
            if let Some(ref id) = r.insert_id {
                row["insertId"] = json!(id);
            }
            row
        }).collect();

        let body = json!({
            "rows": rows,
            "skipInvalidRows": req.skip_invalid_rows,
            "ignoreUnknownValues": req.ignore_unknown_values,
        });

        let resp = self.client
            .post(&url)
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await
            .map_err(|e| RustCloudError::Http(e.to_string()))?;

        match resp.status() {
            StatusCode::OK => {
                let raw: serde_json::Value = resp.json().await
                    .map_err(|e| RustCloudError::Parse(e.to_string()))?;
                let insert_errors = raw["insertErrors"]
                    .as_array()
                    .cloned()
                    .unwrap_or_default()
                    .iter()
                    .map(|e| InsertError {
                        index: e["index"].as_u64().unwrap_or(0) as u32,
                        errors: e["errors"]
                            .as_array()
                            .cloned()
                            .unwrap_or_default()
                            .iter()
                            .map(|err| ErrorProto {
                                reason: err["reason"]
                                    .as_str().unwrap_or("").to_string(),
                                location: err["location"]
                                    .as_str().map(|s| s.to_string()),
                                message: err["message"]
                                    .as_str().unwrap_or("").to_string(),
                            })
                            .collect(),
                    })
                    .collect();
                Ok(InsertRowsResponse { insert_errors })
            }
            status => {
                let text = resp.text().await.unwrap_or_default();
                Err(Self::map_status(status, &text))
            }
        }
    }

    // ── Query ─────────────────────────────────────────────────────────────

    async fn run_query(&self, req: QueryRequest)
        -> Result<QueryResponse, RustCloudError>
    {
        let token = self.token().await?;
        let url = format!("{}/projects/{}/queries", self.base_url, req.project_id);

        let body = json!({
            "query":        req.query,
            "useLegacySql": req.use_legacy_sql,
            "timeoutMs":    req.timeout_ms.unwrap_or(30_000),
            "maxResults":   req.max_results.unwrap_or(1_000),
            "location":     req.location,
        });

        let resp = self.client
            .post(&url)
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await
            .map_err(|e| RustCloudError::Http(e.to_string()))?;

        match resp.status() {
            StatusCode::OK => {
                let raw: serde_json::Value = resp.json().await
                    .map_err(|e| RustCloudError::Parse(e.to_string()))?;
                Ok(parse_query_response(&raw))
            }
            status => {
                let text = resp.text().await.unwrap_or_default();
                Err(Self::map_status(status, &text))
            }
        }
    }

    async fn get_query_results(&self, project_id: &str, job_id: &str)
        -> Result<QueryResponse, RustCloudError>
    {
        let token = self.token().await?;
        let url = format!(
            "{}/projects/{}/queries/{}",
            self.base_url, project_id, job_id,
        );

        let resp = self.client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| RustCloudError::Http(e.to_string()))?;

        match resp.status() {
            StatusCode::OK => {
                let raw: serde_json::Value = resp.json().await
                    .map_err(|e| RustCloudError::Parse(e.to_string()))?;
                Ok(parse_query_response(&raw))
            }
            status => {
                let text = resp.text().await.unwrap_or_default();
                Err(Self::map_status(status, &text))
            }
        }
    }

    // ── Job ───────────────────────────────────────────────────────────────

    async fn create_job(&self, req: CreateJobRequest)
        -> Result<Job, RustCloudError>
    {
        let token = self.token().await?;
        let url = format!("{}/projects/{}/jobs", self.base_url, req.project_id);

        let mut body = json!({
            "configuration": req.configuration,
        });
        if let Some(ref id) = req.job_id {
            body["jobReference"] = json!({
                "projectId": req.project_id,
                "jobId": id,
            });
        }
        if let Some(ref loc) = req.location {
            body["jobReference"]["location"] = json!(loc);
        }

        let resp = self.client
            .post(&url)
            .bearer_auth(&token)
            .json(&body)
            .send()
            .await
            .map_err(|e| RustCloudError::Http(e.to_string()))?;

        match resp.status() {
            StatusCode::OK | StatusCode::CREATED => {
                let raw: serde_json::Value = resp.json().await
                    .map_err(|e| RustCloudError::Parse(e.to_string()))?;
                Ok(parse_job(&raw))
            }
            status => {
                let text = resp.text().await.unwrap_or_default();
                Err(Self::map_status(status, &text))
            }
        }
    }

    async fn get_job(&self, project_id: &str, job_id: &str)
        -> Result<Job, RustCloudError>
    {
        let token = self.token().await?;
        let url = format!(
            "{}/projects/{}/jobs/{}",
            self.base_url, project_id, job_id,
        );

        let resp = self.client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| RustCloudError::Http(e.to_string()))?;

        match resp.status() {
            StatusCode::OK => {
                let raw: serde_json::Value = resp.json().await
                    .map_err(|e| RustCloudError::Parse(e.to_string()))?;
                Ok(parse_job(&raw))
            }
            status => {
                let text = resp.text().await.unwrap_or_default();
                Err(Self::map_status(status, &text))
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  Internal parse helpers
// ═══════════════════════════════════════════════════════════════════════════

fn parse_table(raw: &serde_json::Value) -> Table {
    let schema = raw["schema"]["fields"].as_array().map(|fields| {
        TableSchema {
            fields: fields.iter().map(|f| TableField {
                name: f["name"].as_str().unwrap_or("").to_string(),
                field_type: f["type"].as_str().unwrap_or("").to_string(),
                mode: f["mode"].as_str().unwrap_or("NULLABLE").to_string(),
                description: f["description"].as_str().map(|s| s.to_string()),
            }).collect(),
        }
    });

    Table {
        table_id: raw["tableReference"]["tableId"]
            .as_str().unwrap_or("").to_string(),
        dataset_id: raw["tableReference"]["datasetId"]
            .as_str().unwrap_or("").to_string(),
        project_id: raw["tableReference"]["projectId"]
            .as_str().unwrap_or("").to_string(),
        schema,
        description: raw["description"].as_str().map(|s| s.to_string()),
        num_rows: raw["numRows"].as_str().and_then(|s| s.parse().ok()),
        creation_time: raw["creationTime"]
            .as_str().and_then(|s| s.parse().ok()),
        last_modified_time: raw["lastModifiedTime"]
            .as_str().and_then(|s| s.parse().ok()),
    }
}

fn parse_query_response(raw: &serde_json::Value) -> QueryResponse {
    let schema = raw["schema"]["fields"]
        .as_array()
        .map(|fields| {
            TableSchema {
                fields: fields.iter().map(|f| TableField {
                    name: f["name"].as_str().unwrap_or("").to_string(),
                    field_type: f["type"].as_str().unwrap_or("").to_string(),
                    mode: f["mode"].as_str().unwrap_or("NULLABLE").to_string(),
                    description: f["description"].as_str().map(|s| s.to_string()),
                }).collect(),
            }
        })
        .unwrap_or(TableSchema { fields: vec![] });

    let rows = raw["rows"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .map(|r| {
            r["f"]
                .as_array()
                .cloned()
                .unwrap_or_default()
                .iter()
                .map(|v| v["v"].clone())
                .collect()
        })
        .collect();

    QueryResponse {
        job_id: raw["jobReference"]["jobId"]
            .as_str().unwrap_or("").to_string(),
        schema,
        rows,
        total_rows: raw["totalRows"]
            .as_str().and_then(|s| s.parse().ok()),
        page_token: raw["pageToken"]
            .as_str().map(|s| s.to_string()),
        job_complete: raw["jobComplete"]
            .as_bool().unwrap_or(false),
    }
}

fn parse_job(raw: &serde_json::Value) -> Job {
    Job {
        job_id: raw["jobReference"]["jobId"]
            .as_str().unwrap_or("").to_string(),
        project_id: raw["jobReference"]["projectId"]
            .as_str().unwrap_or("").to_string(),
        status: JobStatus {
            state: raw["status"]["state"]
                .as_str().unwrap_or("UNKNOWN").to_string(),
            error_result: raw["status"]["errorResult"].as_object().map(|e| {
                ErrorProto {
                    reason: e.get("reason")
                        .and_then(|v| v.as_str())
                        .unwrap_or("").to_string(),
                    location: e.get("location")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    message: e.get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("").to_string(),
                }
            }),
        },
        configuration: Some(raw["configuration"].clone()),
        statistics: Some(raw["statistics"].clone()),
        creation_time: raw["statistics"]["creationTime"]
            .as_str().and_then(|s| s.parse().ok()),
        start_time: raw["statistics"]["startTime"]
            .as_str().and_then(|s| s.parse().ok()),
        end_time: raw["statistics"]["endTime"]
            .as_str().and_then(|s| s.parse().ok()),
    }
}
