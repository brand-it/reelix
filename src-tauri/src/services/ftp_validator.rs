use crate::services::ftp_uploader::validate_ftp_settings_internal;
use crate::state::{AppState, FtpConnectionStatus};
use crate::templates::{ftp_status, toast};
use log::debug;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

#[derive(Clone, Debug, PartialEq)]
struct FtpConfigSnapshot {
    host: Option<String>,
    user: Option<String>,
    pass: Option<String>,
}

impl FtpConfigSnapshot {
    fn from_app_state(app_state: &AppState) -> Self {
        Self {
            host: app_state.lock_ftp_host().clone(),
            user: app_state.lock_ftp_user().clone(),
            pass: app_state.lock_ftp_pass().clone(),
        }
    }

    fn matches_current(&self, app_state: &AppState) -> bool {
        let current_host = app_state.lock_ftp_host();
        let current_user = app_state.lock_ftp_user();
        let current_pass = app_state.lock_ftp_pass();

        self.host == *current_host && self.user == *current_user && self.pass == *current_pass
    }
}

pub fn spawn_ftp_validator(app_handle: &AppHandle) {
    let app_handle = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        check_ftp(&app_handle).await;
        start_periodic_ftp_check(&app_handle).await;
    });
}

async fn start_periodic_ftp_check(app_handle: &AppHandle) {
    let mut interval = tokio::time::interval(Duration::from_secs(10));

    loop {
        interval.tick().await;
        let previous_status = {
            let app_state = app_handle.state::<AppState>();
            let status = *app_state.lock_ftp_connection_status();
            status
        };
        let new_status = check_ftp(app_handle).await;
        emit_toast(app_handle, previous_status, new_status);
    }
}

pub fn trigger_ftp_check(app_handle: &AppHandle) {
    let app_handle = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        check_ftp(&app_handle).await;
    });
}

async fn check_ftp(app_handle: &AppHandle) -> FtpConnectionStatus {
    let app_state = app_handle.state::<AppState>();

    // Capture config snapshot at the start of the check
    let config_snapshot = FtpConfigSnapshot::from_app_state(&app_state);

    let status = determine_initial_ftp_status(&app_state);

    {
        let mut connection_status = app_state.lock_ftp_connection_status();
        *connection_status = status;
    }

    if let Ok(turbo) = ftp_status::render_update(status) {
        let _ = app_handle.emit("disks-changed", turbo);
    }

    perform_ftp_check(app_handle, config_snapshot).await
}

async fn perform_ftp_check(
    app_handle: &AppHandle,
    config_snapshot: FtpConfigSnapshot,
) -> FtpConnectionStatus {
    let app_state = app_handle.state::<AppState>();
    let previous_status = *app_state.lock_ftp_connection_status();

    // Check if FTP is configured
    let is_configured = config_snapshot.host.is_some()
        && config_snapshot.user.is_some()
        && config_snapshot.pass.is_some();

    if !is_configured {
        // Verify config hasn't changed before emitting
        if !config_snapshot.matches_current(&app_state) {
            debug!("FTP config changed during check, discarding Unconfigured result");
            return previous_status;
        }

        let mut connection_status = app_state.lock_ftp_connection_status();
        *connection_status = FtpConnectionStatus::Unconfigured;
        if let Ok(turbo) = ftp_status::render_update(FtpConnectionStatus::Unconfigured) {
            let _ = app_handle.emit("disks-changed", turbo);
        }
        return previous_status;
    }

    // Verify config hasn't changed before setting to Checking
    if !config_snapshot.matches_current(&app_state) {
        debug!("FTP config changed during check, discarding before validation");
        return previous_status;
    }

    // Set to checking while we validate
    {
        let mut connection_status = app_state.lock_ftp_connection_status();
        *connection_status = FtpConnectionStatus::Checking;
    }

    // Perform validation
    let new_status = match validate_ftp_settings_internal(&app_state) {
        Ok(_) => FtpConnectionStatus::Connected,
        Err(_) => FtpConnectionStatus::Failed,
    };

    // Verify config hasn't changed before emitting results
    if !config_snapshot.matches_current(&app_state) {
        debug!("FTP config changed during check, discarding validation results");
        return previous_status;
    }

    // Update status
    {
        let mut connection_status = app_state.lock_ftp_connection_status();
        *connection_status = new_status;
    }

    // Emit status update
    if let Ok(turbo) = ftp_status::render_update(new_status) {
        let _ = app_handle.emit("disks-changed", turbo);
    };
    new_status
}

fn emit_toast(
    app_handle: &AppHandle,
    previous_status: FtpConnectionStatus,
    new_status: FtpConnectionStatus,
) {
    if should_emit_toast(previous_status, new_status) {
        let toast_msg = match new_status {
            FtpConnectionStatus::Connected => {
                toast::Toast::success("FTP Connection", "Successfully connected to FTP server")
                    .with_auto_hide(5000)
            }
            FtpConnectionStatus::Failed => {
                toast::Toast::danger("FTP Connection", "Failed to connect to FTP server")
                    .with_auto_hide(0) // Don't auto-hide errors
            }
            _ => return,
        };

        if let Ok(turbo) = toast::render_toast_append(toast_msg) {
            let _ = app_handle.emit("disks-changed", turbo);
        }
    }
}

fn determine_initial_ftp_status(app_state: &AppState) -> FtpConnectionStatus {
    let host = app_state.lock_ftp_host();
    let user = app_state.lock_ftp_user();
    let pass = app_state.lock_ftp_pass();

    if host.is_some() && user.is_some() && pass.is_some() {
        debug!("FTP is configured, starting with Checking status");
        FtpConnectionStatus::Checking
    } else {
        debug!("FTP is not configured");
        FtpConnectionStatus::Unconfigured
    }
}

fn should_emit_toast(previous: FtpConnectionStatus, current: FtpConnectionStatus) -> bool {
    matches!(
        (previous, current),
        (FtpConnectionStatus::Failed, FtpConnectionStatus::Connected)
            | (FtpConnectionStatus::Connected, FtpConnectionStatus::Failed)
    )
}
