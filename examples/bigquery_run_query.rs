// ── examples/bigquery_run_query.rs ────────────────────────────────────────
//! Run an SQL query against BigQuery and display results.
//!
//! # Usage
//! ```bash
//! export GOOGLE_APPLICATION_CREDENTIALS=path/to/sa-key.json
//! cargo run --example bigquery_run_query -- <PROJECT_ID> "SELECT 1 AS num, 'hello' AS greeting"
//! ```

use rustcloud::analytics::bigquery::GcpBigQuery;
use rustcloud::analytics::bigquery::types::QueryRequest;
use rustcloud::analytics::traits::BigQueryService;
use rustcloud::auth::gcp::GcpTokenProvider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let project_id = args
        .get(1)
        .expect("Usage: bigquery_run_query <PROJECT_ID> <SQL>");
    let sql = args
        .get(2)
        .expect("Usage: bigquery_run_query <PROJECT_ID> <SQL>");

    let key_path = std::env::var("GOOGLE_APPLICATION_CREDENTIALS")
        .expect("Set GOOGLE_APPLICATION_CREDENTIALS to your SA key file");

    let token_provider = GcpTokenProvider::from_service_account_file(&key_path).await?;
    let bq = GcpBigQuery::new(token_provider);

    let req = QueryRequest {
        project_id: project_id.clone(),
        query: sql.clone(),
        use_legacy_sql: false,
        timeout_ms: Some(30_000),
        max_results: Some(100),
        location: None,
    };

    println!("Running query: {sql}\n");
    let result = bq.run_query(req).await?;

    // Print schema
    print!("  ");
    for field in &result.schema.fields {
        print!("{:<20}", field.name);
    }
    println!();
    print!("  ");
    for _ in &result.schema.fields {
        print!("{:<20}", "────────────────────");
    }
    println!();

    // Print rows
    for row in &result.rows {
        print!("  ");
        for val in row {
            let s = match val {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Null => "NULL".to_string(),
                other => other.to_string(),
            };
            print!("{:<20}", s);
        }
        println!();
    }

    println!(
        "\n{} row(s) returned (job complete: {})",
        result.total_rows.unwrap_or(result.rows.len() as u64),
        result.job_complete,
    );
    Ok(())
}
