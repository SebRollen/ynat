use chrono::Utc;
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, HttpRequest,
    HttpResponse, RedirectUrl, RefreshToken, TokenResponse, TokenUrl,
};
use rand::Rng;

use crate::common::TokenPair;
use crate::server::config::OAuthConfiguration;
use crate::server::error::ServerError;

// Simple async HTTP client for OAuth2
async fn http_client(request: HttpRequest) -> Result<HttpResponse, reqwest::Error> {
    let client = reqwest::Client::new();
    let mut builder = client
        .request(request.method().clone(), request.uri().to_string())
        .body(request.body().clone());

    for (name, value) in request.headers() {
        builder = builder.header(name.as_str(), value.as_bytes());
    }

    let response = builder.send().await?;
    let status = response.status();
    let body = response.bytes().await?.to_vec();

    let mut http_response = HttpResponse::new(body);
    *http_response.status_mut() = status;

    Ok(http_response)
}

const YNAB_AUTH_URL: &str = "https://app.ynab.com/oauth/authorize";
const YNAB_TOKEN_URL: &str = "https://app.ynab.com/oauth/token";

pub struct OAuthClient {
    client_id: String,
    client_secret: String,
    auth_url: AuthUrl,
    token_url: TokenUrl,
    redirect_url: RedirectUrl,
}

impl OAuthClient {
    pub fn new(config: &OAuthConfiguration) -> Result<Self, ServerError> {
        let auth_url = AuthUrl::new(YNAB_AUTH_URL.to_string())
            .map_err(|e| ServerError::Configuration(format!("Invalid auth URL: {}", e)))?;

        let token_url = TokenUrl::new(YNAB_TOKEN_URL.to_string())
            .map_err(|e| ServerError::Configuration(format!("Invalid token URL: {}", e)))?;

        let redirect_url = RedirectUrl::new(config.redirect_uri.clone())
            .map_err(|e| ServerError::Configuration(format!("Invalid redirect URI: {}", e)))?;

        Ok(Self {
            client_id: config.client_id.clone(),
            client_secret: config.client_secret.clone(),
            auth_url,
            token_url,
            redirect_url,
        })
    }

    /// Build authorization URL with state parameter for CSRF protection
    pub fn build_authorization_url(&self, state: &str) -> Result<String, ServerError> {
        let csrf_token = CsrfToken::new(state.to_string());
        let (auth_url, _) = BasicClient::new(ClientId::new(self.client_id.clone()))
            .set_client_secret(ClientSecret::new(self.client_secret.clone()))
            .set_auth_uri(self.auth_url.clone())
            .set_token_uri(self.token_url.clone())
            .set_redirect_uri(self.redirect_url.clone())
            .authorize_url(|| csrf_token)
            .url();
        Ok(auth_url.to_string())
    }

    /// Exchange authorization code for access and refresh tokens
    pub async fn exchange_code_for_token(&self, code: &str) -> Result<TokenPair, ServerError> {
        let token_result = BasicClient::new(ClientId::new(self.client_id.clone()))
            .set_client_secret(ClientSecret::new(self.client_secret.clone()))
            .set_auth_uri(self.auth_url.clone())
            .set_token_uri(self.token_url.clone())
            .set_redirect_uri(self.redirect_url.clone())
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .request_async(&http_client)
            .await?;

        let access_token = token_result.access_token().secret().to_string();
        let refresh_token = token_result
            .refresh_token()
            .ok_or_else(|| ServerError::OAuthError("No refresh token in response".to_string()))?
            .secret()
            .to_string();

        let expires_in = token_result
            .expires_in()
            .ok_or_else(|| ServerError::OAuthError("No expiration time in response".to_string()))?;

        let now = Utc::now();

        let expires_at = now + expires_in;

        tracing::debug!(
            "Successfully exchanged code for tokens, expires_at: {}",
            expires_at
        );

        Ok(TokenPair {
            access_token,
            refresh_token,
            expires_at,
        })
    }

    /// Refresh an expired access token using a refresh token
    pub async fn refresh_access_token(
        &self,
        refresh_token: &str,
    ) -> Result<TokenPair, ServerError> {
        let token_result = BasicClient::new(ClientId::new(self.client_id.clone()))
            .set_client_secret(ClientSecret::new(self.client_secret.clone()))
            .set_auth_uri(self.auth_url.clone())
            .set_token_uri(self.token_url.clone())
            .set_redirect_uri(self.redirect_url.clone())
            .exchange_refresh_token(&RefreshToken::new(refresh_token.to_string()))
            .request_async(&http_client)
            .await?;

        let access_token = token_result.access_token().secret().to_string();
        let refresh_token = token_result
            .refresh_token()
            .ok_or_else(|| ServerError::OAuthError("No refresh token in response".to_string()))?
            .secret()
            .to_string();

        let expires_in = token_result
            .expires_in()
            .ok_or_else(|| ServerError::OAuthError("No expiration time in response".to_string()))?;

        let now = Utc::now();

        let expires_at = now + expires_in;

        tracing::debug!("Successfully refreshed tokens, expires_at: {}", expires_at);

        Ok(TokenPair {
            access_token,
            refresh_token,
            expires_at,
        })
    }

    /// Generate a random CSRF state token
    pub fn generate_state_token() -> String {
        use base64::Engine;
        let mut rng = rand::rng();
        let random_bytes: Vec<u8> = (0..32).map(|_| rng.random()).collect();
        base64::prelude::BASE64_STANDARD.encode(&random_bytes)
    }
}
