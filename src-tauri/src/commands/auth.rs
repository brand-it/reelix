use crate::services::reelix_manager::{self, PollError};
use crate::state::AppState;
use crate::templates::{self, auth, search};
use tauri::State;

#[tauri::command]
pub fn set_host(
    host: String,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, templates::Error> {
    let host = host.trim().trim_end_matches('/').to_string();

    if host.is_empty() {
        return auth::render_host_setup("", "Please enter a server URL.");
    }

    state.set_manager_host(Some(host.clone()));
    state.set_manager_token(None);

    if !crate::services::reelix_manager::check_health(&host) {
        let _ = state.save(&app_handle);
        return templates::auth::render_host_unreachable(&host);
    }

    if let Err(e) = state.save(&app_handle) {
        return auth::render_host_setup(&host, &format!("Failed to save: {e}"));
    }

    start_device_auth_inner(&host, &state, &app_handle)
}

#[tauri::command]
pub fn start_device_auth(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, templates::Error> {
    let host = match state.get_manager_host() {
        Some(h) => h,
        None => return auth::render_host_setup("", ""),
    };
    start_device_auth_inner(&host, &state, &app_handle)
}

fn start_device_auth_inner(
    host: &str,
    state: &State<'_, AppState>,
    app_handle: &tauri::AppHandle,
) -> Result<String, templates::Error> {
    match reelix_manager::authorize_device(host) {
        Ok(resp) => {
            // Store device_code in state so poll_auth_token can use it
            state.set_pending_device_code(Some(resp.device_code.clone()));
            if let Err(e) = state.save(app_handle) {
                return templates::render_error(&format!("Failed to save device code: {e}"));
            }
            auth::render_device_code(host, &resp.user_code, &resp.verification_uri, "")
        }
        Err(e) => auth::render_host_setup(host, &format!("Failed to connect: {}", e.message)),
    }
}

#[tauri::command]
pub fn poll_auth_token(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, templates::Error> {
    let host = match state.get_manager_host() {
        Some(h) => h,
        None => return auth::render_host_setup("", ""),
    };

    let device_code = match state.get_pending_device_code() {
        Some(c) => c,
        None => return auth::render_host_setup(&host, "Device auth session lost. Please reconnect."),
    };

    match reelix_manager::poll_token(&host, &device_code) {
        Ok(token_resp) => {
            state.set_manager_token(Some(token_resp.access_token));
            state.set_pending_device_code(None);
            if let Err(e) = state.save(&app_handle) {
                return templates::render_error(&format!("Failed to save token: {e}"));
            }
            search::render_index(&app_handle, &crate::the_movie_db::SearchResponse::default())
        }
        Err(PollError::Pending) | Err(PollError::SlowDown) => Ok(String::new()),
        Err(PollError::AccessDenied) => {
            state.set_pending_device_code(None);
            let _ = state.save(&app_handle);
            auth::render_device_code(&host, "", "", "Access was denied. Please try again.")
        }
        Err(PollError::ExpiredToken) => {
            state.set_pending_device_code(None);
            let _ = state.save(&app_handle);
            auth::render_device_code(&host, "", "", "Code expired. Please reconnect.")
        }
        Err(PollError::Http(msg)) => templates::render_error(&msg),
    }
}
