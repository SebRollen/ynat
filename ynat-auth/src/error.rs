use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Token storage error: {0}")]
    TokenStorage(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Auth error: {0}")]
    AuthClient(#[from] crate::client::auth_client::AuthClientError),
}

impl From<config::ConfigError> for AuthError {
    fn from(err: config::ConfigError) -> Self {
        AuthError::Configuration(err.to_string())
    }
}
