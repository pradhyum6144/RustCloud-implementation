// ── examples/bigquery_list_datasets.rs ────────────────────────────────────
//! List all datasets in a GCP project using BigQuery.
//!
//! # Usage
//! ```bash
//! export GOOGLE_APPLICATION_CREDENTIALS=path/to/sa-key.json
//! cargo run --example bigquery_list_datasets -- <PROJECT_ID>
//! ```

use rustcloud::analytics::bigquery::GcpBigQuery;
use rustcloud::analytics::traits::BigQueryService;
use rustcloud::auth::gcp::GcpTokenProvider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let project_id = args
        .get(1)
        .expect("Usage: bigquery_list_datasets <PROJECT_ID>");

    let key_path = std::env::var("GOOGLE_APPLICATION_CREDENTIALS")
        .expect("Set GOOGLE_APPLICATION_CREDENTIALS to your SA key file");

    let token_provider = GcpTokenProvider::from_service_account_file(&key_path).await?;
    let bq = GcpBigQuery::new(token_provider);

    println!("Listing datasets in project '{project_id}'…\n");
    let datasets = bq.list_datasets(project_id).await?;

    if datasets.is_empty() {
        println!("  (no datasets found)");
    } else {
        for ds in &datasets {
            println!(
                "  • {} (location: {}, desc: {})",
                ds.dataset_id,
                ds.location,
                ds.description.as_deref().unwrap_or("—"),
            );
        }
    }

    println!("\nTotal: {} dataset(s)", datasets.len());
    Ok(())
}
