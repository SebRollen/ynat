use chrono::{serde::ts_seconds, DateTime, Utc};
use serde::{Deserialize, Serialize};

// Mirror server models
#[derive(Debug, Serialize)]
pub struct InitiateRequest {
    pub device_id: String,
}

#[derive(Debug, Deserialize)]
pub struct InitiateResponse {
    pub session_id: String,
    pub authorization_url: String,
}

#[derive(Debug, Deserialize)]
pub struct PollResponse {
    pub status: SessionStatus,
    pub tokens: Option<TokenPair>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum SessionStatus {
    Pending,
    Completed,
    Expired,
    Error(String),
}

#[derive(Debug, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    #[serde(with = "ts_seconds")]
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshResponse {
    pub access_token: String,
    pub refresh_token: String,
    #[serde(with = "ts_seconds")]
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug)]
pub enum AuthClientError {
    Http(reqwest::Error),
    Timeout,
    SessionExpired,
    OAuthError(String),
    ServerError(String),
}

impl std::fmt::Display for AuthClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http(e) => write!(f, "HTTP error: {}", e),
            Self::Timeout => write!(f, "Authentication timed out after 5 minutes"),
            Self::SessionExpired => write!(f, "Session expired"),
            Self::OAuthError(msg) => write!(f, "OAuth error: {}", msg),
            Self::ServerError(msg) => write!(f, "Server error: {}", msg),
        }
    }
}

impl std::error::Error for AuthClientError {}

impl From<reqwest::Error> for AuthClientError {
    fn from(err: reqwest::Error) -> Self {
        Self::Http(err)
    }
}
