#  RustCloud

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()
[![Tests](https://img.shields.io/badge/tests-10%20passed-brightgreen.svg)]()

> **Unified, provider-agnostic Rust API for modern cloud analytics and AI services.**

RustCloud provides a single, composable trait interface that abstracts over multiple cloud providers — swap backends without rewriting a single line of business logic.

---

##  Problem

Teams building data pipelines, AI-powered services, or analytics platforms must interface with **BigQuery** (GCP), **Vertex AI** (GCP), **Amazon Bedrock** (AWS), and **Azure OpenAI** simultaneously. Each has a different API surface, auth model, SDK convention, and error philosophy. This fragmentation creates:

-  Separate integration layers per provider
-  Rewritten business logic during migrations
-  Increased cognitive load across fundamentally different SDKs
-  Slower iteration in the domains where speed matters most

**RustCloud solves this** with a trait-based abstraction layer — idiomatic Rust, not SDK wrappers.

---

##  Service Coverage

| Service Category | Trait | Provider | Backend API | Status |
|---|---|---|---|---|
| **Analytics** | `BigQueryService` | GCP | BigQuery REST API v2 |  Complete |
| **ML Platform** | `VertexAIService` | GCP | Vertex AI REST API v1 |  Planned |
| **GenAI (LLM)** | `GenAIService` | AWS | Amazon Bedrock Runtime |  Planned |
| **GenAI (LLM)** | `GenAIService` | Azure | Azure OpenAI Service |  Planned |

---

##  Architecture

```
┌──────────────────────────────────────────────────────┐
│                  User Application                     │
│                                                       │
│  let bq: Box<dyn BigQueryService> = Box::new(…);     │
│  let ai: Box<dyn GenAIService>    = Box::new(…);     │
└──────────────────────┬───────────────────────────────┘
                       │
          ┌────────────┴────────────────┐
          │        Trait Layer          │
          │  BigQueryService            │
          │  VertexAIService            │
          │  GenAIService               │
          └────┬───────┬───────┬────────┘
               │       │       │
     ┌─────────┘       │       └──────────┐
     ▼                 ▼                  ▼
┌──────────┐   ┌──────────────┐   ┌────────────────┐
│GcpBigQuery│  │ GcpVertexAI  │   │AwsBedrock      │
│           │  │              │   │AzureOpenAI     │
└─────┬─────┘  └──────┬───────┘   └───────┬────────┘
      │               │                   │
      ▼               ▼                   ▼
┌──────────────────────────────────────────────────────┐
│                    Auth Layer                         │
│  GCP OAuth2 (JWT)  │  AWS SigV4  │  Azure AD        │
└──────────────────────────────────────────────────────┘
      │               │                   │
      ▼               ▼                   ▼
  BigQuery API    Vertex AI API    Bedrock / Azure API
```

---

##  Quick Start

### BigQuery — List Datasets

```rust
use rustcloud::analytics::traits::BigQueryService;
use rustcloud::analytics::bigquery::GcpBigQuery;
use rustcloud::auth::gcp::GcpTokenProvider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load GCP service account credentials
    let tp = GcpTokenProvider::from_service_account_file("sa-key.json").await?;
    let bq = GcpBigQuery::new(tp);

    // Use the trait — code is provider-agnostic
    let datasets = bq.list_datasets("my-gcp-project").await?;
    for ds in &datasets {
        println!(" {} (location: {})", ds.dataset_id, ds.location);
    }
    Ok(())
}
```

### BigQuery — Run SQL Query

```rust
use rustcloud::analytics::bigquery::types::QueryRequest;

let req = QueryRequest {
    project_id: "my-project".into(),
    query: "SELECT name, count FROM `dataset.table` LIMIT 10".into(),
    use_legacy_sql: false,
    timeout_ms: Some(30_000),
    max_results: Some(100),
    location: None,
};

let result = bq.run_query(req).await?;
println!("Returned {} rows (job complete: {})", result.rows.len(), result.job_complete);
```

---

## API Reference

### `BigQueryService` — 11 Methods

| Method | Description |
|---|---|
| `create_dataset(req)` | Create a new dataset |
| `delete_dataset(project, dataset)` | Delete a dataset |
| `list_datasets(project)` | List all datasets in a project |
| `create_table(req)` | Create a new table |
| `delete_table(project, dataset, table)` | Delete a table |
| `list_tables(project, dataset)` | List all tables in a dataset |
| `insert_rows(req)` | Stream rows into a table |
| `run_query(req)` | Execute a synchronous SQL query |
| `get_query_results(project, job_id)` | Retrieve query results |
| `create_job(req)` | Create an async job (load/extract/copy) |
| `get_job(project, job_id)` | Get job status and metadata |

### `VertexAIService` — 7 Methods *(planned)*

| Method | Description |
|---|---|
| `predict(req)` | Online prediction |
| `batch_predict(req)` | Batch prediction job |
| `deploy_model(req)` | Deploy model to endpoint |
| `undeploy_model(endpoint, model)` | Undeploy model |
| `list_endpoints(project, region)` | List endpoints |
| `list_models(project, region)` | List models |
| `get_model(project, region, model)` | Get model metadata |

### `GenAIService` — 5 Methods *(planned)*

| Method | Description |
|---|---|
| `chat(req)` | Chat completion |
| `complete(req)` | Text completion |
| `embed(req)` | Generate embeddings |
| `list_models()` | List available models |
| `chat_stream(req)` | Streaming chat (token-by-token via `futures::Stream`) |

---

##  Authentication

RustCloud implements three production-grade auth flows:

| Provider | Mechanism | Module |
|---|---|---|
| **GCP** | OAuth2 via RS256 JWT → Bearer token | `auth::gcp::GcpTokenProvider` |
| **AWS** | Signature Version 4 (HMAC-SHA256) | `auth::aws::AwsSigV4Signer` |
| **Azure** | Azure AD client-credentials flow | `auth::azure::AzureTokenProvider` |

All token providers include **automatic caching** and **proactive refresh** (60 seconds before expiry) to prevent mid-request auth failures.

```rust
// GCP — from service account JSON
let gcp = GcpTokenProvider::from_service_account_file("sa-key.json").await?;
let token = gcp.get_token().await?;

// AWS — from environment variables
let aws_creds = AwsCredentials::from_env()?;
let signer = AwsSigV4Signer::new(aws_creds, "us-east-1", "bedrock");

// Azure — from environment variables
let azure_creds = AzureCredentials::from_env()?;
let azure = AzureTokenProvider::new(azure_creds);
let token = azure.get_token().await?;
```

---

##  Error Handling

All provider errors are normalized into a single `RustCloudError` enum — callers never see provider-specific types:

```rust
use rustcloud::error::RustCloudError;

match bq.list_datasets("project").await {
    Ok(datasets) => { /* handle success */ }
    Err(RustCloudError::Auth(msg))      => eprintln!("Auth failed: {msg}"),
    Err(RustCloudError::NotFound(msg))  => eprintln!("Not found: {msg}"),
    Err(RustCloudError::RateLimit(s))   => eprintln!("Rate limited, retry in {s}s"),
    Err(RustCloudError::Timeout)        => eprintln!("Request timed out"),
    Err(e)                              => eprintln!("Other error: {e}"),
}
```

| Variant | HTTP Status | Description |
|---|---|---|
| `Auth(String)` | 401, 403 | Invalid or insufficient credentials |
| `Http(String)` | — | Network / transport failure |
| `Parse(String)` | — | JSON decode or schema mismatch |
| `NotFound(String)` | 404 | Resource does not exist |
| `RateLimit(u64)` | 429 | Too many requests (retry-after seconds) |
| `Quota(String)` | 429 | Provider quota exceeded |
| `Provider(String)` | 4xx/5xx | Other provider-side errors |
| `Timeout` | — | Request exceeded timeout |

---

##  Project Structure

```
rustcloud/
├── Cargo.toml
├── src/
│   ├── lib.rs                          # Crate root
│   ├── error.rs                        # RustCloudError enum
│   ├── auth/
│   │   ├── mod.rs
│   │   ├── gcp.rs                      # OAuth2 JWT → bearer token
│   │   ├── aws.rs                      # SigV4 HMAC-SHA256 signing
│   │   └── azure.rs                    # Azure AD client-credentials
│   ├── analytics/
│   │   ├── mod.rs
│   │   ├── traits.rs                   # BigQueryService trait
│   │   └── bigquery/
│   │       ├── mod.rs                  # GcpBigQuery implementation
│   │       ├── types.rs                # Dataset, Table, Job, Query types
│   │       └── error.rs                # BigQueryError → RustCloudError
│   └── ai/
│       ├── mod.rs
│       ├── traits.rs                   # VertexAIService + GenAIService
│       ├── types.rs                    # 20+ shared AI types
│       ├── vertex/                     # GCP Vertex AI (stub)
│       ├── bedrock/                    # AWS Bedrock (stub)
│       └── azure_openai/              # Azure OpenAI (stub)
├── examples/
│   ├── bigquery_list_datasets.rs
│   └── bigquery_run_query.rs
└── tests/
    └── bigquery_mock.rs                # 10 mock tests (mockito)
```

---

##  Testing

```bash
# Run all tests (no cloud credentials needed — uses HTTP mocks)
cargo test

# Run only BigQuery mock tests
cargo test --test bigquery_mock

# Run with output
cargo test -- --nocapture
```

**Current test results:**
```
test test_list_datasets_success ........... ok
test test_list_datasets_empty ............. ok
test test_list_datasets_not_found ......... ok
test test_list_datasets_forbidden ......... ok
test test_create_dataset_success .......... ok
test test_delete_dataset_success .......... ok
test test_run_query_success ............... ok
test test_run_query_server_error .......... ok
test test_insert_rows_success ............. ok
test test_get_job_success ................. ok

test result: ok. 10 passed; 0 failed; 0 ignored
```

---

##  Building

```bash
# Build the library
cargo build

# Build with release optimizations
cargo build --release

# Generate documentation
cargo doc --no-deps --open

# Run clippy lints
cargo clippy
```

---

##  Dependencies

| Crate | Purpose |
|---|---|
| `tokio` | Async runtime |
| `async-trait` | Async trait methods |
| `reqwest` | HTTP client (JSON + streaming) |
| `serde` / `serde_json` | Serialization |
| `thiserror` | Error derive macros |
| `jsonwebtoken` | GCP JWT (RS256) |
| `hmac` / `sha2` | AWS SigV4 signing |
| `futures` | Stream trait for GenAI streaming |
| `chrono` | Timestamp handling |
| `mockito` | HTTP mocking (dev) |

---

##  Roadmap

| Week | Focus | Status |
|---|---|---|
| 1 | Trait interfaces + error architecture |  Complete |
| 2 | Authentication modules (GCP, AWS, Azure) |  Complete |
| 3–4 | BigQuery dataset & table operations | Complete |
| 5 | BigQuery query & job operations | Complete |
| 6 | Integration tests + documentation |  In progress |
| 7–8 | Vertex AI implementation |  Planned |
| 9–10 | Bedrock + Azure OpenAI implementation |  Planned |
| 11–12 | Documentation, CI, hardening |  Planned |


<p align="center">
  Built with  Rust • Part of <a href="https://summerofcode.withgoogle.com/">Google Summer of Code 2026</a>
</p>
