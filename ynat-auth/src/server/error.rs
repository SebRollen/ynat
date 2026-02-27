use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("OAuth error: {0}")]
    OAuthError(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Rate limited")]
    RateLimited,
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ServerError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ServerError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ServerError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            ServerError::OAuthError(msg) => (StatusCode::BAD_GATEWAY, msg),
            ServerError::Configuration(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ServerError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ServerError::RateLimited => (
                StatusCode::TOO_MANY_REQUESTS,
                "Rate limit exceeded".to_string(),
            ),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

impl
    From<
        oauth2::RequestTokenError<
            reqwest::Error,
            oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
        >,
    > for ServerError
{
    fn from(
        err: oauth2::RequestTokenError<
            reqwest::Error,
            oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
        >,
    ) -> Self {
        ServerError::OAuthError(format!("Token request failed: {}", err))
    }
}

impl From<config::ConfigError> for ServerError {
    fn from(err: config::ConfigError) -> Self {
        ServerError::Configuration(format!("Configuration error: {}", err))
    }
}
