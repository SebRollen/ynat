pub mod config;
pub mod error;
pub mod handlers;
pub mod models;
pub mod services;

pub use config::Configuration;
pub use error::ServerError;

use services::{OAuthClient, SessionStore};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub session_store: Arc<SessionStore>,
    pub oauth_client: Arc<OAuthClient>,
}
