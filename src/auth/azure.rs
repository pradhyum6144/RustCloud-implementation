// ── src/auth/azure.rs ─────────────────────────────────────────────────────
//! Azure Active Directory client-credentials authentication.
//!
//! Implements the OAuth2 client-credentials flow against:
//! `https://login.microsoftonline.com/{tenant_id}/oauth2/v2.0/token`
//!
//! Tokens are cached and refreshed proactively (same pattern as GCP).

use crate::error::RustCloudError;
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

/// Azure AD credentials.
#[derive(Debug, Clone)]
pub struct AzureCredentials {
    pub tenant_id: String,
    pub client_id: String,
    pub client_secret: String,
}

impl AzureCredentials {
    /// Load from environment variables `AZURE_TENANT_ID`, `AZURE_CLIENT_ID`,
    /// `AZURE_CLIENT_SECRET`.
    pub fn from_env() -> Result<Self, RustCloudError> {
        let tenant_id = std::env::var("AZURE_TENANT_ID")
            .map_err(|_| RustCloudError::Auth("AZURE_TENANT_ID not set".into()))?;
        let client_id = std::env::var("AZURE_CLIENT_ID")
            .map_err(|_| RustCloudError::Auth("AZURE_CLIENT_ID not set".into()))?;
        let client_secret = std::env::var("AZURE_CLIENT_SECRET")
            .map_err(|_| RustCloudError::Auth("AZURE_CLIENT_SECRET not set".into()))?;
        Ok(Self { tenant_id, client_id, client_secret })
    }
}

// ── Internal types ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct CachedToken {
    access_token: String,
    expires_at: u64,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

// ── Public provider ───────────────────────────────────────────────────────

/// Provides OAuth2 bearer tokens for Azure OpenAI Service.
///
/// # Example
/// ```no_run
/// # use rustcloud::auth::azure::{AzureTokenProvider, AzureCredentials};
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let creds = AzureCredentials::from_env()?;
/// let tp = AzureTokenProvider::new(creds);
/// let token = tp.get_token().await?;
/// println!("Bearer {token}");
/// # Ok(()) }
/// ```
#[derive(Clone)]
pub struct AzureTokenProvider {
    credentials: AzureCredentials,
    scope: String,
    client: Client,
    cache: Arc<Mutex<Option<CachedToken>>>,
}

impl AzureTokenProvider {
    /// Default scope for Azure OpenAI / Cognitive Services.
    const DEFAULT_SCOPE: &'static str = "https://cognitiveservices.azure.com/.default";

    pub fn new(credentials: AzureCredentials) -> Self {
        Self {
            credentials,
            scope: Self::DEFAULT_SCOPE.to_string(),
            client: Client::new(),
            cache: Arc::new(Mutex::new(None)),
        }
    }

    /// Override the default scope.
    pub fn with_scope(mut self, scope: &str) -> Self {
        self.scope = scope.to_string();
        self
    }

    /// Return a valid bearer token, refreshing proactively.
    pub async fn get_token(&self) -> Result<String, RustCloudError> {
        let mut guard = self.cache.lock().await;
        let now = now_secs();

        if let Some(cached) = guard.as_ref() {
            if cached.expires_at > now + 60 {
                return Ok(cached.access_token.clone());
            }
        }

        let token = self.fetch_token().await?;
        let access = token.access_token.clone();
        *guard = Some(CachedToken {
            access_token: token.access_token,
            expires_at: now + token.expires_in,
        });
        Ok(access)
    }

    async fn fetch_token(&self) -> Result<TokenResponse, RustCloudError> {
        let url = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
            self.credentials.tenant_id,
        );

        let resp = self
            .client
            .post(&url)
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", &self.credentials.client_id),
                ("client_secret", &self.credentials.client_secret),
                ("scope", &self.scope),
            ])
            .send()
            .await
            .map_err(|e| RustCloudError::Http(format!("Azure token exchange: {e}")))?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(RustCloudError::Auth(format!(
                "Azure token exchange failed: {text}"
            )));
        }

        resp.json::<TokenResponse>()
            .await
            .map_err(|e| RustCloudError::Parse(format!("Azure token response: {e}")))
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before UNIX epoch")
        .as_secs()
}
