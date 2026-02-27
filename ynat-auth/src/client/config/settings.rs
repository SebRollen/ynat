use config::{Config, ConfigError, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    #[serde(default = "default_server_url")]
    pub server_url: String,
}

fn default_server_url() -> String {
    "https://ynat-auth-server.fly.dev".to_string()
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let config_path =
            std::env::var("YNAB_TUI_CONFIG").unwrap_or_else(|_| "config.toml".to_string());

        let settings = Config::builder()
            .add_source(File::with_name(&config_path).required(false))
            .add_source(config::Environment::with_prefix("YNAB_TUI").separator("__"))
            .build()?;

        settings.try_deserialize()
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.server_url.is_empty() {
            return Err("auth.server_url is required".to_string());
        }
        if !self.server_url.starts_with("http") {
            return Err("auth.server_url must be a valid HTTP(S) URL".to_string());
        }
        Ok(())
    }
}
