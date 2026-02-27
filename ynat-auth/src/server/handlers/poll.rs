use axum::{
    extract::{Path, Query, State},
    Json,
};

use crate::server::{
    error::ServerError,
    models::{PollParams, PollResponse, SessionStatus},
    AppState,
};

pub async fn poll_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Query(params): Query<PollParams>,
) -> Result<Json<PollResponse>, ServerError> {
    // Create span with device_id and session_id for all logs in this request
    let span = tracing::info_span!(
        "poll_session",
        device_id = %params.device_id,
        session_id = %session_id
    );
    let _enter = span.enter();

    // Validate device_id matches session
    if !state
        .session_store
        .validate_device(&session_id, &params.device_id)
    {
        tracing::warn!("Device ID mismatch for session");
        return Err(ServerError::Forbidden(
            "Device ID does not match session".to_string(),
        ));
    }

    // Get session
    let session = state
        .session_store
        .get_session(&session_id)
        .ok_or_else(|| ServerError::NotFound("Session not found or expired".to_string()))?;

    let response = match session.status {
        SessionStatus::Completed => {
            // Extract tokens
            let tokens = session.tokens.clone();

            // Delete session after successful poll (one-time retrieval)
            state.session_store.delete_session(&session_id);

            tracing::info!("Session polled successfully, tokens retrieved");

            PollResponse {
                status: SessionStatus::Completed,
                tokens,
            }
        }
        _ => PollResponse {
            status: session.status.clone(),
            tokens: None,
        },
    };

    Ok(Json(response))
}
