use super::{render, ApiError};
use crate::models::movie_db::{self, TvResponse};
use crate::state::AppState;
use tauri::State;
use tera::Context;

pub fn render_show(app_state: &State<'_, AppState>, tv: &TvResponse) -> Result<String, ApiError> {
    let query: String = app_state.query.lock().unwrap().to_string();

    let mut context = Context::new();
    context.insert("tv", &movie_db::TvView::from(tv.to_owned()));
    context.insert("query", &query);

    render(&app_state.tera, "tvs/show.html.turbo", &context, None)
}
