// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use super::helpers::save_query;
use crate::services::plex::{
    find_movie, find_season, find_tv, get_movie_certification, search_multi,
};
use crate::services::the_movie_db;
use crate::state::AppState;
use crate::templates::{self, render_error};
use tauri::State;
use tauri_plugin_opener::OpenerExt;

// This is the entry point, basically it decides what to first show the user
#[tauri::command]
pub fn index(state: State<'_, AppState>) -> Result<String, templates::ApiError> {
    match search_multi(&state, "Martian") {
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
        Err(e) => render_error(&state, &format!("failed to open url: {e:?}")),
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
pub fn search(search: &str, state: State<'_, AppState>) -> Result<String, templates::ApiError> {
    save_query(&state, search);

    let api_key = &state.lock_the_movie_db_key();
    let language = "en-US";
    let movie_db = the_movie_db::TheMovieDb::new(api_key, language);
    let response = match movie_db.search_multi(search, 1) {
        Ok(resp) => resp,
        Err(e) => return templates::the_movie_db::render_show(&state, &e.message),
    };

    templates::search::render_results(&state, search, &response)
}
