use crate::services::ftp_uploader;
use crate::state::{AppState, FtpConfig};
use crate::templates::{ftp_status, toast};
use log::debug;
use std::collections::HashSet;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FtpConnectionStatus {
    Unconfigured,
    Checking,
    Connected,
    Failed,
}

#[derive(Clone)]
pub struct FtpChecker {
    pub status: FtpConnectionStatus,
    pub validation_error: Option<ftp_uploader::FtpValidationError>,
}

impl Default for FtpChecker {
    fn default() -> Self {
        Self {
            status: FtpConnectionStatus::Unconfigured,
            validation_error: None,
        }
    }
}

impl FtpChecker {
    pub fn new() -> Self {
        Self {
            status: FtpConnectionStatus::Checking,
            validation_error: None,
        }
    }

    fn config_changed_during_check(&self, app_handle: &AppHandle, baseline: &FtpConfig) -> bool {
        let app_state = app_handle.state::<AppState>();
        let current_config = app_state.lock_ftp_config();
        should_discard_checker_update(baseline, &current_config)
    }

    fn describe_missing_config(&self, config: &FtpConfig) -> String {
        let mut missing_fields = Vec::new();
        if config.host.is_none() {
            missing_fields.push("host");
        }
        if config.user.is_none() {
            missing_fields.push("user");
        }
        if config.pass.is_none() {
            missing_fields.push("pass");
        }
        format!(
            "You are missing the following FTP settings: {}",
            missing_fields.join(", ")
        )
    }

    fn check(&mut self, app_handle: &AppHandle, baseline: &FtpConfig) {
        if !baseline.is_configured() {
            self.status = FtpConnectionStatus::Unconfigured;
            let mut error = ftp_uploader::FtpValidationError::new();

            error.add_error(
                "FTP settings are incomplete".to_string(),
                ftp_uploader::FtpErrorType::MissingConfig,
                None,
                Some(self.describe_missing_config(baseline)),
                Vec::new(),
            );
            self.validation_error = Some(error);
        } else if let Err(error) = self.check_ftp_connection(app_handle) {
            self.status = FtpConnectionStatus::Failed;
            self.validation_error = Some(error);
        } else if let Err(error) = self.validate_ftp_paths(app_handle) {
            self.status = FtpConnectionStatus::Failed;
            self.validation_error = Some(error);
        } else {
            self.status = FtpConnectionStatus::Connected;
            self.validation_error = None;
        };
    }

    fn check_ftp_connection(
        &self,
        app_handle: &AppHandle,
    ) -> Result<(), ftp_uploader::FtpValidationError> {
        let state = app_handle.state::<AppState>();
        match ftp_uploader::connect_to_ftp(&state) {
            Ok(_stream) => Ok(()),
            Err(e) => {
                let mut error = ftp_uploader::FtpValidationError::new();
                error.add_error(
                    "Failed to connect to FTP server".to_string(),
                    ftp_uploader::FtpErrorType::ConnectionFailed,
                    None,
                    Some(e.to_string()),
                    Vec::new(),
                );
                Err(error)
            }
        }
    }

    fn validate_ftp_paths(
        &self,
        app_handle: &AppHandle,
    ) -> Result<(), ftp_uploader::FtpValidationError> {
        let state = app_handle.state::<AppState>();
        let movie_upload_path = state.lock_ftp_movie_upload_path().clone();
        let tv_upload_path = state.lock_ftp_tv_upload_path().clone();
        let mut validation_error = ftp_uploader::FtpValidationError::new();

        // Check both upload paths
        if movie_upload_path.is_none() {
            let suggestions = self.suggest_path_list(app_handle, "");
            validation_error.add_error(
                "Movie upload path must be configured".to_string(),
                ftp_uploader::FtpErrorType::MissingConfig,
                None,
                None,
                suggestions,
            );
        }
        if tv_upload_path.is_none() {
            let suggestions = self.suggest_path_list(app_handle, "");
            validation_error.add_error(
                "TV upload path must be configured".to_string(),
                ftp_uploader::FtpErrorType::MissingConfig,
                None,
                None,
                suggestions,
            );
        }

        // If we have config errors, return them
        if validation_error.has_errors() {
            return Err(validation_error);
        }

        // Try to connect
        let mut ftp_stream = match ftp_uploader::connect_to_ftp(&state) {
            Ok(stream) => stream,
            Err(e) => {
                validation_error.add_error(
                    "Failed to connect to FTP server".to_string(),
                    ftp_uploader::FtpErrorType::ConnectionFailed,
                    None,
                    Some(e.to_string()),
                    Vec::new(),
                );
                return Err(validation_error);
            }
        };

        // Validate movie path
        let movie_path = movie_upload_path.unwrap();
        if let Err(e) = ftp_uploader::cwd(&mut ftp_stream, &movie_path) {
            let path_str = movie_path.to_string_lossy().to_string();
            let suggestions = self.suggest_path_list(app_handle, &path_str);
            validation_error.add_error(
                "Movie path not found".to_string(),
                ftp_uploader::FtpErrorType::PathNotFound,
                Some(path_str),
                Some(e.to_string()),
                suggestions,
            );
        }

        // Validate TV path
        let tv_path = tv_upload_path.unwrap();
        if let Err(e) = ftp_uploader::cwd(&mut ftp_stream, &tv_path) {
            let path_str = tv_path.to_string_lossy().to_string();
            let suggestions = self.suggest_path_list(app_handle, &path_str);
            validation_error.add_error(
                "TV path not found".to_string(),
                ftp_uploader::FtpErrorType::PathNotFound,
                Some(path_str),
                Some(e.to_string()),
                suggestions,
            );
        }

        // Try to quit cleanly
        if let Err(e) = ftp_stream.quit() {
            validation_error.add_error(
                "Failed to close FTP connection".to_string(),
                ftp_uploader::FtpErrorType::Other,
                None,
                Some(e.to_string()),
                Vec::new(),
            );
        }

        if validation_error.has_errors() {
            Err(validation_error)
        } else {
            Ok(())
        }
    }

