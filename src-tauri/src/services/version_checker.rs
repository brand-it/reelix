use crate::services::semantic_version::SemanticVersion;
use crate::standard_error::StandardError;
use log::debug;
use tauri::App;
use tauri::AppHandle;
use tauri::Emitter;
use tauri::Manager;

use super::github_api;
use crate::templates;

#[derive(Debug, Clone)]
pub struct VersionState {
    pub current_version: SemanticVersion,
    pub latest_version: SemanticVersion,
    pub has_update: bool,
}

impl VersionState {
    pub fn new(current_version: SemanticVersion, latest_version: SemanticVersion) -> Self {
        let has_update = latest_version > current_version;
        Self {
            current_version,
            latest_version,
            has_update,
        }
    }
}

pub fn spawn_version_checker(app: &App) {
    let app_handle = app.handle().clone();
    tauri::async_runtime::spawn(async move {
        match check_on_boot(&app_handle).await {
            Ok(version_state) => {
                let app_state = app_handle.state::<crate::state::AppState>();
                app_state
                    .update(
                        &app_handle,
                        "latest_version",
                        Some(version_state.latest_version.to_string()),
                    )
                    .map(|_| debug!("Latest version updated: {:?}", version_state.latest_version))
                    .ok();

                if version_state.has_update {
                    if let Ok(turbo) = templates::update_indicator::render_update(&version_state) {
                        let _ = app_handle.emit("disks-changed", turbo);
                    }
                }
            }
            Err(e) => {
                debug!("Failed to check for updates: {e}");
            }
        }
    });
}

pub async fn check_on_boot(app_handle: &AppHandle) -> Result<VersionState, StandardError> {
    let current_version =
        match SemanticVersion::parse(&app_handle.package_info().version.to_string()) {
            Ok(v) => v,
            Err(_e) => SemanticVersion::none(),
        };
    let app_state = app_handle.state::<crate::state::AppState>();
    let version_state = app_state.get_version_state(app_handle);
    if version_state.has_update {
        return Ok(version_state);
    }

    let latest_version = github_api::fetch_latest_release_version().await;

    Ok(VersionState::new(current_version, latest_version))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_state_new_has_update() {
        let current_version = SemanticVersion::parse("1.0.0").unwrap();
        let latest_version = SemanticVersion::parse("1.1.0").unwrap();
        let state = VersionState::new(current_version, latest_version);

        assert_eq!(
            state.current_version,
            SemanticVersion::parse("1.0.0").unwrap()
        );
        assert_eq!(
            state.latest_version,
            SemanticVersion::parse("1.1.0").unwrap()
        );
        assert!(state.has_update);
    }

    #[test]
    fn test_version_state_new_no_update_with_older_version() {
        let current_version = SemanticVersion::parse("1.0.0").unwrap();
        let latest_version = SemanticVersion::parse("1.0.0").unwrap();
        let state = VersionState::new(current_version, latest_version);

        assert_eq!(
            state.current_version,
            SemanticVersion::parse("1.0.0").unwrap()
        );
        assert_eq!(
            state.latest_version,
            SemanticVersion::parse("1.0.0").unwrap()
        );
        assert!(!state.has_update);
    }

    #[test]
    fn test_version_state_new_no_update() {
        let current_version = SemanticVersion::parse("1.0.0").unwrap();
        let latest_version = SemanticVersion::parse("0.1.0").unwrap();
        let state = VersionState::new(current_version, latest_version);

        assert_eq!(
            state.current_version,
            SemanticVersion::parse("1.0.0").unwrap()
        );
        assert_eq!(
            state.latest_version,
            SemanticVersion::parse("0.1.0").unwrap()
        );
        assert!(!state.has_update);
    }

    #[test]
    fn test_version_state_struct_with_update() {
        let state = VersionState {
            current_version: SemanticVersion::parse("1.0.0").unwrap(),
            latest_version: SemanticVersion::parse("1.1.0").unwrap(),
            has_update: true,
        };

        assert_eq!(
            state.current_version,
            SemanticVersion::parse("1.0.0").unwrap()
        );
        assert_eq!(
            state.latest_version,
            SemanticVersion::parse("1.1.0").unwrap()
        );
        assert!(state.has_update);
    }

    #[test]
    fn test_version_state_struct_no_update() {
        let state = VersionState {
            current_version: SemanticVersion::parse("1.0.0").unwrap(),
            latest_version: SemanticVersion::none(),
            has_update: false,
        };

        assert_eq!(
            state.current_version,
            SemanticVersion::parse("1.0.0").unwrap()
        );
        assert_eq!(state.latest_version, SemanticVersion::none());
        assert!(!state.has_update);
    }
}
