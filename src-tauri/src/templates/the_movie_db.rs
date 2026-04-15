use super::InlineTemplate;
use super::{render, Error, GenericError};
use crate::{state::AppState, templates::GenericErrorTurbo};
use askama::Template;
use tauri::State;

#[derive(Template)]
#[template(path = "the_movie_db/index.turbo.html")]
pub struct TheMovieDBIndexTurbo<'a> {
    pub generic_error_turbo: &'a GenericErrorTurbo<'a>,
    pub the_movie_db_index: &'a TheMovieDBIndex,
}

#[derive(Template)]
#[template(path = "the_movie_db/index.html")]
pub struct TheMovieDBIndex;

impl TheMovieDBIndex {
    pub fn dom_id(&self) -> &'static str {
        super::INDEX_ID
    }
}

pub fn render_index(_app_state: &State<'_, AppState>, error_message: &str) -> Result<String, Error> {
    let error = GenericErrorTurbo {
        generic_error: &GenericError {
            message: &format!("Server API error: {error_message}"),
        },
    };
    let template = TheMovieDBIndexTurbo {
        generic_error_turbo: &error,
        the_movie_db_index: &TheMovieDBIndex,
    };
    render(template)
}
