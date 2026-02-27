use chrono::{serde::ts_seconds, DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Token pair returned from OAuth flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    #[serde(with = "ts_seconds")]
    pub expires_at: DateTime<Utc>,
}

/// Stored token with expiration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredToken {
    pub access_token: String,
    pub refresh_token: String,
    #[serde(with = "ts_seconds")]
    pub expires_at: DateTime<Utc>,
}

impl From<TokenPair> for StoredToken {
    fn from(tokens: TokenPair) -> Self {
        Self {
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            expires_at: tokens.expires_at,
        }
    }
}
