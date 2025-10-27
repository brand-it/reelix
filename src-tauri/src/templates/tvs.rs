use crate::models::movie_db::TvResponse;
use crate::state::AppState;
use tauri::State;

// #[derive(Template)]
// #[template(path = "tvs/show.turbo.html")]
// pub struct TvsShow<'a> {
//     pub tv: &'a TvResponse,
//     pub query: &'a String,
// }

pub fn render_show(
    _app_state: &State<'_, AppState>,
    _tv: &TvResponse,
) -> Result<String, super::Error> {
    // let query = &app_state.query.lock().unwrap().to_string();

    // let template = TvsShow { tv, query };

    // render(template)
    Ok("".to_string())
}
