use crate::common::StoredToken;
use crate::error::AuthError;
use chrono::{Duration, Utc};
use std::fs;
use std::path::PathBuf;

const EXPIRY_BUFFER: Duration = Duration::minutes(5);

pub struct TokenStore {
    token_path: PathBuf,
}

impl TokenStore {
    pub fn new() -> Result<Self, AuthError> {
        let cache_dir = Self::get_cache_dir()?;
        let token_path = cache_dir.join("token.json");

        // Create cache directory if it doesn't exist
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir).map_err(|e| {
                AuthError::TokenStorage(format!("Failed to create cache directory: {}", e))
            })?;
        }

        Ok(Self { token_path })
    }

    fn get_cache_dir() -> Result<PathBuf, AuthError> {
        let cache_dir = dirs::cache_dir().expect("Always returns").join("ynat");
        Ok(cache_dir)
    }

    pub fn save_token(&self, token: &StoredToken) -> Result<(), AuthError> {
        let json = serde_json::to_string_pretty(token)?;

        // Write token to file
        fs::write(&self.token_path, json)
            .map_err(|e| AuthError::TokenStorage(format!("Failed to save token: {}", e)))?;

        // Set permissions to 0600 (read/write for owner only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&self.token_path)
                .map_err(|e| {
                    AuthError::TokenStorage(format!("Failed to get file permissions: {}", e))
                })?
                .permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&self.token_path, perms).map_err(|e| {
                AuthError::TokenStorage(format!("Failed to set file permissions: {}", e))
            })?;
        }

        Ok(())
    }

    pub fn load_token(&self) -> Result<Option<StoredToken>, AuthError> {
        if !self.token_path.exists() {
            return Ok(None);
        }

        let json = fs::read_to_string(&self.token_path)
            .map_err(|e| AuthError::TokenStorage(format!("Failed to read token: {}", e)))?;

        let token: StoredToken = serde_json::from_str(&json)?;
        Ok(Some(token))
    }

    pub fn delete_token(&self) -> Result<(), AuthError> {
        if self.token_path.exists() {
            fs::remove_file(&self.token_path)
                .map_err(|e| AuthError::TokenStorage(format!("Failed to delete token: {}", e)))?;
        }
        Ok(())
    }

    pub fn is_token_expired(&self, token: &StoredToken) -> bool {
        let now = Utc::now();

        // Add buffer to expire tokens 5 minutes early
        token.expires_at <= (now + EXPIRY_BUFFER)
    }
}
