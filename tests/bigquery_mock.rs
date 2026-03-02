// ── tests/bigquery_mock.rs ───────────────────────────────────────────────
//! Mock-based unit tests for the BigQuery implementation.
//!
//! Uses `mockito` to simulate BigQuery REST API v2 responses, so these
//! tests run fully offline with no credentials required.

use mockito::{Matcher, Server};
use rustcloud::analytics::bigquery::GcpBigQuery;
use rustcloud::analytics::bigquery::types::*;
use rustcloud::analytics::traits::BigQueryService;
use rustcloud::auth::gcp::{GcpTokenProvider, ServiceAccountKey};
use std::collections::HashMap;

/// Create a GcpTokenProvider that returns a fixed token.
///
/// Since we can't sign real JWTs without a private key, we mock the
/// token exchange endpoint as well.
async fn mock_token_provider(server: &mut Server) -> GcpTokenProvider {
    // Mock the OAuth2 token endpoint
    server
        .mock("POST", "/token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"access_token":"mock-token-12345","expires_in":3600}"#)
        .create_async()
        .await;

    let key = ServiceAccountKey {
        client_email: "test@test.iam.gserviceaccount.com".to_string(),
        private_key: include_str!("test_fixtures/fake_rsa_key.pem").to_string(),
        token_uri: format!("{}/token", server.url()),
        project_id: Some("test-project".to_string()),
    };

    GcpTokenProvider::new(key)
}

/// Helper to create a `GcpBigQuery` pointed at the mock server.
async fn mock_bq(server: &mut Server) -> GcpBigQuery {
    let tp = mock_token_provider(server).await;
    GcpBigQuery::with_base_url(tp, &format!("{}/bigquery/v2", server.url()))
}

// ═══════════════════════════════════════════════════════════════════════════
//  list_datasets
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_list_datasets_success() {
    let mut server = Server::new_async().await;
    let bq = mock_bq(&mut server).await;

    let mock = server
        .mock("GET", "/bigquery/v2/projects/test-project/datasets")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "datasets": [
                    {
                        "datasetReference": {
                            "datasetId": "my_dataset",
                            "projectId": "test-project"
                        },
                        "location": "US",
                        "friendlyName": "My Dataset"
                    }
                ]
            }"#,
        )
        .create_async()
        .await;

    let datasets = bq.list_datasets("test-project").await.unwrap();
    assert_eq!(datasets.len(), 1);
    assert_eq!(datasets[0].dataset_id, "my_dataset");
    assert_eq!(datasets[0].location, "US");

    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_datasets_empty() {
    let mut server = Server::new_async().await;
    let bq = mock_bq(&mut server).await;

    server
        .mock("GET", "/bigquery/v2/projects/test-project/datasets")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{}"#)
        .create_async()
        .await;

    let datasets = bq.list_datasets("test-project").await.unwrap();
    assert!(datasets.is_empty());
}

