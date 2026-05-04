//! OAuth2 device flow authentication methods for Reelix Manager.
//!
//! These methods do not require an existing token - they are used to
//! obtain the initial access token through the OAuth2 device flow.

use crate::state::AppState;
use crate::templates;
use serde::Deserialize;
use tauri_plugin_http::reqwest::blocking::Client;

use tauri::{Emitter, Manager};
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
            let generation = state.next_device_auth_generation();

            state.set_pending_device_code(Some(resp.device_code.clone()));
            if let Err(e) = state.save(app_handle) {
                return crate::templates::render_error(&format!("Failed to save device code: {e}"));
            }

            let turbo = crate::templates::auth::render_device_code(
                host,
                &resp.user_code,
                &resp.verification_uri,
                "",
            )?;

            spawn_token_poller(
                generation,
                resp.device_code,
                resp.interval,
                resp.expires_in,
                app_handle.clone(),
            );

            Ok(turbo)
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

/// Async version of `poll_token` for use in spawned background tasks.
pub async fn poll_token_async(host: &str, device_code: &str) -> Result<TokenResponse, PollError> {
    let url = format!("{host}/oauth/token");
    let body = serde_json::json!({
        "client_id": CLIENT_ID,
        "grant_type": "urn:ietf:params:oauth:grant-type:device_code",
        "device_code": device_code,
    });

    let client = tauri_plugin_http::reqwest::Client::new();
    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| PollError::Http(format!("Request failed: {e}")))?;

    let status = resp.status();

    if status.is_success() {
        return resp
            .json::<TokenResponse>()
            .await
            .map_err(|e| PollError::Http(format!("Failed to parse token: {e}")));
    }

    let oauth_error: OAuthError = resp
        .json()
        .await
        .map_err(|e| PollError::Http(format!("Failed to parse error response: {e}")))?;

    match oauth_error.error.as_str() {
        "authorization_pending" => Err(PollError::Pending),
        "slow_down" => Err(PollError::SlowDown),
        "access_denied" => Err(PollError::AccessDenied),
        "expired_token" => Err(PollError::ExpiredToken),
        other => Err(PollError::Http(format!("Unexpected error: {other}"))),
    }
}

