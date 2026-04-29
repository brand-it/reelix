use crate::services::reelix_manager::ReelixManager;
use crate::state::AppState;
use crate::reelix_manager::{MovieResponse, SeasonResponse, TvResponse};
use std::collections::HashSet;
use tauri::{AppHandle, Manager};

pub fn find_movie(
    app_handle: &AppHandle,
    id: u32,
) -> Result<(MovieResponse, bool), crate::services::reelix_manager::Error> {
    let state: tauri::State<AppState> = app_handle.state::<AppState>();
    let manager = ReelixManager::new(&state);
    manager.find_movie(id)
}

pub fn find_tv(
    app_handle: &AppHandle,
    id: u32,
) -> Result<TvResponse, crate::services::reelix_manager::Error> {
    let state: tauri::State<AppState> = app_handle.state::<AppState>();
    let manager = ReelixManager::new(&state);
    manager.find_tv(id)
}

pub fn find_season(
    app_handle: &AppHandle,
    tv_id: u32,
    season_number: u32,
) -> Result<(SeasonResponse, HashSet<u32>), crate::services::reelix_manager::Error> {
    let state: tauri::State<AppState> = app_handle.state::<AppState>();
    let manager = ReelixManager::new(&state);
    manager.find_season(tv_id, season_number)
}

pub fn get_movie_certification(
    _app_handle: &AppHandle,
    _movie_id: &u32,
) -> Result<Option<String>, crate::services::reelix_manager::Error> {
    Ok(None)
}
