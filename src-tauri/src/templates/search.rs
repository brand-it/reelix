use super::disks::build_disk_option;
use super::{render, the_movie_db, ApiError};
use crate::models::movie_db::SearchResponse;
use crate::services::plex::search_multi;
use crate::state::AppState;
use tauri::State;
use tera::Context;

pub fn render_index(app_state: &State<'_, AppState>) -> Result<String, ApiError> {
    let disk_option = build_disk_option(app_state);
    let mut context = Context::from_serialize(&disk_option).unwrap();
    let query = app_state.query.lock().unwrap().to_string();
    let search = match search_multi(app_state, &query) {
        Ok(resp) => resp,
        Err(e) => return the_movie_db::render_show(app_state, &e.message),
    };

    context.insert("query", &query);
    context.insert("search", &search);
    render(&app_state.tera, "search/index.html.turbo", &context, None)
}

pub fn render_results(
    app_state: &State<'_, AppState>,
    query: &str,
    search: &SearchResponse,
) -> Result<String, ApiError> {
    let mut context: Context = Context::new();
    context.insert("search", search);
    context.insert("query", query);

    render(&app_state.tera, "search/results.html.turbo", &context, None)
}

pub fn render_suggestion(
    app_state: &State<'_, AppState>,
    query: &str,
    suggestion: &Option<String>,
) -> Result<String, ApiError> {
    let mut context: Context = Context::new();
    context.insert("query", query);
    context.insert("suggestion", suggestion);

    render(
        &app_state.tera,
        "search/suggestion.html.turbo",
        &context,
        None,
    )
}
