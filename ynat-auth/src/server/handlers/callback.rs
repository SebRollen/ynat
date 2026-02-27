use axum::{
    extract::{Query, State},
    response::Html,
};

use crate::server::{error::ServerError, models::CallbackParams, AppState};

const SUCCESS_HTML: &str = r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Authentication Successful</title>
    <style>
        body {
            margin: 0;
            padding: 0;
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
        }
        .container {
            background: white;
            border-radius: 12px;
            padding: 48px;
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
            text-align: center;
            max-width: 400px;
        }
        .checkmark {
            width: 64px;
            height: 64px;
            border-radius: 50%;
            background: #10B981;
            color: white;
            display: inline-flex;
            align-items: center;
            justify-content: center;
            font-size: 32px;
            margin-bottom: 24px;
        }
        h1 {
            color: #1F2937;
            margin: 0 0 12px 0;
            font-size: 24px;
            font-weight: 600;
        }
        p {
            color: #6B7280;
            margin: 0 0 24px 0;
            line-height: 1.5;
        }
        .footer {
            color: #9CA3AF;
            font-size: 14px;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="checkmark">✓</div>
        <h1>Authentication Successful!</h1>
        <p>You have successfully authorized YNAT. You can now close this window and return to your terminal.</p>
        <div class="footer">Powered by YNAT</div>
    </div>
</body>
</html>"#;

const ERROR_HTML_TEMPLATE: &str = r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Authentication Error</title>
    <style>
        body {
            margin: 0;
            padding: 0;
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
            background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
        }
        .container {
            background: white;
            border-radius: 12px;
            padding: 48px;
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
            text-align: center;
            max-width: 400px;
        }
        .error-icon {
            width: 64px;
            height: 64px;
            border-radius: 50%;
            background: #EF4444;
            color: white;
            display: inline-flex;
            align-items: center;
            justify-content: center;
            font-size: 32px;
            margin-bottom: 24px;
        }
        h1 {
            color: #1F2937;
            margin: 0 0 12px 0;
            font-size: 24px;
            font-weight: 600;
        }
        p {
            color: #6B7280;
            margin: 0 0 24px 0;
            line-height: 1.5;
        }
        .error-details {
            background: #FEE2E2;
            border-radius: 8px;
            padding: 16px;
            color: #991B1B;
            font-family: monospace;
            font-size: 14px;
            margin-bottom: 24px;
        }
        .footer {
            color: #9CA3AF;
            font-size: 14px;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="error-icon">✗</div>
        <h1>Authentication Failed</h1>
        <p>There was an error during the authorization process.</p>
        <div class="error-details">{ERROR}</div>
        <p>Please close this window and try again from your terminal.</p>
        <div class="footer">Powered by YNAT</div>
    </div>
</body>
</html>"#;

pub async fn oauth_callback(
    State(state): State<AppState>,
    Query(params): Query<CallbackParams>,
) -> Result<Html<String>, ServerError> {
    // The state parameter is the session_id
    let session_id = &params.state;

    // Validate session exists and get device_id
    let session = state
        .session_store
        .get_session(session_id)
        .ok_or_else(|| ServerError::NotFound("Invalid or expired session".to_string()))?;

    // Create span with device_id and session_id for all logs in this request
    let span = tracing::info_span!(
        "oauth_callback",
        device_id = %session.device_id,
        session_id = %session_id
    );
    let _enter = span.enter();

    // Check for OAuth errors
    if let Some(error) = params.error {
        state.session_store.error_session(session_id, error.clone());

        tracing::warn!(error = %error, "OAuth callback error");

        return Ok(Html(ERROR_HTML_TEMPLATE.replace("{ERROR}", &error)));
    }

    // Get authorization code
    let code = params
        .code
        .ok_or_else(|| ServerError::BadRequest("Missing authorization code".to_string()))?;

    // Exchange code for tokens
    let tokens = state.oauth_client.exchange_code_for_token(&code).await?;

    // Store tokens in session
    state.session_store.complete_session(session_id, tokens);

    tracing::info!("OAuth callback successful");

    Ok(Html(SUCCESS_HTML.to_string()))
}
