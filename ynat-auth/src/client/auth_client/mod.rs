mod models;

use crate::common::StoredToken;
pub use models::AuthClientError;
use models::*;
use reqwest::Client;
use std::time::Duration;

const POLL_INTERVAL_MS: u64 = 2000; // Poll every 2 seconds
const POLL_TIMEOUT_SECS: u64 = 300; // 5 minute timeout

pub struct ServerAuthClient {
    http_client: Client,
    server_url: String,
    device_id: String,
}

impl ServerAuthClient {
    pub fn new(server_url: String, device_id: String) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            http_client,
            server_url,
            device_id,
        }
    }

    pub async fn initiate_auth(&self) -> Result<(String, String), AuthClientError> {
        let url = format!("{}/auth/initiate", self.server_url);
        let req = InitiateRequest {
            device_id: self.device_id.clone(),
        };

        let resp = self
            .http_client
            .post(&url)
            .json(&req)
            .send()
            .await?
            .error_for_status()?
            .json::<InitiateResponse>()
            .await?;

        Ok((resp.session_id, resp.authorization_url))
    }

    pub async fn poll_session(&self, session_id: &str) -> Result<StoredToken, AuthClientError> {
        let url = format!("{}/auth/poll/{}", self.server_url, session_id);
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(POLL_TIMEOUT_SECS);

        loop {
            if start.elapsed() > timeout {
                return Err(AuthClientError::Timeout);
            }

            let resp = self
                .http_client
                .get(&url)
                .query(&[("device_id", &self.device_id)])
                .send()
                .await?
                .error_for_status()?
                .json::<PollResponse>()
                .await?;

            match resp.status {
                SessionStatus::Completed => {
                    let tokens = resp
                        .tokens
                        .ok_or(AuthClientError::ServerError("Missing tokens".into()))?;
                    return Ok(StoredToken {
                        access_token: tokens.access_token,
                        refresh_token: tokens.refresh_token,
                        expires_at: tokens.expires_at,
                    });
                }
                SessionStatus::Error(msg) => {
                    return Err(AuthClientError::OAuthError(msg));
                }
                SessionStatus::Expired => {
                    return Err(AuthClientError::SessionExpired);
                }
                SessionStatus::Pending => {
                    tokio::time::sleep(Duration::from_millis(POLL_INTERVAL_MS)).await;
                }
            }
        }
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<StoredToken, AuthClientError> {
        let url = format!("{}/auth/refresh", self.server_url);
        let req = RefreshRequest {
            refresh_token: refresh_token.to_string(),
        };

        let resp = self
            .http_client
            .post(&url)
            .json(&req)
            .send()
            .await?
            .error_for_status()?
            .json::<RefreshResponse>()
            .await?;

        Ok(StoredToken {
            access_token: resp.access_token,
            refresh_token: resp.refresh_token,
            expires_at: resp.expires_at,
        })
    }
}
