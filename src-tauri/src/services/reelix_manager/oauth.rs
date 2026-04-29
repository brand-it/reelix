//! OAuth2 device flow authentication methods for Reelix Manager.
//!
//! These methods do not require an existing token - they are used to
//! obtain the initial access token through the OAuth2 device flow.

use crate::state::AppState;
use crate::templates;
use serde::Deserialize;
use tauri_plugin_http::reqwest::blocking::Client;

use super::error::{CLIENT_ID, SCOPE};
use super::{DeviceCodeResponse, Error, PollError, TokenResponse};

/// Internal OAuth error response from the server
#[derive(Debug, Deserialize)]
struct OAuthError {
    error: String,
}

/// Start the OAuth2 device authorization flow.
///
/// This performs the authorize_device call, stores the device code in state,
/// saves state, and renders the device code UI.
///
/// Shared between `commands::auth` and `templates::auth::render_on_error`
/// to avoid circular module dependencies.
pub fn start_device_auth_flow(
    host: &str,
    state: &tauri::State<'_, AppState>,
    app_handle: &tauri::AppHandle,
) -> Result<String, templates::Error> {
    match authorize_device(host) {
        Ok(resp) => {
            state.set_pending_device_code(Some(resp.device_code.clone()));
            if let Err(e) = state.save(app_handle) {
                return crate::templates::render_error(&format!(
                    "Failed to save device code: {e}"
                ));
            }
            crate::templates::auth::render_device_code(
                host,
                &resp.user_code,
                &resp.verification_uri,
                "",
            )
        }
        Err(e) => crate::templates::auth::render_host_setup(
            host,
            &format!("Failed to connect: {}", e.message),
        ),
    }
}

/// Authorize a device for OAuth2 device flow
///
/// This is the first step of authentication. It returns a device code
/// and user code that the user must enter on a separate device.
///
/// API: POST /oauth/authorize_device
pub fn authorize_device(host: &str) -> Result<DeviceCodeResponse, Error> {
    let url = format!("{host}/oauth/authorize_device");
    let body = serde_json::json!({
        "client_id": CLIENT_ID,
        "scope": SCOPE,
    });

    let client = Client::new();
    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .map_err(|e| Error::new(format!("Request failed: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        return Err(Error::new(format!("Server error: {status}")));
    }

    resp.json::<DeviceCodeResponse>()
        .map_err(|e| Error::new(format!("Failed to parse device code response: {e}")))
}

/// Poll for a token after device authorization
///
/// This should be called repeatedly at the interval specified in the
/// DeviceCodeResponse until a token is received or an error occurs.
///
/// API: POST /oauth/token
pub fn poll_token(host: &str, device_code: &str) -> Result<TokenResponse, PollError> {
    let url = format!("{host}/oauth/token");
    let body = serde_json::json!({
        "client_id": CLIENT_ID,
        "grant_type": "urn:ietf:params:oauth:grant-type:device_code",
        "device_code": device_code,
    });

    let client = Client::new();
    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .map_err(|e| PollError::Http(format!("Request failed: {e}")))?;

    let status = resp.status();

    if status.is_success() {
        return resp
            .json::<TokenResponse>()
            .map_err(|e| PollError::Http(format!("Failed to parse token: {e}")));
    }

    let oauth_error: OAuthError = resp
        .json()
        .map_err(|e| PollError::Http(format!("Failed to parse error response: {e}")))?;

    match oauth_error.error.as_str() {
        "authorization_pending" => Err(PollError::Pending),
        "slow_down" => Err(PollError::SlowDown),
        "access_denied" => Err(PollError::AccessDenied),
        "expired_token" => Err(PollError::ExpiredToken),
        other => Err(PollError::Http(format!("Unexpected error: {other}"))),
    }
}

/// Check if the Reelix Manager server is healthy
///
/// API: GET /up
pub fn check_health(host: &str) -> bool {
    let url = format!("{host}/up");
    let client = Client::new();
    client
        .get(&url)
        .send()
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}
