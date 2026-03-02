// ── src/auth/gcp.rs ───────────────────────────────────────────────────────
//! GCP OAuth2 authentication via service-account JSON key files.
//!
//! Implements the two-step flow:
//! 1. Build an RS256-signed JWT assertion from the service account private key
//! 2. Exchange the JWT for a short-lived OAuth2 bearer token via
//!    `https://oauth2.googleapis.com/token`
//!
//! Tokens are cached in an `Arc<Mutex<…>>` and proactively refreshed 60 s
//! before expiry to avoid mid-request failures.

use crate::error::RustCloudError;
use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

// ── Service-account JSON schema ───────────────────────────────────────────

/// Subset of a GCP service-account JSON key file.
#[derive(Debug, Clone, Deserialize)]
pub struct ServiceAccountKey {
    pub client_email: String,
    pub private_key: String,
    pub token_uri: String,
    pub project_id: Option<String>,
}

// ── JWT claims ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct JwtClaims {
    iss: String,
    scope: String,
    aud: String,
    iat: u64,
    exp: u64,
}

// ── Token cache entry ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct CachedToken {
    access_token: String,
    expires_at: u64, // UNIX timestamp
}

// ── Token exchange response ───────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

// ── Public provider ───────────────────────────────────────────────────────

/// Provides OAuth2 bearer tokens for GCP APIs.
///
/// # Example
/// ```no_run
/// # use rustcloud::auth::gcp::GcpTokenProvider;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let tp = GcpTokenProvider::from_service_account_file("sa-key.json").await?;
/// let token = tp.get_token().await?;
/// println!("Bearer {token}");
/// # Ok(()) }
/// ```
#[derive(Clone)]
pub struct GcpTokenProvider {
    key: ServiceAccountKey,
    scopes: String,
    client: Client,
    cache: Arc<Mutex<Option<CachedToken>>>,
}

impl GcpTokenProvider {
    /// Default scopes covering BigQuery + Vertex AI.
    const DEFAULT_SCOPES: &'static str =
        "https://www.googleapis.com/auth/cloud-platform";

    /// Load credentials from a service-account JSON file on disk.
    pub async fn from_service_account_file(path: &str) -> Result<Self, RustCloudError> {
        let data = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| RustCloudError::Auth(format!("Cannot read key file: {e}")))?;
        let key: ServiceAccountKey = serde_json::from_str(&data)
            .map_err(|e| RustCloudError::Auth(format!("Invalid key JSON: {e}")))?;
        Ok(Self::new(key))
    }

    /// Create from an already-parsed [`ServiceAccountKey`].
    pub fn new(key: ServiceAccountKey) -> Self {
        Self {
            key,
            scopes: Self::DEFAULT_SCOPES.to_string(),
            client: Client::new(),
            cache: Arc::new(Mutex::new(None)),
        }
    }

    /// Override the default scopes.
    pub fn with_scopes(mut self, scopes: &str) -> Self {
        self.scopes = scopes.to_string();
        self
    }

    /// Return a valid bearer token, refreshing if necessary.
    ///
    /// Tokens are refreshed proactively 60 s before expiry.
    pub async fn get_token(&self) -> Result<String, RustCloudError> {
        let mut guard = self.cache.lock().await;
        let now = now_secs();

        if let Some(cached) = guard.as_ref() {
            if cached.expires_at > now + 60 {
                return Ok(cached.access_token.clone());
            }
        }

        let token = self.fetch_token(now).await?;
        let access = token.access_token.clone();
        *guard = Some(CachedToken {
            access_token: token.access_token,
            expires_at: now + token.expires_in,
        });
        Ok(access)
    }

    /// Build JWT, exchange for bearer token.
    async fn fetch_token(&self, now: u64) -> Result<TokenResponse, RustCloudError> {
        let claims = JwtClaims {
            iss: self.key.client_email.clone(),
            scope: self.scopes.clone(),
            aud: self.key.token_uri.clone(),
            iat: now,
            exp: now + 3600,
        };

        let header = Header::new(Algorithm::RS256);
        let encoding_key = EncodingKey::from_rsa_pem(self.key.private_key.as_bytes())
            .map_err(|e| RustCloudError::Auth(format!("Invalid RSA key: {e}")))?;

        let jwt = encode(&header, &claims, &encoding_key)
            .map_err(|e| RustCloudError::Auth(format!("JWT encode failed: {e}")))?;

        let resp = self
            .client
            .post(&self.key.token_uri)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", &jwt),
            ])
            .send()
            .await
            .map_err(|e| RustCloudError::Http(format!("Token exchange: {e}")))?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(RustCloudError::Auth(format!("Token exchange failed: {text}")));
        }

        resp.json::<TokenResponse>()
            .await
            .map_err(|e| RustCloudError::Parse(format!("Token response: {e}")))
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before UNIX epoch")
        .as_secs()
}
