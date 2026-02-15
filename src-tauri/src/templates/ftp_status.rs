use crate::state::FtpConnectionStatus;
use crate::templates::InlineTemplate;
use crate::templates::ftp_settings::FtpSettingsStatusMessage;
use askama::Template;

#[derive(Template)]
#[template(path = "ftp_status/container.html")]
pub struct FtpStatus {
    pub status: FtpConnectionStatus,
}

impl FtpStatus {
    pub fn dom_id(&self) -> &'static str {
        "ftp-status"
    }

    pub fn status_text(&self) -> &'static str {
        match self.status {
            FtpConnectionStatus::Unconfigured => "Setup FTP",
            FtpConnectionStatus::Checking => "Checking...",
            FtpConnectionStatus::Connected => "Connected",
            FtpConnectionStatus::Failed => "Connection Failed",
        }
    }

    pub fn icon_class(&self) -> &'static str {
        match self.status {
            FtpConnectionStatus::Unconfigured => "fas fa-cog",
            FtpConnectionStatus::Checking => "fas fa-spinner fa-spin",
            FtpConnectionStatus::Connected => "fas fa-check-circle",
            FtpConnectionStatus::Failed => "fas fa-exclamation-circle",
        }
    }
}

#[derive(Template)]
#[template(path = "ftp_status/update.turbo.html")]
pub struct FtpStatusUpdate {
    pub ftp_status: FtpStatus,
    pub ftp_settings_status_message: FtpSettingsStatusMessage,
}

pub fn render_update(status: FtpConnectionStatus) -> Result<String, crate::templates::Error> {
    let ftp_status = FtpStatus { status };
    let ftp_settings_status_message = FtpSettingsStatusMessage::new(status);
    let template = FtpStatusUpdate {
        ftp_status,
        ftp_settings_status_message,
    };
    crate::templates::render(template)
}
