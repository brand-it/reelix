use crate::state::FtpConnectionStatus;
use crate::templates::ftp_status::FtpStatus;
use crate::templates::InlineTemplate;
use askama::Template;

#[derive(Template)]
#[template(path = "ftp_settings/index.turbo.html")]
pub struct FtpSettingsIndexTurbo<'a> {
    pub ftp_settings_index: &'a FtpSettingsIndex<'a>,
}

#[derive(Template)]
#[template(path = "ftp_settings/index.html")]
pub struct FtpSettingsIndex<'a> {
    pub ftp_host: &'a Option<String>,
    pub ftp_user: &'a Option<String>,
    pub ftp_pass: &'a Option<String>,
    pub ftp_movie_upload_path: &'a Option<String>,
    pub status_message: FtpSettingsStatusMessage,
    pub ftp_status: FtpStatus,
}

impl FtpSettingsIndex<'_> {
    pub fn dom_id(&self) -> &'static str {
        super::INDEX_ID
    }
}

#[derive(Template)]
#[template(path = "ftp_settings/status_message.html")]
pub struct FtpSettingsStatusMessage {
    pub status: FtpConnectionStatus,
}

impl FtpSettingsStatusMessage {
    pub fn new(status: FtpConnectionStatus) -> Self {
        Self { status }
    }

    pub fn dom_id(&self) -> &'static str {
        "ftp-settings-status-message"
    }
}

pub fn render_show(state: &crate::state::AppState) -> Result<String, crate::templates::Error> {
    let ftp_host = state.lock_ftp_host();
    let ftp_user = state.lock_ftp_user();
    let ftp_pass = state.lock_ftp_pass();
    let ftp_movie_upload_path = state.lock_ftp_movie_upload_path();
    let ftp_movie_upload_path_str: Option<String> = ftp_movie_upload_path
        .as_ref()
        .map(|p| p.to_string_lossy().to_string());
    let status = state.lock_ftp_connection_status();
    let ftp_status = FtpStatus { status: *status };
    let status_message = FtpSettingsStatusMessage::new(*status);
    let ftp_settings_index = FtpSettingsIndex {
        ftp_host: &ftp_host,
        ftp_user: &ftp_user,
        ftp_pass: &ftp_pass,
        ftp_movie_upload_path: &ftp_movie_upload_path_str,
        status_message,
        ftp_status,
    };
    let template = FtpSettingsIndexTurbo {
        ftp_settings_index: &ftp_settings_index,
    };
    crate::templates::render(template)
}
