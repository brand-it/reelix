use super::{render, ApiError};
use crate::services::ftp_uploader::validate_ftp_settings;
use crate::state::AppState;
use tauri::State;
use tera::Context;

pub fn render_show(app_state: &State<'_, AppState>) -> Result<String, ApiError> {
    let mut context = Context::new();

    let ftp_host = {
        let locked_ftp_host = app_state.lock_ftp_host();
        locked_ftp_host.clone()
    };
    context.insert("ftp_host", &ftp_host);

    let ftp_pass = {
        let locked_ftp_pass = app_state.lock_ftp_pass();
        locked_ftp_pass.clone()
    };
    context.insert("ftp_pass", &ftp_pass);

    let ftp_user = {
        let locked_ftp_user = app_state.lock_ftp_user();
        locked_ftp_user.clone()
    };
    context.insert("ftp_user", &ftp_user);

    let ftp_movie_upload_path = {
        let locked_ftp_movie_upload_path = app_state.lock_ftp_movie_upload_path();
        locked_ftp_movie_upload_path.clone() // or extract what's needed
    };
    context.insert("ftp_movie_upload_path", &ftp_movie_upload_path);

    if let Err(message) = validate_ftp_settings(app_state) {
        context.insert("message", &message);
    };

    render(
        &app_state.tera,
        "ftp_settings/show.html.turbo",
        &context,
        None,
    )
}
