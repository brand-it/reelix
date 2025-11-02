use super::the_movie_db;
use crate::models::movie_db;
use crate::models::optical_disk_info::TvSeasonContent;
use crate::state::AppState;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

// This is the local stored location of movies not the FTP location
pub fn movies_dir() -> PathBuf {
    let home_dir = dirs::home_dir().expect("failed to find home dir");
    home_dir.join("Movies")
}

pub fn create_movie_dir(movie: &movie_db::MovieResponse) -> PathBuf {
    let dir = movies_dir().join(movie.title_year());
    let message = format!("Failed to create {}", dir.display());
    if !dir.exists() {
        fs::create_dir_all(&dir).expect(&message);
    }
    dir
}

pub fn create_season_episode_dir(content: &TvSeasonContent) -> PathBuf {
    let home_dir = dirs::home_dir().expect("failed to find home dir");
    let dir = home_dir
        .join("TV Shows")
        .join(content.tv.title_year())
        .join(format!("Season {:02}", content.season.season_number));
    let message = format!("Failed to create {}", dir.display());
    if !dir.exists() {
        fs::create_dir_all(&dir).expect(&message);
    }
    dir
}

pub fn search_multi(
    app_state: &tauri::State<'_, AppState>,
    query: &str,
) -> Result<movie_db::SearchResponse, the_movie_db::Error> {
    let api_key = &app_state.lock_the_movie_db_key().to_string();
    let language = "en-US";
    let movie_db = the_movie_db::TheMovieDb::new(api_key, language);

    movie_db.search_multi(query, 1)
}

pub fn find_movie(
    app_handle: &AppHandle,
    id: u32,
) -> Result<movie_db::MovieResponse, the_movie_db::Error> {
    let state: tauri::State<AppState> = app_handle.state::<AppState>();
    let api_key = &state.lock_the_movie_db_key().to_string();

    let language = "en-US";
    let movie_db = the_movie_db::TheMovieDb::new(api_key, language);
    movie_db.movie(id)
}

pub fn find_tv(
    app_handle: &AppHandle,
    id: u32,
) -> Result<movie_db::TvResponse, the_movie_db::Error> {
    let state: tauri::State<AppState> = app_handle.state::<AppState>();
    let api_key = &state.lock_the_movie_db_key().to_string();

    let language = "en-US";
    let movie_db = the_movie_db::TheMovieDb::new(api_key, language);
    movie_db.tv(id)
}

pub fn find_season(
    app_handle: &AppHandle,
    tv_id: u32,
    season_number: u32,
) -> Result<movie_db::SeasonResponse, the_movie_db::Error> {
    let state: tauri::State<AppState> = app_handle.state::<AppState>();
    let api_key = &state.lock_the_movie_db_key().to_string();

    let language = "en-US";
    let movie_db = the_movie_db::TheMovieDb::new(api_key, language);
    movie_db.season(tv_id, season_number)
}

pub fn get_movie_certification(
    app_handle: &AppHandle,
    movie_id: &u32,
) -> Result<Option<String>, the_movie_db::Error> {
    let state: tauri::State<AppState> = app_handle.state::<AppState>();
    let api_key = &state.lock_the_movie_db_key().to_string();

    let language = "en-US";
    let movie_db = the_movie_db::TheMovieDb::new(api_key, language);
    let release_dates = movie_db.movie_release_dates(movie_id)?;

    Ok(release_dates
        .results
        .iter()
        .find(|entry| entry.iso_3166_1 == "US")
        .and_then(|us| us.release_dates.first())
        .map(|rd| rd.certification.trim().to_string()))
}
