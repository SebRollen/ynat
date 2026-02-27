use axum::{extract::State, Json};
use uuid::Uuid;

use crate::server::{
    error::ServerError,
    models::{InitiateRequest, InitiateResponse},
    services::OAuthClient,
    AppState,
};

pub async fn initiate_auth(
    State(state): State<AppState>,
    Json(req): Json<InitiateRequest>,
) -> Result<Json<InitiateResponse>, ServerError> {
    // Create span with device_id for all logs in this request
    let span = tracing::info_span!("initiate_auth", device_id = %req.device_id);
    let _enter = span.enter();

    // Validate device_id format (should be a valid UUID)
    Uuid::parse_str(&req.device_id).map_err(|_| {
        ServerError::BadRequest("Invalid device_id format, must be a UUID".to_string())
    })?;

    // Generate CSRF state token
    let csrf_state = OAuthClient::generate_state_token();

    // Create session
    let session_id = state
        .session_store
        .create_session(req.device_id.clone(), csrf_state.clone());

    // Build OAuth authorization URL
    let auth_url = state.oauth_client.build_authorization_url(&session_id)?;

    tracing::info!(
        session_id = %session_id,
        "Initiated auth session"
    );

    Ok(Json(InitiateResponse {
        session_id,
        authorization_url: auth_url,
    }))
}
