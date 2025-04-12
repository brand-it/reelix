// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use crate::models::movie_db;
use crate::models::optical_disk_info::DiskId;
use crate::services::converter::cast_to_u32;
use crate::services::plex::{create_dir, find_movie, rename_file};
use crate::services::the_movie_db::Error;
use crate::services::{makemkvcon, template, the_movie_db};
use crate::state::{get_api_key, AppState};
use serde::Serialize;
use serde_json::json;
use tauri::State;
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_shell::ShellExt;
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
    context.insert("optical_disks", &state.optical_disks);
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
    context.insert("optical_disks", &state.optical_disks);

    template::render(&state.tera, "tvs/show.html.turbo", &context, None)
}

#[tauri::command]
pub fn season(
    tv_id: u32,
    season_id: u32,
    state: State<'_, AppState>,
) -> Result<String, template::ApiError> {
    let api_key = get_api_key(&state);
    let language = "en-US";
    let movie_db = the_movie_db::TheMovieDb::new(&api_key, &language);

    let tv = match movie_db.tv(tv_id) {
        Ok(resp) => resp,
        Err(e) => return render_tmdb_error(&state, &e.message),
    };

    let season = match tv.find_season(season_id) {
        Some(resp) => resp,
        None => {
            return render_error(
                &state,
                &format!("Failed to find Seasons {} in {}", season_id, tv.name),
            )
        }
    };

    let mut context = Context::new();
    context.insert("tv", &movie_db::TvView::from(tv));
    context.insert("season", &season);
    context.insert("optical_disks", &state.optical_disks);

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
pub fn rip_one(
    disk_id: &str,
    title_id: &str,
    mvdb_id: &str,
    app_state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, template::ApiError> {
    let title_id = cast_to_u32(title_id.to_string());
    let mvdb_id = cast_to_u32(mvdb_id.to_string());
    match DiskId::try_from(disk_id) {
        Ok(id) => match app_state.find_optical_disk_by_id(&id) {
            Some(optical_disk) => match find_movie(&app_handle, mvdb_id) {
                Ok(movie) => {
                    let movie_dir = create_dir(&movie);
                    optical_disk
                        .lock()
                        .unwrap()
                        .set_movie_details(Some(movie.clone()));
                    tauri::async_runtime::spawn(async move {
                        let results =
                            makemkvcon::rip_title(&app_handle, &id, title_id, &movie_dir).await;
                        match results {
                            Ok(_r) => match rename_file(&app_handle, &movie, id, title_id) {
                                Ok(p) => {
                                    let file_path = p.to_string_lossy().to_string();
                                    app_handle
                                        .notification()
                                        .builder()
                                        .title("Reelix")
                                        .body(format!("Finished Ripping {}", &file_path))
                                        .show()
                                        .unwrap();
                                }
                                Err(e) => {
                                    app_handle
                                        .notification()
                                        .builder()
                                        .title("Reelix")
                                        .body(format!("Error Ripping {}", &e))
                                        .show()
                                        .unwrap();
                                }
                            },
                            Err(message) => {
                                println!("failed {}", message);
                            }
                        }
                    });
                }
                Err(e) => eprintln!("Failure {}", e.message),
            },
            None => eprintln!("Failed to find optical disk"),
        },

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

// ----------------------------------
// ------- Helper Functions ---------
// ----------------------------------

fn render_tmdb_error(
    state: &State<AppState>,
    error_message: &str,
) -> Result<String, template::ApiError> {
    let api_key = get_api_key(state);
    let mut context = Context::new();
    context.insert("code", "500");
    context.insert("message", &format!("Error from TMDB: {}", error_message));
    context.insert("api_key", &api_key.to_owned());
    template::render(&state.tera, "the_movie_db/show.html.turbo", &context, None)
}

fn render_error(
    state: &State<'_, AppState>,
    error_message: &str,
) -> Result<String, template::ApiError> {
    let api_error = template::ApiError {
        code: 500,
        message: error_message.to_owned(),
        api_key: None,
    };

    let context = Context::from_serialize(&api_error)
        .expect("Failed to serialize API error context in the_movie_db command");
    template::render(&state.tera, "error.html.turbo", &context, None)
}

fn get_movie_certification(
    movie_db: the_movie_db::TheMovieDb,
    id: u32,
) -> Result<Option<String>, Error> {
    let release_dates = match movie_db.movie_release_dates(id) {
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

fn get_query(state: &State<'_, AppState>) -> String {
    state.query.lock().unwrap().to_string()
}

fn save_query(state: &State<'_, AppState>, search: &str) {
    let mut query = state.query.lock().unwrap();
    *query = search.to_string();
}

fn render_search_index(state: &State<'_, AppState>) -> Result<String, template::ApiError> {
    let mut context = Context::new();
    context.insert("optical_disks", &state.optical_disks);
    let binding_selected_disk_id = state
        .selected_optical_disk_id
        .lock()
        .expect("failed to lock selected optical disk id");
    let guard_selected_disk_id = binding_selected_disk_id.as_ref();
    if guard_selected_disk_id.is_some() {
        let disk_id = guard_selected_disk_id.unwrap().clone();
        context.insert("selected_optical_disk_id", &disk_id);
    }
    template::render(&state.tera, "search/index.html.turbo", &context, None)
}
