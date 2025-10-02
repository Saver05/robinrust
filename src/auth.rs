//! Authentication helpers for signing Robinhood API requests.
//!
//! This module loads credentials from environment variables and builds the
//! required headers (x-api-key, x-timestamp, x-signature) for each request.
//! It uses Ed25519 to sign a message composed of api key, timestamp, path,
//! method, and request body.

use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use base64::{engine::general_purpose::STANDARD as b64, Engine as _};
use ed25519_dalek::{Signer, SigningKey};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Client;

/// Robinhood API credentials and signing keys.
///
/// Use `from_env` to construct from environment variables and `auth_headers` to
/// produce the required headers for authenticated requests.
pub struct Robinhood {
    pub api_key: String,                 // <- the "rh-api-..." value
    pub signing_priv_b64: String,        // <- base64-encoded 32-byte Ed25519 private key
    pub signing_public_key: String,
}

impl Robinhood {
    /// Construct a Robinhood client by reading required environment variables.
    ///
    /// Loads a .env file if present. Panics if any required variable is missing.
    pub fn from_env() -> Self {
        dotenv::dotenv().ok();
        Self {
            api_key: env::var("ROBINHOOD_API_KEY").expect("missing ROBINHOOD_API_KEY"),
            signing_priv_b64: env::var("ROBINHOOD_SIGNING_PRIVATE_B64")
                .expect("missing ROBINHOOD_SIGNING_PRIVATE_B64"),
            signing_public_key: env::var("ROBINHOOD_PUBLIC_KEY")
                .expect("missing ROBINHOOD_PUBLIC_KEY"),
        }
    }

    /// Create a base64 Ed25519 signature and timestamp for the given request.
    ///
    /// The signed message is `api_key + timestamp + path + method + body`.
    /// Returns a tuple of (signature_base64, timestamp_seconds_string).
    fn create_signature(&self, path: &str, method: &str, body: &str) -> (String, String) {
        // decode private key to 32 bytes
        let sk_bytes_vec = b64.decode(&self.signing_priv_b64).expect("bad base64");
        let sk_bytes: [u8; 32] = sk_bytes_vec.as_slice()
            .try_into().expect("private key must be 32 bytes");
        let signing_key = SigningKey::from_bytes(&sk_bytes);

        // unix seconds timestamp
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH).expect("clock error")
            .as_secs() as i64;

        // message = api_key + timestamp + path + method + (body or "")
        let msg = format!("{}{}{}{}{}", self.api_key, ts, path, method, body);
        let sig_b64 = b64.encode(signing_key.sign(msg.as_bytes()).to_bytes());
        (sig_b64, ts.to_string())
    }

    /// Build the required authentication headers for a Robinhood request.
    ///
    /// Parameters:
    /// - `path`: The request path beginning with '/'.
    /// - `method`: HTTP verb (e.g., "GET", "POST").
    /// - `body`: The raw request body string (empty string for GETs).
    pub fn auth_headers(&self, path: &str, method: &str, body: &str) -> HeaderMap {
        let (sig, ts) = self.create_signature(path, method, body);
        let mut h = HeaderMap::new();
        h.insert(HeaderName::from_static("x-api-key"), HeaderValue::from_str(&self.api_key).unwrap());
        h.insert(HeaderName::from_static("x-timestamp"), HeaderValue::from_str(&ts).unwrap());
        h.insert(HeaderName::from_static("x-signature"), HeaderValue::from_str(&sig).unwrap());
        h
    }
}

#[tokio::test]
async fn test_auth() {
    let rh = Robinhood::from_env();
    let path = "/api/v1/crypto/trading/accounts/";
    let headers = rh.auth_headers(path, "GET", "");
    let client = Client::new();
    let resp = client
        .get(format!("https://trading.robinhood.com{path}"))
        .headers(headers)
        .send()
        .await
        .unwrap();
    println!("{:?}", resp.text().await.unwrap());
}
