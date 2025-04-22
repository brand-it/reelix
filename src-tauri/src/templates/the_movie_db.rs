use super::{render, ApiError};
use crate::state::{get_api_key, AppState};
use tauri::State;
use tera::Context;

pub fn render_show(
    app_state: &State<'_, AppState>,
    error_message: &str,
) -> Result<String, ApiError> {
    let api_key = get_api_key(&app_state);
    let mut context = Context::new();
    context.insert("code", "500");
    context.insert("message", &format!("Error from TMDB: {}", error_message));
    context.insert("api_key", &api_key.to_owned());
    render(
        &app_state.tera,
        "the_movie_db/show.html.turbo",
        &context,
        None,
    )
}
