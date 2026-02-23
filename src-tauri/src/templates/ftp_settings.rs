use crate::services::ftp_validator::FtpChecker;
use crate::state::FtpConfig;
use crate::templates::ftp_status::FtpStatusContainer;
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
    pub ftp_config: &'a FtpConfig,
    pub ftp_status_container: &'a FtpStatusContainer<'a>,
    pub status_message: &'a FtpSettingsStatusMessage<'a>,
}

impl FtpSettingsIndex<'_> {
    pub fn dom_id(&self) -> &'static str {
        super::INDEX_ID
    }
}

#[derive(Template)]
#[template(path = "ftp_settings/status_message.html")]
pub struct FtpSettingsStatusMessage<'a> {
    pub ftp_checker: &'a FtpChecker,
}

impl FtpSettingsStatusMessage<'_> {
    pub fn dom_id(&self) -> &'static str {
        "ftp-settings-status-message"
    }
}

pub fn render_show(state: &crate::state::AppState) -> Result<String, crate::templates::Error> {
    let ftp_checker = state.ftp_config.lock().unwrap().checker.clone();
    let ftp_status_container = FtpStatusContainer {
        ftp_checker: &ftp_checker,
    };

    let status_message = FtpSettingsStatusMessage {
        ftp_checker: &ftp_checker,
    };
    let ftp_settings_index = FtpSettingsIndex {
        ftp_config: &state.ftp_config.lock().unwrap(),
        ftp_status_container: &ftp_status_container,
        status_message: &status_message,
    };
    let template = FtpSettingsIndexTurbo {
        ftp_settings_index: &ftp_settings_index,
    };
    crate::templates::render(template)
}
