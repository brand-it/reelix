// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use super::helpers::save_query;
use crate::models::optical_disk_info::DiskId;
use crate::services::plex::{
    find_movie, find_season, find_tv, get_movie_certification, search_multi,
};
use crate::services::the_movie_db;
use crate::state::{get_api_key, AppState};
use crate::templates::{self, render_error};
use serde_json::json;
use tauri::State;
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_store::StoreExt;

// This is the entry point, basically it decides what to first show the user
#[tauri::command]
pub fn index(state: State<'_, AppState>) -> Result<String, templates::ApiError> {
    match search_multi(&state, &"Martian") {
        Ok(resp) => resp,
        Err(e) => return templates::the_movie_db::render_show(&state, &e.message),
    };
    templates::search::render_index(&state)
}

#[tauri::command]
pub fn open_url(
    url: &str,
    app_handle: tauri::AppHandle,
    state: State<AppState>,
) -> Result<String, templates::ApiError> {
    let response = app_handle.opener().open_url(url, None::<&str>);

    match response {
        Ok(_r) => Ok("".to_string()),
        Err(e) => render_error(&state, &format!("failed to open url: {:?}", e)),
    }
}

#[tauri::command]
pub fn movie(
    id: u32,
    app_state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, templates::ApiError> {
    let movie = match find_movie(&app_handle, id) {
        Ok(resp) => resp,
        Err(e) => return templates::the_movie_db::render_show(&app_state, &e.message),
    };

    let certification = match get_movie_certification(&app_handle, &id) {
        Ok(resp) => resp,
        Err(e) => return templates::the_movie_db::render_show(&app_state, &e.message),
    };
    templates::movies::render_show(&app_state, &movie, &certification)
}

#[tauri::command]
pub fn tv(
    id: u32,
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<String, templates::ApiError> {
    let tv = match find_tv(&app_handle, id) {
        Ok(resp) => resp,
        Err(e) => return templates::the_movie_db::render_show(&state, &e.message),
    };

    templates::tvs::render_show(&state, &tv)
}

#[tauri::command]
pub fn season(
    tv_id: u32,
    season_number: u32,
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<String, templates::ApiError> {
    let tv = match find_tv(&app_handle, tv_id) {
        Ok(resp) => resp,
        Err(e) => return templates::the_movie_db::render_show(&state, &e.message),
    };

    let season = match find_season(&app_handle, tv_id, season_number) {
        Ok(resp) => resp,
        Err(e) => return templates::the_movie_db::render_show(&state, &e.message),
    };

    templates::seasons::render_show(&state, &tv, &season)
}

#[tauri::command]
pub fn the_movie_db(
    key: &str,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, templates::ApiError> {
    let mut movie_db_key = state
        .the_movie_db_key
        .write()
        .expect("Failed to acquire lock on the_movie_db_key in the_movie_db command");
    *movie_db_key = key.to_string();
    let response = search_multi(&state, &"Avengers");
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
    templates::search::render_index(&state)
}

#[tauri::command]
pub fn search(search: &str, state: State<'_, AppState>) -> Result<String, templates::ApiError> {
    save_query(&state, search);

    let api_key = get_api_key(&state);
    let language = "en-US";
    let movie_db = the_movie_db::TheMovieDb::new(&api_key, &language);
    let response = match movie_db.search_multi(search, 1) {
        Ok(resp) => resp,
        Err(e) => return templates::the_movie_db::render_show(&state, &e.message),
    };

    templates::search::render_results(&state, &search, &response)
}

#[tauri::command]
pub fn selected_disk(
    disk_id: u32,
    state: State<'_, AppState>,
) -> Result<String, templates::ApiError> {
    match DiskId::try_from(disk_id) {
        Ok(id) => {
            let mut selected_optical_disk_id = state
                .selected_optical_disk_id
                .write()
                .expect("failed to lock selected disk ID");
            *selected_optical_disk_id = Some(id);
        }
        Err(_e) => {
            return render_error(&state, &format!("Failed to covert {} to DiskID", &disk_id))
        }
    };

    templates::disk_titles::render_options(&state)
}
