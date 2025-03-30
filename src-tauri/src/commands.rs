use crate::models::optical_disk_info::DiskId;
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use crate::services::{makemkvcon, template, the_movie_db};
use crate::state::AppState;
use serde::Serialize;
use serde_json::json;
use tauri::State;
use tauri_plugin_shell::ShellExt;
use tauri_plugin_store::StoreExt;
use tera::Context;

#[derive(Serialize)]
struct Search {
    query: String,
    search: the_movie_db::SearchResponse,
}

// This is the entry point, basically it decides what to first show the user
#[tauri::command]
pub fn index(state: State<'_, AppState>) -> Result<String, template::ApiError> {
    let api_key: String = {
        let locked_key = state
            .the_movie_db_key
            .lock()
            .expect("Failed to acquire lock on the_movie_db_key in index command");
        locked_key.clone()
    };
    let language = "en-US".to_string();
    let movie_db = the_movie_db::TheMovieDb::new(api_key.clone(), language);
    let response = movie_db.search_multi("Martian", 1);

    match response {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Error from TMDB: {}", e.message);
            let mut context = Context::new();
            context.insert("api_key", &api_key);
            // let context = Context::from_serialize(&movie_db).expect("Failed to retrieve the value");
            return template::render(&state.tera, "the_movie_db/show.html.turbo", &context, None);
        }
    };
    let mut context = Context::new();
    context.insert("optical_disks", &state.optical_disks);

    template::render(&state.tera, "search/index.html.turbo", &context, None)
}

#[tauri::command]
pub fn open_browser(url: &str, app_handle: tauri::AppHandle) -> String {
    let shell = app_handle.shell();

    #[cfg(target_os = "macos")]
    let browser_cmd = "open";

    #[cfg(target_os = "windows")]
    let browser_cmd = "cmd /C start";

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let browser_cmd = "xdg-open";

    tauri::async_runtime::block_on(async move {
        match shell.command(browser_cmd).args([url]).output().await {
            Ok(resp) => format!("Result: {:?}", String::from_utf8(resp.stdout)),
            Err(e) => {
                eprintln!("Open URL Error: {e}");
                format!("Open URL Error: {}", e)
            }
        }
    })
}

#[tauri::command]
pub fn movie(
    id: u32,
    query: &str,
    state: State<'_, AppState>,
) -> Result<String, template::ApiError> {
    let api_key: String = {
        let locked_key = state
            .the_movie_db_key
            .lock()
            .expect("Failed to acquire lock on the_movie_db_key in movie command");
        locked_key.clone()
    };
    let language: String = "en-US".to_string();
    let movie_db = the_movie_db::TheMovieDb::new(api_key, language);
    let movie_response = movie_db.movie(id);
    let movie = match movie_response {
        Ok(resp) => resp,
        Err(e) => {
            let api_key = {
                let locked_key = state
                    .the_movie_db_key
                    .lock()
                    .expect("Failed to acquire lock on the_movie_db_key for error handling in movie command");
                locked_key.clone()
            };

            let mut context = Context::new();
            context.insert("code", "500");
            context.insert("message", &format!("Error from TMDB: {}", e.message));
            context.insert("api_key", &api_key);
            return template::render(&state.tera, "the_movie_db/show.html.turbo", &context, None);
        }
    };
    let release_dates_response = movie_db.release_dates(id);
    let release_dates = match release_dates_response {
        Ok(resp) => resp,
        Err(e) => {
            let api_key = {
                let locked_key = state
                    .the_movie_db_key
                    .lock()
                    .expect("Failed to acquire lock on the_movie_db_key for release_dates error handling in movie command");
                locked_key.clone()
            };

            let mut context = Context::new();
            context.insert("code", "500");
            context.insert("message", &format!("Error from TMDB: {}", e.message));
            context.insert("api_key", &api_key);
            return template::render(&state.tera, "the_movie_db/show.html.turbo", &context, None);
        }
    };
    let mut context = Context::new();
    let certification = release_dates
        .results
        .iter()
        .find(|entry| entry.iso_3166_1 == "US")
        .and_then(|us| us.release_dates.first())
        .map(|rd| rd.certification.trim());
    context.insert("movie", &movie);
    context.insert("query", query);
    context.insert("certification", &certification);
    context.insert("optical_disks", &state.optical_disks);
    template::render(&state.tera, "movies/show.html.turbo", &context, None)
}

#[tauri::command]
pub fn the_movie_db(
    key: &str,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, template::ApiError> {
    let mut movie_db_key: std::sync::MutexGuard<'_, String> = state
        .the_movie_db_key
        .lock()
        .expect("Failed to acquire lock on the_movie_db_key in the_movie_db command");
    *movie_db_key = key.to_string();
    let api_key = key.to_string();
    let language = "en-US".to_string();
    let movie_db = the_movie_db::TheMovieDb::new(api_key, language);
    let response = movie_db.search_multi("Avengers", 1);
    match response {
        Ok(resp) => resp,
        Err(e) => {
            let api_error = template::ApiError {
                code: 500,
                message: e.message,
                api_key: None,
            };

            let context = Context::from_serialize(&api_error)
                .expect("Failed to serialize API error context in the_movie_db command");
            return template::render(&state.tera, "error.html.turbo", &context, None);
        }
    };
    let store = app_handle
        .store("store.json")
        .expect("Failed to load store.json for persistence in the_movie_db command");
    store.set("the_movie_db_key", json!(key));
    store
        .save()
        .expect("Failed to save store.json in the_movie_db command");

    template::render(
        &state.tera,
        "search/index.html.turbo",
        &Context::new(),
        None,
    )
}

#[tauri::command]
pub fn search(search: &str, state: State<'_, AppState>) -> Result<String, template::ApiError> {
    let api_key: String = {
        let locked_key = state
            .the_movie_db_key
            .lock()
            .expect("Failed to acquire lock on the_movie_db_key in search command");
        locked_key.clone()
    };
    let language: String = "en-US".to_string();
    let movie_db = the_movie_db::TheMovieDb::new(api_key, language);
    let response = movie_db.search_multi(search, 1);
    let response = match response {
        Ok(resp) => resp,
        Err(e) => {
            let api_key = {
                let locked_key = state
                    .the_movie_db_key
                    .lock()
                    .expect("Failed to acquire lock on the_movie_db_key for error handling in search command");
                locked_key.clone()
            };

            let mut context = Context::new();
            context.insert("code", "500");
            context.insert("message", &format!("Error from TMDB: {}", e.message));
            context.insert("api_key", &api_key);
            return template::render(&state.tera, "the_movie_db/show.html.turbo", &context, None);
        }
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
pub fn rip_one(
    disk_id: &str,
    title_id: &str,
    app_state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, template::ApiError> {
    match DiskId::try_from(disk_id) {
        Ok(id) => {
            if let Some(home_dir) = dirs::home_dir() {
                let movies_dir = home_dir.join("Movies");
                let title_id = title_id.to_string();
                tauri::async_runtime::spawn(async move {
                    let _ = makemkvcon::rip_title(&app_handle, &id, &title_id, &movies_dir).await;
                });
            } else {
                eprintln!("Could not determine home directory in rip_one command.");
            }
        }
        Err(e) => {
            eprintln!("Error parsing disk_id in rip_one: {}", e);
        }
    }

    template::render(
        &app_state.tera,
        "disks/toast_progress.html.turbo",
        &Context::new(),
        None,
    )
}
