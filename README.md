# RustCloud

> Unified, provider-agnostic Rust API for modern cloud analytics and AI services.

RustCloud provides a single, composable trait interface that abstracts over multiple cloud providers — so you can swap backends without rewriting business logic.

## Service Coverage

| Service Category | Trait              | Provider          | Backend                     | Status  |
|------------------|--------------------|-------------------|-----------------------------|---------|
| **Analytics**    | `BigQueryService`  | GCP               | BigQuery REST API v2        | ✅ Done |
| **ML Platform**  | `VertexAIService`  | GCP               | Vertex AI REST API v1       | 🚧 Stub |
| **GenAI (LLM)**  | `GenAIService`     | AWS               | Amazon Bedrock Runtime      | 🚧 Stub |
| **GenAI (LLM)**  | `GenAIService`     | Azure             | Azure OpenAI Service        | 🚧 Stub |

## Quick Start

```rust
use rustcloud::analytics::traits::BigQueryService;
use rustcloud::analytics::bigquery::GcpBigQuery;
use rustcloud::auth::gcp::GcpTokenProvider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token_provider = GcpTokenProvider::from_service_account_file("sa.json").await?;
    let bq = GcpBigQuery::new(token_provider);

    let datasets = bq.list_datasets("my-project").await?;
    for ds in &datasets {
        println!("{}", ds.dataset_id);
    }
    Ok(())
}
```

## Architecture

```
User Application
        │
   Trait Layer  (BigQueryService │ VertexAIService │ GenAIService)
        │
   Provider Impl  (GcpBigQuery │ GcpVertexAI │ AwsBedrock │ AzureOpenAI)
        │
   Auth Layer  (GCP OAuth2 │ AWS SigV4 │ Azure AD)
        │
   Cloud REST APIs
```

## Building

```bash
cargo build
cargo test
cargo doc --no-deps --open
```

## License

MIT
