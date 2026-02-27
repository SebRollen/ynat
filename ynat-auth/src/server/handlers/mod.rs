mod callback;
mod initiate;
mod poll;
mod refresh;

pub use callback::oauth_callback;
pub use initiate::initiate_auth;
pub use poll::poll_session;
pub use refresh::refresh_token;

use crate::server::models::HealthResponse;
use axum::Json;

pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}
