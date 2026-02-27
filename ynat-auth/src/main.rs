use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use ynat_auth::server::{
    config::Configuration,
    handlers,
    services::{OAuthClient, SessionStore},
    AppState,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing with structured logging
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(false)
                .with_file(false)
                .with_line_number(false),
        )
        .init();

    // Load configuration
    let configuration = Configuration::new()?;
    tracing::info!("Configuration loaded successfully");

    // Initialize services
    let session_store = Arc::new(SessionStore::new(configuration.server.session_ttl_seconds));
    let oauth_client = Arc::new(OAuthClient::new(&configuration.oauth)?);

    let app_state = AppState {
        session_store,
        oauth_client,
    };

    // Build router
    let app = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/auth/initiate", post(handlers::initiate_auth))
        .route("/auth/callback", get(handlers::oauth_callback))
        .route("/auth/poll/{session_id}", get(handlers::poll_session))
        .route("/auth/refresh", post(handlers::refresh_token))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    // Start server
    let addr = format!(
        "{}:{}",
        configuration.server.host, configuration.server.port
    );
    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
