use super::the_movie_db;
use crate::models::movie_db;
use crate::models::optical_disk_info::TvSeasonContent;
use crate::state::{get_api_key, AppState};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub fn create_movie_dir(movie: &movie_db::MovieResponse) -> PathBuf {
    let home_dir = dirs::home_dir().expect("failed to find home dir");
    let dir = home_dir.join("Movies").join(movie.title_year());
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

pub fn find_movie(
    app_handle: &AppHandle,
    id: u32,
) -> Result<movie_db::MovieResponse, the_movie_db::Error> {
    let state: tauri::State<AppState> = app_handle.state::<AppState>();
    let api_key = get_api_key(&state);

    let language = "en-US";
    let movie_db = the_movie_db::TheMovieDb::new(&api_key, &language);
    movie_db.movie(id)
}

pub fn find_tv(
    app_handle: &AppHandle,
    id: u32,
) -> Result<movie_db::TvResponse, the_movie_db::Error> {
    let state: tauri::State<AppState> = app_handle.state::<AppState>();
    let api_key = get_api_key(&state);

    let language = "en-US";
    let movie_db = the_movie_db::TheMovieDb::new(&api_key, &language);
    movie_db.tv(id)
}

pub fn find_season(
    app_handle: &AppHandle,
    tv_id: u32,
    season_number: u32,
) -> Result<movie_db::SeasonResponse, the_movie_db::Error> {
    let state: tauri::State<AppState> = app_handle.state::<AppState>();
    let api_key = get_api_key(&state);

    let language = "en-US";
    let movie_db = the_movie_db::TheMovieDb::new(&api_key, &language);
    movie_db.season(tv_id, season_number)
}

pub fn get_movie_certification(
    app_handle: &AppHandle,
    movie_id: &u32,
) -> Result<Option<String>, the_movie_db::Error> {
    let state: tauri::State<AppState> = app_handle.state::<AppState>();
    let api_key = get_api_key(&state);

    let language = "en-US";
    let movie_db = the_movie_db::TheMovieDb::new(&api_key, &language);
    let release_dates = match movie_db.movie_release_dates(movie_id) {
        Ok(resp) => resp,
        Err(e) => return Err(e),
    };

    Ok(release_dates
        .results
        .iter()
        .find(|entry| entry.iso_3166_1 == "US")
        .and_then(|us| us.release_dates.first())
        .map(|rd| rd.certification.trim().to_string()))
}
