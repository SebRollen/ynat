use axum::{extract::State, Json};

use crate::server::{
    error::ServerError,
    models::{RefreshRequest, RefreshResponse},
    AppState,
};

pub async fn refresh_token(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<RefreshResponse>, ServerError> {
    tracing::debug!("Token refresh requested");

    let tokens = state
        .oauth_client
        .refresh_access_token(&req.refresh_token)
        .await?;

    tracing::info!("Token refresh successful");

    Ok(Json(RefreshResponse {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        expires_at: tokens.expires_at,
    }))
}
