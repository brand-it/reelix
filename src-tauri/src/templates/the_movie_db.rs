use super::{render, Error, GenericError};
use crate::state::AppState;
use askama::Template;
use tauri::State;

#[derive(Template)]
#[template(path = "the_movie_db/show.turbo.html")]
pub struct TheMovieDBShow<'a> {
    error: GenericError<'a>,
    api_key: &'a str,
}

pub fn render_show(app_state: &State<'_, AppState>, error_message: &str) -> Result<String, Error> {
    let error = GenericError {
        message: &format!("Error from TMDB: {error_message}"),
    };
    let api_key = &app_state.lock_the_movie_db_key().to_string();
    let template = TheMovieDBShow { error, api_key };
    render(template)
}
