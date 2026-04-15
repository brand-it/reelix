use crate::services::reelix_manager;
use crate::state::AppState;
use crate::the_movie_db;
use std::collections::HashSet;
use tauri::{AppHandle, Manager};

pub fn find_movie(
    app_handle: &AppHandle,
    id: u32,
) -> Result<(the_movie_db::MovieResponse, bool), the_movie_db::Error> {
    let state: tauri::State<AppState> = app_handle.state::<AppState>();
    let host = state.get_manager_host().unwrap_or_default();
    let token = state.get_manager_token().unwrap_or_default();

    reelix_manager::find_movie(&host, &token, id)
        .map_err(|e| the_movie_db::Error { code: 0, message: e.message })
}

pub fn find_tv(
    app_handle: &AppHandle,
    id: u32,
) -> Result<the_movie_db::TvResponse, the_movie_db::Error> {
    let state: tauri::State<AppState> = app_handle.state::<AppState>();
    let host = state.get_manager_host().unwrap_or_default();
    let token = state.get_manager_token().unwrap_or_default();

    reelix_manager::find_tv(&host, &token, id)
        .map_err(|e| the_movie_db::Error { code: 0, message: e.message })
}

pub fn find_season(
    app_handle: &AppHandle,
    tv_id: u32,
    season_number: u32,
) -> Result<(the_movie_db::SeasonResponse, HashSet<u32>), the_movie_db::Error> {
    let state: tauri::State<AppState> = app_handle.state::<AppState>();
    let host = state.get_manager_host().unwrap_or_default();
    let token = state.get_manager_token().unwrap_or_default();

    reelix_manager::find_season(&host, &token, tv_id, season_number)
        .map_err(|e| the_movie_db::Error { code: 0, message: e.message })
}

pub fn get_movie_certification(
    _app_handle: &AppHandle,
    _movie_id: &u32,
) -> Result<Option<String>, the_movie_db::Error> {
    Ok(None)
}
