use crate::models::movie_db;
use crate::models::optical_disk_info::OpticalDiskInfo;
use crate::state::AppState;
use askama::Template;
use tauri::State;

#[derive(Template)]
#[template(path = "movies/cards.html")]
pub struct MoviesCards<'a> {
    pub selected_disk: &'a Option<OpticalDiskInfo>,
}

pub fn render_show(
    _app_state: &State<'_, AppState>,
    _movie: &movie_db::MovieResponse,
    _certification: &Option<String>,
) -> Result<String, super::Error> {
    Ok("".to_string())
    // let query = app_state.query.lock().unwrap().to_string();

    // let mut context = Context::new();
    // let relative_mkv_file_path = movie.to_file_path();

    // context.insert("movie", &movie_db::MovieView::from(movie.to_owned()));
    // context.insert("query", &query);
    // context.insert("certification", &certification);
    // context.insert("selected_disk", &app_state.selected_disk());
    // context.insert(
    //     "ripped",
    //     &ftp_uploader::file_exists(&relative_mkv_file_path, app_state),
    // );

    // render(&app_state.tera, "movies/show.html.turbo", &context, None)
}

pub fn render_cards(
    _app_state: &State<'_, AppState>,
    _movie: &movie_db::MovieResponse,
) -> Result<String, super::Error> {
    Ok("".to_string())
    // let mut context = Context::new();

    // context.insert("movie", &movie_db::MovieView::from(movie.to_owned()));
    // context.insert("selected_disk", &app_state.selected_disk());
    // render(&app_state.tera, "movies/cards.html.turbo", &context, None)
}
