use super::{render, ApiError};
use crate::models::movie_db;
use crate::services::ftp_uploader;
use crate::state::AppState;
use tauri::State;
use tera::Context;

pub fn render_show(
    app_state: &State<'_, AppState>,
    movie: &movie_db::MovieResponse,
    certification: &Option<String>,
) -> Result<String, ApiError> {
    let query = app_state.query.lock().unwrap().to_string();

    let mut context = Context::new();
    let relative_mkv_file_path = movie.to_file_path();

    context.insert("movie", &movie_db::MovieView::from(movie.to_owned()));
    context.insert("query", &query);
    context.insert("certification", &certification);
    context.insert("selected_disk", &app_state.selected_disk());
    context.insert(
        "ripped",
        &ftp_uploader::file_exists(&relative_mkv_file_path, app_state),
    );

    render(&app_state.tera, "movies/show.html.turbo", &context, None)
}

pub fn render_cards(
    app_state: &State<'_, AppState>,
    movie: &movie_db::MovieResponse,
) -> Result<String, ApiError> {
    let mut context = Context::new();

    context.insert("movie", &movie_db::MovieView::from(movie.to_owned()));
    context.insert("selected_disk", &app_state.selected_disk());
    render(&app_state.tera, "movies/cards.html.turbo", &context, None)
}
