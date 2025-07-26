use crate::services::plex::search_multi;
use crate::state::AppState;
use crate::templates::{ftp_settings, render_error, search, ApiError};
use serde_json::json;
use tauri::State;
use tauri_plugin_store::StoreExt;

#[tauri::command]
pub fn ftp_settings(state: State<'_, AppState>) -> Result<String, ApiError> {
    ftp_settings::render_show(&state)
}

#[tauri::command]
pub fn update_ftp_settings(
    ftp_host: String,
    ftp_user: String,
    ftp_pass: String,
    ftp_movie_upload_path: String,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, ApiError> {
    let store = app_handle
        .store("store.json")
        .expect("Failed to load store.json for persistence in the_movie_db command");
    store.set("ftp_host", json!(ftp_host));
    store.set("ftp_pass", json!(ftp_pass));
    store.set("ftp_user", json!(ftp_user));
    store.set("ftp_movie_upload_path", json!(ftp_movie_upload_path));
    store
        .save()
        .expect("Failed to save store.json in the_movie_db command");
    state.update("ftp_host", Some(ftp_host)).unwrap();
    state.update("ftp_pass", Some(ftp_pass)).unwrap();
    state.update("ftp_user", Some(ftp_user)).unwrap();
    state
        .update("ftp_movie_upload_path", Some(ftp_movie_upload_path))
        .unwrap();

    ftp_settings::render_show(&state)
}

#[tauri::command]
pub fn the_movie_db(
    key: &str,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, ApiError> {
    state
        .update("the_movie_db_key", Some(key.to_string()))
        .unwrap();
    let response = search_multi(&state, "Avengers");
    match response {
        Ok(resp) => resp,
        Err(e) => return render_error(&state, &e.message),
    };
    let store = app_handle
        .store("store.json")
        .expect("Failed to load store.json for persistence in the_movie_db command");
    store.set("the_movie_db_key", json!(key));
    store
        .save()
        .expect("Failed to save store.json in the_movie_db command");
    search::render_index(&state)
}
