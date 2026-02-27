pub mod auth_client;
mod config;
mod device_id;
mod token_storage;

pub use auth_client::ServerAuthClient;
pub use config::Settings;
pub use device_id::DeviceIdStore;
pub use token_storage::TokenStore;

use crate::common::StoredToken;
use crate::error::AuthError;

/// Authenticate user before starting TUI
/// Returns a valid token or exits with error
pub async fn authenticate() -> Result<StoredToken, AuthError> {
    // Load configuration
    let settings = Settings::new().map_err(|e| {
        eprintln!("Failed to load configuration: {}", e);
        eprintln!("\nPlease create a config.toml file with the following content:");
        eprintln!("\n[auth]");
        eprintln!("server_url = \"http://localhost:8080\"  # For local development");
        eprintln!("# server_url = \"https://your-auth-server.com\"  # For production");
        AuthError::Configuration(e.to_string())
    })?;

    // Validate configuration
    settings.validate().map_err(|e| {
        eprintln!("Configuration validation failed: {}", e);
        AuthError::Configuration(e)
    })?;

    // Load or create device ID
    let device_id_store = DeviceIdStore::new()?;
    let device_id = device_id_store.load_or_create()?;

    // Initialize clients
    let auth_client = ServerAuthClient::new(settings.server_url.clone(), device_id);
    let token_store = TokenStore::new()?;

    // Check for existing token
    if let Some(token) = token_store.load_token()? {
        if !token_store.is_token_expired(&token) {
            // Token is valid
            return Ok(token);
        }

        // Token expired, try to refresh
        println!("Token expired, attempting to refresh...");
        match auth_client.refresh_token(&token.refresh_token).await {
            Ok(new_token) => {
                token_store.save_token(&new_token)?;
                println!("✓ Token refreshed successfully");
                return Ok(new_token);
            }
            Err(e) => {
                eprintln!("Failed to refresh token: {}", e);
                token_store.delete_token()?;
            }
        }
    }

    // No valid token - need to authenticate
    println!("\n=== YNAB Authentication Required ===\n");
    println!("This will open your browser to authorize the application.");
    println!("After authorization, please wait while we complete the process.\n");
    println!("Press Enter to start authentication, or Ctrl+C to cancel...");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    // Initiate auth on server
    let (session_id, auth_url) = auth_client.initiate_auth().await?;

    // Open browser
    if let Err(e) = open::that(&auth_url) {
        eprintln!("Failed to open browser automatically: {}", e);
        eprintln!("\nPlease open this URL in your browser:");
        eprintln!("{}\n", auth_url);
    } else {
        println!("Browser opened. Please authorize the application...");
        println!("\nYou can also open this URL directly in your browser:");
        println!("{}\n", auth_url);
    }

    // Poll for completion
    println!("Waiting for authorization...");
    let token = auth_client.poll_session(&session_id).await?;

    // Save token
    token_store.save_token(&token)?;
    println!("✓ Authentication successful!\n");

    Ok(token)
}