    /// Get directory suggestions as a Vec for structured error handling
    fn suggest_path_list(&self, app_handle: &AppHandle, attempted_path: &str) -> Vec<String> {
        let state = app_handle.state::<AppState>();
        let mut ftp_stream = match ftp_uploader::connect_to_ftp(&state) {
            Ok(stream) => stream,
            Err(_) => return Vec::new(),
        };

        // If blank, suggest root directories
        if attempted_path.is_empty() || attempted_path == "/" {
            match ftp_uploader::list_directories(&mut ftp_stream, "/") {
                Ok(dirs) if !dirs.is_empty() => {
                    let _ = ftp_stream.quit();
                    return rank_suggestions(dirs, attempted_path);
                }
                _ => {
                    let _ = ftp_stream.quit();
                    return Vec::new();
                }
            }
        }

        // Walk up the path until we find one that exists
        let path_parts: Vec<&str> = attempted_path
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        for i in (0..path_parts.len()).rev() {
            let test_path = if i == 0 {
                "/".to_string()
            } else {
                format!("/{}", path_parts[..i].join("/"))
            };

            // Try this path
            if let Ok(dirs) = ftp_uploader::list_directories(&mut ftp_stream, &test_path) {
                if !dirs.is_empty() {
                    let _ = ftp_stream.quit();
                    return rank_suggestions(dirs, attempted_path);
                }
            }
        }

        let _ = ftp_stream.quit();
        Vec::new()
    }
}

fn tokenize_for_match(value: &str) -> Vec<String> {
    value
        .split(|c: char| !c.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(|token| token.to_ascii_lowercase())
        .collect()
}

fn suggestion_score(candidate: &str, attempted_path: &str) -> i32 {
    let candidate_lower = candidate.to_ascii_lowercase();
    let attempted_parts: Vec<&str> = attempted_path
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();
    let last_segment = attempted_parts
        .last()
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();

    let mut score = 0;
    if !last_segment.is_empty() {
        if candidate_lower == last_segment {
            score += 60;
        }
        if candidate_lower.contains(&last_segment) || last_segment.contains(&candidate_lower) {
            score += 30;
        }
    }

    let attempted_tokens = tokenize_for_match(attempted_path);
    let candidate_tokens = tokenize_for_match(candidate);
    for token in attempted_tokens {
        if candidate_tokens
            .iter()
            .any(|candidate_token| candidate_token == &token)
        {
            score += 10;
        } else if candidate_lower.contains(&token) {
            score += 5;
        }
    }

    if candidate.starts_with('@') {
        score -= 4;
    }

    score
}

fn rank_suggestions(dirs: Vec<String>, attempted_path: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut ranked: Vec<(i32, String)> = dirs
        .into_iter()
        .filter(|dir| !dir.trim().is_empty() && dir != "." && dir != "..")
        .filter(|dir| seen.insert(dir.to_ascii_lowercase()))
        .map(|dir| (suggestion_score(&dir, attempted_path), dir))
        .collect();

    ranked.sort_by(|left, right| {
        right.0.cmp(&left.0).then_with(|| {
            left.1
                .to_ascii_lowercase()
                .cmp(&right.1.to_ascii_lowercase())
        })
    });

    ranked.into_iter().take(20).map(|(_, dir)| dir).collect()
}

fn should_discard_checker_update(baseline: &FtpConfig, current: &FtpConfig) -> bool {
    baseline != current
}

pub fn spawn_ftp_validator(app_handle: &AppHandle) {
    let app_handle = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        start_periodic_ftp_check(&app_handle).await;
    });
}

