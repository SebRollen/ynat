use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::common::TokenPair;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthSession {
    pub session_id: String,
    pub device_id: String,
    pub state: String,
    pub status: SessionStatus,
    pub created_at: DateTime<Utc>,
    pub tokens: Option<TokenPair>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum SessionStatus {
    Pending,
    Completed,
    Expired,
    Error(String),
}
