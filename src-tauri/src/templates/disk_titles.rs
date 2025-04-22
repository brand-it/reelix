use super::{render, ApiError};
use crate::state::AppState;
use tauri::State;
use tera::Context;

pub fn render_options(app_state: &State<'_, AppState>) -> Result<String, ApiError> {
    let mut context = Context::new();
    context.insert("selected_disk", &app_state.selected_disk());
    render(
        &app_state.tera,
        "disk_titles/options.html.turbo",
        &context,
        None,
    )
}