fn run_ftp_check_with_statuses(
    app_handle: &AppHandle,
) -> (FtpConnectionStatus, FtpConnectionStatus) {
    let app_state = app_handle.state::<AppState>();
    let ftp_config = app_state.ftp_config.clone();

    let config_snapshot = {
        let config = ftp_config
            .lock()
            .expect("failed to lock ftp_config to clone config snapshot");
        config.clone()
    };

    let mut checker = FtpChecker::new();

    let previous_status = config_snapshot.checker.status;

    // Publish an immediate "Checking" state so the UI reflects active validation.
    {
        let mut config = ftp_config
            .lock()
            .expect("failed to lock ftp_config to set checker checking state");
        config.checker = checker.clone();
    }
    if let Ok(turbo) = ftp_status::render_update(app_handle) {
        app_handle.emit("disks-changed", turbo).unwrap_or_else(|e| {
            debug!("Failed to emit FTP status update: {e}");
        });
    }

    checker.check(app_handle, &config_snapshot);
    let new_status = checker.status;

    if checker.config_changed_during_check(app_handle, &config_snapshot) {
        debug!("FTP config changed during check, discarding result");
        return (previous_status, previous_status);
    }

    {
        let mut config = ftp_config
            .lock()
            .expect("failed to lock ftp_config to update checker");
        config.checker = checker;
    }

    if let Ok(turbo) = ftp_status::render_update(app_handle) {
        app_handle.emit("disks-changed", turbo).unwrap_or_else(|e| {
            debug!("Failed to emit FTP status update: {e}");
        });
    }

    (previous_status, new_status)
}

async fn start_periodic_ftp_check(app_handle: &AppHandle) {
    let mut interval = tokio::time::interval(Duration::from_secs(30));

    loop {
        interval.tick().await;
        let (previous_status, new_status) = run_ftp_check_with_statuses(app_handle);
        emit_toast(app_handle, previous_status, new_status);
    }
}

pub fn trigger_ftp_check(app_handle: &AppHandle) {
    let app_handle = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        let _ = run_ftp_check_with_statuses(&app_handle);
    });
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
                    .with_action("View Settings", "/ftp_settings")
            }
            FtpConnectionStatus::Failed => {
                toast::Toast::danger("FTP Connection", "Failed to connect to FTP server")
                    .with_auto_hide(0) // Don't auto-hide errors
                    .with_action("Fix Settings", "/ftp_settings")
            }
            _ => return,
        };

        if let Ok(turbo) = toast::render_toast_append(toast_msg) {
            let _ = app_handle.emit("disks-changed", turbo);
        }
    }
}

fn should_emit_toast(previous: FtpConnectionStatus, current: FtpConnectionStatus) -> bool {
    matches!(
        (previous, current),
        (FtpConnectionStatus::Failed, FtpConnectionStatus::Connected)
            | (FtpConnectionStatus::Connected, FtpConnectionStatus::Failed)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn configured_config() -> FtpConfig {
        FtpConfig {
            host: Some("ftp.example.com:21".to_string()),
            user: Some("reelix".to_string()),
            pass: Some("secret".to_string()),
            movie_upload_path: Some(PathBuf::from("/Media/Movies")),
            tv_upload_path: Some(PathBuf::from("/Media/TV Shows")),
            checker: FtpChecker::default(),
        }
    }

    #[test]
    fn should_not_discard_when_config_unchanged() {
        let baseline = configured_config();
        let current = baseline.clone();

        assert!(!should_discard_checker_update(&baseline, &current));
    }

    #[test]
    fn should_discard_when_host_changes() {
        let baseline = configured_config();
        let mut current = baseline.clone();
        current.host = Some("ftp.new-host.local:21".to_string());

        assert!(should_discard_checker_update(&baseline, &current));
    }

    #[test]
    fn should_discard_when_upload_path_changes() {
        let baseline = configured_config();
        let mut current = baseline.clone();
        current.tv_upload_path = Some(PathBuf::from("/Media/Shows"));

        assert!(should_discard_checker_update(&baseline, &current));
    }

    #[test]
    fn should_not_discard_when_only_checker_changes() {
        let baseline = configured_config();
        let mut current = baseline.clone();
        current.checker.status = FtpConnectionStatus::Failed;
        current.checker.validation_error = Some(ftp_uploader::FtpValidationError::new());

        assert!(!should_discard_checker_update(&baseline, &current));
    }
}
