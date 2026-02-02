use crate::state::AppState;
use crate::the_movie_db;
use tauri::{AppHandle, Manager};

pub fn search_multi(
    app_state: &tauri::State<'_, AppState>,
    query: &str,
) -> Result<the_movie_db::SearchResponse, the_movie_db::Error> {
    let api_key = &app_state.lock_the_movie_db_key().to_string();
    let language = "en-US";
    let movie_db = the_movie_db::TheMovieDb::new(api_key, language);

    movie_db.search_multi(query, 1)
}

pub fn find_movie(
    app_handle: &AppHandle,
    id: u32,
) -> Result<the_movie_db::MovieResponse, the_movie_db::Error> {
    let state: tauri::State<AppState> = app_handle.state::<AppState>();
    let api_key = &state.lock_the_movie_db_key().to_string();

    let language = "en-US";
    let movie_db = the_movie_db::TheMovieDb::new(api_key, language);
    movie_db.movie(id)
}

pub fn find_tv(
    app_handle: &AppHandle,
    id: u32,
) -> Result<the_movie_db::TvResponse, the_movie_db::Error> {
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
) -> Result<the_movie_db::SeasonResponse, the_movie_db::Error> {
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
