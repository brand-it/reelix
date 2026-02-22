use crate::services::ftp_validator::{FtpChecker, FtpConnectionStatus};
use crate::templates::ftp_settings::FtpSettingsStatusMessage;
use crate::templates::InlineTemplate;
use askama::Template;
use tauri::Manager;

#[derive(Template)]
#[template(path = "ftp_status/container.html")]
pub struct FtpStatusContainer<'a> {
    pub ftp_checker: &'a FtpChecker,
}

impl FtpStatusContainer<'_> {
    pub fn dom_id(&self) -> &'static str {
        "ftp-status"
    }

    pub fn status_text(&self) -> &'static str {
        match self.ftp_checker.status {
            FtpConnectionStatus::Unconfigured => "Setup FTP",
            FtpConnectionStatus::Checking => "Checking...",
            FtpConnectionStatus::Connected => "Connected",
            FtpConnectionStatus::Failed => "Connection Failed",
        }
    }

    pub fn icon_class(&self) -> &'static str {
        match self.ftp_checker.status {
            FtpConnectionStatus::Unconfigured => "fas fa-cog",
            FtpConnectionStatus::Checking => "fas fa-spinner fa-spin",
            FtpConnectionStatus::Connected => "fas fa-check-circle",
            FtpConnectionStatus::Failed => "fas fa-exclamation-circle",
        }
    }
}

#[derive(Template)]
#[template(path = "ftp_status/update.turbo.html")]
pub struct FtpStatusUpdate<'a> {
    pub ftp_status: &'a FtpStatusContainer<'a>,
    pub ftp_settings_status_message: &'a FtpSettingsStatusMessage<'a>,
}

pub fn render_update(app_handle: &tauri::AppHandle) -> Result<String, crate::templates::Error> {
    let app_state = app_handle.state::<crate::state::AppState>();
    let ftp_checker = app_state.ftp_config.lock().unwrap().checker.clone();
    let ftp_status = FtpStatusContainer {
        ftp_checker: &ftp_checker,
    };
    let ftp_settings_status_message = FtpSettingsStatusMessage {
        ftp_checker: &ftp_checker,
    };

    let template = FtpStatusUpdate {
        ftp_status: &ftp_status,
        ftp_settings_status_message: &ftp_settings_status_message,
    };
    crate::templates::render(template)
}