#[tokio::test]
async fn test_list_datasets_not_found() {
    let mut server = Server::new_async().await;
    let bq = mock_bq(&mut server).await;

    server
        .mock("GET", "/bigquery/v2/projects/bad-project/datasets")
        .with_status(404)
        .with_body(r#"{"error": {"message": "Project not found"}}"#)
        .create_async()
        .await;

    let err = bq.list_datasets("bad-project").await.unwrap_err();
    assert!(
        format!("{err}").contains("not found") || format!("{err}").contains("Not found"),
        "Expected NotFound error, got: {err}"
    );
}

#[tokio::test]
async fn test_list_datasets_forbidden() {
    let mut server = Server::new_async().await;
    let bq = mock_bq(&mut server).await;

    server
        .mock("GET", "/bigquery/v2/projects/test-project/datasets")
        .with_status(403)
        .with_body(r#"{"error": {"message": "Access denied"}}"#)
        .create_async()
        .await;

    let err = bq.list_datasets("test-project").await.unwrap_err();
    assert!(
        format!("{err}").contains("Authentication") || format!("{err}").contains("permission"),
        "Expected Auth error, got: {err}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
//  create_dataset
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_create_dataset_success() {
    let mut server = Server::new_async().await;
    let bq = mock_bq(&mut server).await;

    server
        .mock("POST", "/bigquery/v2/projects/test-project/datasets")
        .match_body(Matcher::PartialJsonString(
            r#"{"datasetReference":{"datasetId":"new_ds"}}"#.to_string(),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "datasetReference": {
                    "datasetId": "new_ds",
                    "projectId": "test-project"
                },
                "location": "EU"
            }"#,
        )
        .create_async()
        .await;

    let req = CreateDatasetRequest {
        project_id: "test-project".to_string(),
        dataset_id: "new_ds".to_string(),
        location: "EU".to_string(),
        description: None,
        labels: HashMap::new(),
    };

    let ds = bq.create_dataset(req).await.unwrap();
    assert_eq!(ds.dataset_id, "new_ds");
    assert_eq!(ds.location, "EU");
}

// ═══════════════════════════════════════════════════════════════════════════
//  run_query
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_run_query_success() {
    let mut server = Server::new_async().await;
    let bq = mock_bq(&mut server).await;

    server
        .mock("POST", "/bigquery/v2/projects/test-project/queries")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "jobReference": { "jobId": "job_abc123" },
                "schema": {
                    "fields": [
                        { "name": "num", "type": "INTEGER", "mode": "NULLABLE" },
                        { "name": "greeting", "type": "STRING", "mode": "NULLABLE" }
                    ]
                },
                "rows": [
                    { "f": [ { "v": "1" }, { "v": "hello" } ] },
                    { "f": [ { "v": "2" }, { "v": "world" } ] }
                ],
                "totalRows": "2",
                "jobComplete": true
            }"#,
        )
        .create_async()
        .await;

    let req = QueryRequest {
        project_id: "test-project".to_string(),
        query: "SELECT 1 AS num, 'hello' AS greeting".to_string(),
        use_legacy_sql: false,
        timeout_ms: None,
        max_results: None,
        location: None,
    };

    let result = bq.run_query(req).await.unwrap();
    assert_eq!(result.job_id, "job_abc123");
    assert!(result.job_complete);
    assert_eq!(result.schema.fields.len(), 2);
    assert_eq!(result.schema.fields[0].name, "num");
    assert_eq!(result.rows.len(), 2);
    assert_eq!(result.total_rows, Some(2));
}

#[tokio::test]
async fn test_run_query_server_error() {
    let mut server = Server::new_async().await;
    let bq = mock_bq(&mut server).await;

    server
        .mock("POST", "/bigquery/v2/projects/test-project/queries")
        .with_status(500)
        .with_body(r#"{"error": {"message": "Internal server error"}}"#)
        .create_async()
        .await;

    let req = QueryRequest {
        project_id: "test-project".to_string(),
        query: "SELECT 1".to_string(),
        use_legacy_sql: false,
        timeout_ms: None,
        max_results: None,
        location: None,
    };

    let err = bq.run_query(req).await.unwrap_err();
    assert!(
        format!("{err}").contains("500") || format!("{err}").contains("Provider"),
        "Expected Provider error, got: {err}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
//  insert_rows
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_insert_rows_success() {
    let mut server = Server::new_async().await;
    let bq = mock_bq(&mut server).await;

    server
        .mock(
            "POST",
            "/bigquery/v2/projects/test-project/datasets/ds1/tables/t1/insertAll",
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{}"#)
        .create_async()
        .await;

    let req = InsertRowsRequest {
        project_id: "test-project".to_string(),
        dataset_id: "ds1".to_string(),
        table_id: "t1".to_string(),
        rows: vec![InsertRow {
            insert_id: Some("row1".to_string()),
            json: serde_json::json!({"name": "Alice", "age": 30}),
        }],
        skip_invalid_rows: false,
        ignore_unknown_values: false,
    };

    let result = bq.insert_rows(req).await.unwrap();
    assert!(result.insert_errors.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
//  delete_dataset
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_delete_dataset_success() {
    let mut server = Server::new_async().await;
    let bq = mock_bq(&mut server).await;

    server
        .mock(
            "DELETE",
            "/bigquery/v2/projects/test-project/datasets/old_ds?deleteContents=true",
        )
        .with_status(204)
        .create_async()
        .await;

    bq.delete_dataset("test-project", "old_ds").await.unwrap();
}

// ═══════════════════════════════════════════════════════════════════════════
//  get_job
// ═══════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_get_job_success() {
    let mut server = Server::new_async().await;
    let bq = mock_bq(&mut server).await;

    server
        .mock("GET", "/bigquery/v2/projects/test-project/jobs/job_xyz")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "jobReference": {
                    "projectId": "test-project",
                    "jobId": "job_xyz"
                },
                "status": { "state": "DONE" },
                "configuration": {},
                "statistics": {}
            }"#,
        )
        .create_async()
        .await;

    let job = bq.get_job("test-project", "job_xyz").await.unwrap();
    assert_eq!(job.job_id, "job_xyz");
    assert_eq!(job.status.state, "DONE");
}
