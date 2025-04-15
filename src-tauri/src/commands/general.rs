// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use super::helpers::{
    get_movie_certification, get_query, render_error, render_search_index, render_tmdb_error,
    save_query,
};
use crate::models::movie_db;
use crate::models::optical_disk_info::DiskId;
use crate::services::{template, the_movie_db};
use crate::state::{get_api_key, AppState};
use serde::Serialize;
use serde_json::json;
use tauri::State;
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_store::StoreExt;
use tera::Context;

#[derive(Serialize)]
struct Search {
    query: String,
    search: movie_db::SearchResponse,
}

// This is the entry point, basically it decides what to first show the user
#[tauri::command]
pub fn index(state: State<'_, AppState>) -> Result<String, template::ApiError> {
    let api_key = get_api_key(&state);
    let language = "en-US";
    let movie_db = the_movie_db::TheMovieDb::new(&api_key, &language);
    let response = movie_db.search_multi("Martian", 1);

    match response {
        Ok(resp) => resp,
        Err(e) => return render_tmdb_error(&state, &e.message),
    };
    render_search_index(&state)
}

#[tauri::command]
pub fn open_url(
    url: &str,
    app_handle: tauri::AppHandle,
    state: State<AppState>,
) -> Result<String, template::ApiError> {
    let response = app_handle.opener().open_url(url, None::<&str>);

    match response {
        Ok(_r) => Ok("".to_string()),
        Err(e) => render_error(&state, &format!("failed to open url: {:?}", e)),
    }
}

#[tauri::command]
pub fn movie(id: u32, state: State<'_, AppState>) -> Result<String, template::ApiError> {
    let api_key = get_api_key(&state);
    let language = "en-US";
    let movie_db = the_movie_db::TheMovieDb::new(&api_key, &language);
    let query = get_query(&state);

    let movie = match movie_db.movie(id) {
        Ok(resp) => resp,
        Err(e) => return render_tmdb_error(&state, &e.message),
    };

    let certification = match get_movie_certification(movie_db, id) {
        Ok(resp) => resp,
        Err(e) => return render_tmdb_error(&state, &e.message),
    };
    let mut context = Context::new();

    context.insert("movie", &movie_db::MovieView::from(movie));
    context.insert("query", &query);
    context.insert("certification", &certification);
    context.insert("selected_disk", &state.selected_disk());
    template::render(&state.tera, "movies/show.html.turbo", &context, None)
}

#[tauri::command]
pub fn tv(id: u32, state: State<'_, AppState>) -> Result<String, template::ApiError> {
    let api_key = get_api_key(&state);
    let language = "en-US";
    let movie_db = the_movie_db::TheMovieDb::new(&api_key, &language);
    let query: String = get_query(&state);

    let tv = match movie_db.tv(id) {
        Ok(resp) => resp,
        Err(e) => return render_tmdb_error(&state, &e.message),
    };

    let mut context = Context::new();
    context.insert("tv", &movie_db::TvView::from(tv));
    context.insert("query", &query);

    template::render(&state.tera, "tvs/show.html.turbo", &context, None)
}

#[tauri::command]
pub fn season(
    tv_id: u32,
    season_number: u32,
    state: State<'_, AppState>,
) -> Result<String, template::ApiError> {
    let api_key = get_api_key(&state);
    let language = "en-US";
    let movie_db = the_movie_db::TheMovieDb::new(&api_key, &language);

    let tv = match movie_db.tv(tv_id) {
        Ok(resp) => resp,
        Err(e) => return render_tmdb_error(&state, &e.message),
    };

    let season = match movie_db.season(tv_id, season_number) {
        Ok(resp) => resp,
        Err(e) => return render_tmdb_error(&state, &e.message),
    };

    let mut context = Context::new();
    context.insert("tv", &movie_db::TvView::from(tv));
    context.insert("season", &movie_db::SeasonView::from(season));
    context.insert("selected_disk", &state.selected_disk());

    template::render(&state.tera, "seasons/show.html.turbo", &context, None)
}

#[tauri::command]
pub fn the_movie_db(
    key: &str,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, template::ApiError> {
    let mut movie_db_key = state
        .the_movie_db_key
        .write()
        .expect("Failed to acquire lock on the_movie_db_key in the_movie_db command");
    *movie_db_key = key.to_string();
    let api_key = key.to_string();
    let language = "en-US";
    let movie_db = the_movie_db::TheMovieDb::new(&api_key, &language);
    let response = movie_db.search_multi("Avengers", 1);
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
    render_search_index(&state)
}

#[tauri::command]
pub fn search(search: &str, state: State<'_, AppState>) -> Result<String, template::ApiError> {
    save_query(&state, search);

    let api_key = get_api_key(&state);
    let language = "en-US";
    let movie_db = the_movie_db::TheMovieDb::new(&api_key, &language);
    let response = match movie_db.search_multi(search, 1) {
        Ok(resp) => resp,
        Err(e) => return render_tmdb_error(&state, &e.message),
    };

    let search = Search {
        query: search.to_string(),
        search: response,
    };

    let context = Context::from_serialize(&search)
        .expect("Failed to serialize search context in search command");

    template::render(&state.tera, "search/results.html.turbo", &context, None)
}

#[tauri::command]
pub fn selected_disk(
    disk_id: u32,
    state: State<'_, AppState>,
) -> Result<String, template::ApiError> {
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
    }

    let mut context = Context::new();
    context.insert("selected_disk", &state.selected_disk());

    template::render(
        &state.tera,
        "disk_titles/options.html.turbo",
        &context,
        None,
    )
}
