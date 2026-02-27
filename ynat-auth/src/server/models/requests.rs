use chrono::{serde::ts_seconds, DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::SessionStatus;
use crate::common::TokenPair;

// POST /auth/initiate
#[derive(Debug, Deserialize)]
pub struct InitiateRequest {
    pub device_id: String,
}

#[derive(Debug, Serialize)]
pub struct InitiateResponse {
    pub session_id: String,
    pub authorization_url: String,
}

// GET /auth/callback
#[derive(Debug, Deserialize)]
pub struct CallbackParams {
    pub code: Option<String>,
    pub state: String,
    pub error: Option<String>,
}

// GET /auth/poll/{session_id}
#[derive(Debug, Deserialize)]
pub struct PollParams {
    pub device_id: String,
}

#[derive(Debug, Serialize)]
pub struct PollResponse {
    pub status: SessionStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens: Option<TokenPair>,
}

// POST /auth/refresh
#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct RefreshResponse {
    pub access_token: String,
    pub refresh_token: String,
    #[serde(with = "ts_seconds")]
    pub expires_at: DateTime<Utc>,
}

// Health check
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}
