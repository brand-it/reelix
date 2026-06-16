use crate::services::reelix_manager::{self, Error as ReelixManagerError};
use crate::state::AppState;
use super::{render, Error, InlineTemplate};
use askama::Template;
use tauri::State;

const AUTH_DOM_ID: &str = "body";

#[derive(Template)]
#[template(path = "auth/host_setup.html")]
pub struct HostSetup<'a> {
    pub current_host: &'a str,
    pub error_message: &'a str,
}

impl HostSetup<'_> {
    pub fn dom_id(&self) -> &'static str {
        AUTH_DOM_ID
    }
}

#[derive(Template)]
#[template(path = "auth/host_setup.turbo.html")]
pub struct HostSetupTurbo<'a> {
    pub host_setup: &'a HostSetup<'a>,
}

#[derive(Template)]
#[template(path = "auth/device_code.html")]
pub struct DeviceCode<'a> {
    pub host: &'a str,
    pub user_code: &'a str,
    pub verification_uri: &'a str,
    pub error_message: &'a str,
}

impl DeviceCode<'_> {
    pub fn dom_id(&self) -> &'static str {
        AUTH_DOM_ID
    }
}

#[derive(Template)]
#[template(path = "auth/device_code.turbo.html")]
pub struct DeviceCodeTurbo<'a> {
    pub device_code: &'a DeviceCode<'a>,
}

#[derive(Template)]
#[template(path = "auth/host_unreachable.html")]
pub struct HostUnreachable<'a> {
    pub host: &'a str,
}

impl HostUnreachable<'_> {
    pub fn dom_id(&self) -> &'static str {
        AUTH_DOM_ID
    }
}

#[derive(Template)]
#[template(path = "auth/host_unreachable.turbo.html")]
pub struct HostUnreachableTurbo<'a> {
    pub host_unreachable: &'a HostUnreachable<'a>,
}

pub fn render_host_unreachable(host: &str) -> Result<String, Error> {
    let host_unreachable = HostUnreachable { host };
    let template = HostUnreachableTurbo { host_unreachable: &host_unreachable };
    render(template)
}

pub fn render_host_setup(current_host: &str, error_message: &str) -> Result<String, Error> {
    let host_setup = HostSetup { current_host, error_message };
    let template = HostSetupTurbo { host_setup: &host_setup };
    render(template)
}

pub fn render_device_code(
    host: &str,
    user_code: &str,
    verification_uri: &str,
    error_message: &str,
) -> Result<String, Error> {
    let device_code = DeviceCode { host, user_code, verification_uri, error_message };
    let template = DeviceCodeTurbo { device_code: &device_code };
    render(template)
}

/// Centralized error handler for Reelix Manager API errors.
///
/// Auth failures (401/422) clear the token and initiate the device
/// authorization flow. Non-auth errors (network failures, server errors,
/// etc.) show the host setup screen to let the user check the server
/// connection without clearing a potentially valid token.
pub fn render_on_error(
    error: &ReelixManagerError,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, Error> {
    let host = state.get_manager_host().unwrap_or_default();

    if error.is_auth_failure() {
        // Token is invalid — clear it and start device auth flow.
        state.set_manager_token(None);
        let _ = state.save(&app_handle);
        reelix_manager::oauth::start_device_auth_flow(&host, &state, &app_handle)
    } else {
        // Network error, server error, or other non-auth failure.
        // Preserve the token — the server may just be temporarily down.
        render_host_setup(&host, &error.message)
    }
}

/// Handle a Reelix Manager API result. On success, returns the value.
/// On error, delegates to `render_on_error` for classification-based
/// handling (auth vs non-auth errors) and returns `Err(templates::Error)`
/// with the HTML in the `template` field.
///
/// The JS `turboInvoke` catch block detects `error.template` and pipes
/// it through `processTurboResponse` instead of logging.
///
/// Usage:
/// ```ignore
/// let search = auth::response_or_error(manager.search(&query, 1), app_state, app_handle)?;
/// ```
pub fn response_or_error<T>(
    result: Result<T, ReelixManagerError>,
    state: State<'_, AppState>,
    app_handle: &tauri::AppHandle,
) -> Result<T, Error> {
    match result {
        Ok(resp) => Ok(resp),
        Err(e) => {
            let auth_html = render_on_error(&e, state, app_handle.clone())
                .unwrap_or_else(|err| err.message);
            Err(Error {
                message: format!("Reelix Manager error: {}", e.message),
                template: Some(auth_html),
            })
        }
    }
}
