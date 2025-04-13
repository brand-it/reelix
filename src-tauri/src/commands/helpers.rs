use crate::services::the_movie_db::Error;
use crate::services::{template, the_movie_db};
use crate::state::{get_api_key, AppState};
use tauri::State;
use tera::Context;

pub fn render_tmdb_error(
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

pub fn render_error(
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

pub fn get_movie_certification(
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

pub fn get_query(state: &State<'_, AppState>) -> String {
    state.query.lock().unwrap().to_string()
}

pub fn save_query(state: &State<'_, AppState>, search: &str) {
    let mut query = state.query.lock().unwrap();
    *query = search.to_string();
}

pub fn render_search_index(state: &State<'_, AppState>) -> Result<String, template::ApiError> {
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
