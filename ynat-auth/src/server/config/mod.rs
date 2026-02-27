use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Configuration {
    pub server: ServerConfiguration,
    pub oauth: OAuthConfiguration,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfiguration {
    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default = "default_session_ttl")]
    pub session_ttl_seconds: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OAuthConfiguration {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_session_ttl() -> u64 {
    600
}

impl Configuration {
    pub fn new() -> Result<Self, config::ConfigError> {
        let mut builder = config::Config::builder();

        if std::path::Path::new("config.toml").exists() {
            builder = builder.add_source(config::File::with_name("config"));
        }

        builder = builder.add_source(config::Environment::with_prefix("YNAB_AUTH").separator("__"));

        builder.build()?.try_deserialize()
    }
}
