// ── src/auth/aws.rs ───────────────────────────────────────────────────────
//! AWS Signature Version 4 request signing.
//!
//! Implements the full SigV4 algorithm as specified by AWS:
//! <https://docs.aws.amazon.com/general/latest/gr/sigv4_signing.html>
//!
//! The signer constructs the canonical request, string-to-sign, and signing
//! key, then attaches the `Authorization` header (plus `X-Amz-Date` and
//! `X-Amz-Content-Sha256`) to an outgoing [`reqwest::RequestBuilder`].

use crate::error::RustCloudError;
use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

type HmacSha256 = Hmac<Sha256>;

/// AWS credentials pair.
#[derive(Debug, Clone)]
pub struct AwsCredentials {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: Option<String>,
}

impl AwsCredentials {
    /// Load from environment variables `AWS_ACCESS_KEY_ID` and
    /// `AWS_SECRET_ACCESS_KEY` (with optional `AWS_SESSION_TOKEN`).
    pub fn from_env() -> Result<Self, RustCloudError> {
        let access_key_id = std::env::var("AWS_ACCESS_KEY_ID")
            .map_err(|_| RustCloudError::Auth("AWS_ACCESS_KEY_ID not set".into()))?;
        let secret_access_key = std::env::var("AWS_SECRET_ACCESS_KEY")
            .map_err(|_| RustCloudError::Auth("AWS_SECRET_ACCESS_KEY not set".into()))?;
        let session_token = std::env::var("AWS_SESSION_TOKEN").ok();
        Ok(Self { access_key_id, secret_access_key, session_token })
    }
}

/// Stateless SigV4 signer — signs individual requests.
#[derive(Debug, Clone)]
pub struct AwsSigV4Signer {
    pub credentials: AwsCredentials,
    pub region: String,
    pub service: String,
}

impl AwsSigV4Signer {
    pub fn new(credentials: AwsCredentials, region: &str, service: &str) -> Self {
        Self {
            credentials,
            region: region.to_string(),
            service: service.to_string(),
        }
    }

    /// Sign a request and return the headers that must be added.
    ///
    /// Returns a map of header-name → header-value.
    pub fn sign_request(
        &self,
        method: &str,
        url: &url::Url,
        headers: &BTreeMap<String, String>,
        payload: &[u8],
    ) -> Result<BTreeMap<String, String>, RustCloudError> {
        let now = Utc::now();
        let date_stamp = now.format("%Y%m%d").to_string();
        let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();

        // Payload hash
        let payload_hash = hex::encode(Sha256::digest(payload));

        // Merge incoming headers + required SigV4 headers
        let mut canonical_headers = headers.clone();
        canonical_headers.insert("host".to_string(), url.host_str().unwrap_or("").to_string());
        canonical_headers.insert("x-amz-date".to_string(), amz_date.clone());
        canonical_headers.insert("x-amz-content-sha256".to_string(), payload_hash.clone());

        if let Some(ref tok) = self.credentials.session_token {
            canonical_headers.insert("x-amz-security-token".to_string(), tok.clone());
        }

        // Signed headers (sorted, semicolon-delimited)
        let signed_headers: Vec<&String> = canonical_headers.keys().collect();
        let signed_headers_str = signed_headers
            .iter()
            .map(|k| k.as_str())
            .collect::<Vec<_>>()
            .join(";");

        // Canonical headers block
        let canonical_headers_str: String = canonical_headers
            .iter()
            .map(|(k, v)| format!("{}:{}\n", k, v.trim()))
            .collect();

        // Canonical query string
        let canonical_query = url.query().unwrap_or("");

        // Canonical URI
        let canonical_uri = url.path();

        // Canonical request
        let canonical_request = format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            method,
            canonical_uri,
            canonical_query,
            canonical_headers_str,
            signed_headers_str,
            payload_hash,
        );

        let canonical_request_hash = hex::encode(Sha256::digest(canonical_request.as_bytes()));

        // Credential scope
        let credential_scope = format!(
            "{}/{}/{}/aws4_request",
            date_stamp, self.region, self.service
        );

        // String to sign
        let string_to_sign = format!(
            "AWS4-HMAC-SHA256\n{}\n{}\n{}",
            amz_date, credential_scope, canonical_request_hash
        );

        // Signing key
        let signing_key = self.derive_signing_key(&date_stamp)?;

        // Signature
        let signature = hex::encode(hmac_sha256(&signing_key, string_to_sign.as_bytes())?);

        // Authorization header
        let authorization = format!(
            "AWS4-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
            self.credentials.access_key_id,
            credential_scope,
            signed_headers_str,
            signature,
        );

        let mut out = BTreeMap::new();
        out.insert("Authorization".to_string(), authorization);
        out.insert("X-Amz-Date".to_string(), amz_date);
        out.insert("X-Amz-Content-Sha256".to_string(), payload_hash);
        if let Some(ref tok) = self.credentials.session_token {
            out.insert("X-Amz-Security-Token".to_string(), tok.clone());
        }
        Ok(out)
    }

    /// Four-step HMAC key derivation.
    fn derive_signing_key(&self, date_stamp: &str) -> Result<Vec<u8>, RustCloudError> {
        let k_date = hmac_sha256(
            format!("AWS4{}", self.credentials.secret_access_key).as_bytes(),
            date_stamp.as_bytes(),
        )?;
        let k_region = hmac_sha256(&k_date, self.region.as_bytes())?;
        let k_service = hmac_sha256(&k_region, self.service.as_bytes())?;
        hmac_sha256(&k_service, b"aws4_request")
    }
}

/// HMAC-SHA256 helper.
fn hmac_sha256(key: &[u8], data: &[u8]) -> Result<Vec<u8>, RustCloudError> {
    let mut mac = HmacSha256::new_from_slice(key)
        .map_err(|e| RustCloudError::Auth(format!("HMAC key error: {e}")))?;
    mac.update(data);
    Ok(mac.finalize().into_bytes().to_vec())
}
