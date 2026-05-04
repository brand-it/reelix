use crate::reelix_manager::ReelixManager;
use crate::services::reelix_manager;
use crate::state::AppState;
use crate::templates::{self, auth};
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

    if !crate::services::reelix_manager::oauth::check_health(&host) {
        let _ = state.save(&app_handle);
        return templates::auth::render_host_unreachable(&host);
    }

    let manager = ReelixManager::new(&state);
    let query = state.query.lock().unwrap().to_string();

    let search = templates::auth::response_or_error(manager.search(&query, 1), state, &app_handle)?;

    templates::search::render_index(&app_handle, &search)
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
    reelix_manager::oauth::start_device_auth_flow(&host, &state, &app_handle)
}

#[tauri::command]
pub fn edit_host(state: State<'_, AppState>) -> Result<String, templates::Error> {
    let host = &state.get_manager_host().unwrap_or_default();
    let error_message = match reelix_manager::oauth::authorize_device(host) {
        Ok(_) => "",
        Err(e) => &format!("Failed to connect: {}", e.message),
    };
    templates::auth::render_host_setup(host, error_message)
}
