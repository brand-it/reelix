use super::disks::build_disk_option;
use super::{render, ApiError};
use crate::models::movie_db::{self, SeasonEpisode, SeasonResponse, TvResponse};
use crate::state::AppState;
use tauri::State;
use tera::Context;

pub fn render_show(
    app_state: &State<'_, AppState>,
    tv: &TvResponse,
    season: &SeasonResponse,
) -> Result<String, ApiError> {
    let mut context = Context::new();
    context.insert("tv", &movie_db::TvView::from(tv.to_owned()));
    context.insert("season", &movie_db::SeasonView::from(season.to_owned()));
    context.insert("selected_disk", &app_state.selected_disk());

    render(&app_state.tera, "seasons/show.html.turbo", &context, None)
}

pub fn render_title_selected(
    app_state: &State<'_, AppState>,
    season: SeasonResponse,
) -> Result<String, ApiError> {
    let disk_option = build_disk_option(app_state);
    let mut context = Context::from_serialize(&disk_option).unwrap();
    context.insert("season", &movie_db::SeasonView::from(season));
    render(
        &app_state.tera,
        "seasons/title_selected.html.turbo",
        &context,
        None,
    )
}
