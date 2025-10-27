use crate::services::ftp_uploader::validate_ftp_settings;
use crate::state::AppState;
use crate::templates::InlineTemplate;
use askama::Template;
use tauri::State;

#[derive(Template)]
#[template(path = "ftp_settings/show.turbo.html")]
pub struct FtpSettingsShowTurbo<'a> {
    pub ftp_settings_show: &'a FtpSettingsShow<'a>,
}

#[derive(Template)]
#[template(path = "ftp_settings/show.html")]
pub struct FtpSettingsShow<'a> {
    pub ftp_host: &'a Option<String>,
    pub ftp_user: &'a Option<String>,
    pub ftp_pass: &'a Option<String>,
    pub ftp_movie_upload_path: &'a Option<String>,
    pub message: &'a str,
}

pub fn render_show(app_state: &State<'_, AppState>) -> Result<String, super::Error> {
    let ftp_host = {
        let locked_ftp_host = app_state.lock_ftp_host();
        locked_ftp_host.clone()
    };

    let ftp_pass = {
        let locked_ftp_pass = app_state.lock_ftp_pass();
        locked_ftp_pass.clone()
    };

    let ftp_user = {
        let locked_ftp_user = app_state.lock_ftp_user();
        locked_ftp_user.clone()
    };

    let ftp_movie_upload_path = {
        let locked_ftp_movie_upload_path = app_state.lock_ftp_movie_upload_path();
        locked_ftp_movie_upload_path.clone() // or extract what's needed
    };
    let mut message = String::new();
    if let Err(msg) = validate_ftp_settings(app_state) {
        message = msg;
    };
    let ftp_settings_show = FtpSettingsShow {
        ftp_host: &ftp_host,
        ftp_user: &ftp_user,
        ftp_pass: &ftp_pass,
        ftp_movie_upload_path: &ftp_movie_upload_path,
        message: &message,
    };
    let template = FtpSettingsShowTurbo {
        ftp_settings_show: &ftp_settings_show,
    };
    super::render(template)
}
