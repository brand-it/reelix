use super::{render, ApiError};
use crate::state::AppState;
use tauri::State;
use tera::Context;

pub fn render_show(
    app_state: &State<'_, AppState>,
    error_message: &str,
) -> Result<String, ApiError> {
    let mut context = Context::new();
    context.insert("code", "500");
    context.insert("message", &format!("Error from TMDB: {error_message}"));
    let api_key = &app_state.lock_the_movie_db_key().to_string();
    context.insert("api_key", api_key);
    render(
        &app_state.tera,
        "the_movie_db/show.html.turbo",
        &context,
        None,
    )
}