/// Spawn a background task to poll for the OAuth token.
///
/// The poller uses a generation counter for cancellation: if a new auth
/// flow starts before this one completes, the poller exits silently.
pub fn spawn_token_poller(
    generation: u32,
    device_code: String,
    interval: u32,
    expires_in: u32,
    app_handle: tauri::AppHandle,
) {
    tauri::async_runtime::spawn(async move {
        log::info!(
            "Token poller started: interval {interval}s, expires_in {expires_in}s"
        );
        // RFC 8628: start with server-provided interval, clamp to >= 1s
        let mut current_interval =
            std::time::Duration::from_secs(interval.max(1) as u64);
        let deadline =
            std::time::Instant::now() + std::time::Duration::from_secs(expires_in as u64);
        let mut http_retries = 0;
        let mut poll_count = 0;

        loop {
            // Cancel check - a new auth flow started
            {
                let state = app_handle.state::<AppState>();
                if state.current_device_auth_generation() != generation {
                    log::info!("Token poller cancelled (new auth flow started)");
                    return;
                }
            }

            // Timeout check
            {
                let state = app_handle.state::<AppState>();
                if std::time::Instant::now() >= deadline {
                    log::warn!("Token poller: device code expired");
                    state.set_pending_device_code(None);
                    let _ = state.save(&app_handle);
                    match crate::templates::auth::render_device_code(
                        &state.get_manager_host().unwrap_or_default(),
                        "",
                        "",
                        "Code expired. Please reconnect.",
                    ) {
                        Ok(turbo) => {
                            if let Err(e) = app_handle.emit("disks-changed", turbo) {
                                log::error!("Token poller: failed to emit expired state: {e}");
                            }
                        }
                        Err(e) => {
                            log::error!("Token poller: failed to render expired state: {e}");
                        }
                    }
                    return;
                }
            }

            let host: String = {
                let state = app_handle.state::<AppState>();
                match state.get_manager_host() {
                    Some(h) => h,
                    None => {
                        log::warn!("Token poller: no manager host configured");
                        return;
                    }
                }
            };

            poll_count += 1;
            log::info!("Token poller: poll #{poll_count}, interval {current_interval:?}");

            match poll_token_async(&host, &device_code).await {
                Ok(token_resp) => {
                    log::info!("Token poller: token received, authenticated");
                    {
                        let state = app_handle.state::<AppState>();
                        state.set_manager_token(Some(token_resp.access_token));
                        state.set_pending_device_code(None);
                        match state.save(&app_handle) {
                            Ok(()) => log::info!("Token poller: state saved successfully"),
                            Err(e) => log::error!("Token poller: failed to save state: {e}"),
                        }
                    }
                    match crate::templates::search::render_index(
                        &app_handle,
                        &crate::reelix_manager::SearchResponse::default(),
                    ) {
                        Ok(turbo) => {
                            match app_handle.emit("disks-changed", turbo) {
                                Ok(()) => log::info!("Token poller: emitted disks-changed event"),
                                Err(e) => log::error!("Token poller: failed to emit disks-changed: {e}"),
                            }
                        }
                        Err(e) => log::error!("Token poller: failed to render search index: {e}"),
                    }
                    return;
                }
                Err(PollError::Pending) => {
                    http_retries = 0;
                    tokio::time::sleep(current_interval).await;
                }
                Err(PollError::SlowDown) => {
                    http_retries = 0;
                    current_interval *= 2; // RFC 8628 backoff
                    log::info!("Token poller: server requested slow-down, doubling interval to {current_interval:?}");
                    tokio::time::sleep(current_interval).await;
                }
                Err(PollError::AccessDenied) => {
                    log::warn!("Token poller: access denied by user");
                    {
                        let state = app_handle.state::<AppState>();
                        state.set_pending_device_code(None);
                        let _ = state.save(&app_handle);
                    }
                    match crate::templates::auth::render_device_code(
                        &host,
                        "",
                        "",
                        "Access was denied. Please try again.",
                    ) {
                        Ok(turbo) => {
                            if let Err(e) = app_handle.emit("disks-changed", turbo) {
                                log::error!("Token poller: failed to emit access denied state: {e}");
                            }
                        }
                        Err(e) => {
                            log::error!("Token poller: failed to render access denied state: {e}");
                        }
                    }
                    return;
                }
                Err(PollError::ExpiredToken) => {
                    log::warn!("Token poller: device code expired (server-side)");
                    {
                        let state = app_handle.state::<AppState>();
                        state.set_pending_device_code(None);
                        let _ = state.save(&app_handle);
                    }
                    match crate::templates::auth::render_device_code(
                        &host,
                        "",
                        "",
                        "Code expired. Please reconnect.",
                    ) {
                        Ok(turbo) => {
                            if let Err(e) = app_handle.emit("disks-changed", turbo) {
                                log::error!("Token poller: failed to emit expired state: {e}");
                            }
                        }
                        Err(e) => {
                            log::error!("Token poller: failed to render expired state: {e}");
                        }
                    }
                    return;
                }
                Err(PollError::Http(msg)) => {
                    log::warn!("Token poller: HTTP error: {msg} (attempt {} of 3)", http_retries + 1);
                    if http_retries >= 3 {
                        log::error!("Token poller: giving up after {http_retries} HTTP errors: {msg}");
                        {
                            let state = app_handle.state::<AppState>();
                            state.set_pending_device_code(None);
                            let _ = state.save(&app_handle);
                        }
                        match crate::templates::auth::render_device_code(
                            &host,
                            "",
                            "",
                            &format!("Connection error: {msg}"),
                        ) {
                            Ok(turbo) => {
                                if let Err(e) = app_handle.emit("disks-changed", turbo) {
                                    log::error!("Token poller: failed to emit connection error state: {e}");
                                }
                            }
                            Err(e) => {
                                log::error!("Token poller: failed to render connection error state: {e}");
                            }
                        }
                        return;
                    }
                    http_retries += 1;
                    let backoff = std::time::Duration::from_secs(2u64.pow(http_retries));
                    tokio::time::sleep(backoff).await;
                }
            }
        }
    });
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
