use crate::services::plex::search_multi;
use crate::state::AppState;
use crate::templates::{ftp_settings, render_error, search, Error};
use tauri::State;

#[tauri::command]
pub fn ftp_settings(state: State<'_, AppState>) -> Result<String, Error> {
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
) -> Result<String, Error> {
    state
        .update(&app_handle, "ftp_host", Some(ftp_host))
        .unwrap();
    state
        .update(&app_handle, "ftp_pass", Some(ftp_pass))
        .unwrap();
    state
        .update(&app_handle, "ftp_user", Some(ftp_user))
        .unwrap();
    state
        .update(
            &app_handle,
            "ftp_movie_upload_path",
            Some(ftp_movie_upload_path),
        )
        .unwrap();

    ftp_settings::render_show(&state)
}

#[tauri::command]
pub fn the_movie_db(
    key: &str,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, Error> {
    state
        .update(&app_handle, "the_movie_db_key", Some(key.to_string()))
        .unwrap();
    let response = search_multi(&state, "Avengers");
    match response {
        Ok(resp) => resp,
        Err(e) => return render_error(&e.message),
    };
    search::render_index(&app_handle)
}
