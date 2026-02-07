use log::debug;
use serde::{Deserialize, Serialize};
use tauri::App;
use tauri::AppHandle;
use tauri::Emitter;
use tauri::Manager;
use tauri_plugin_http::reqwest::Client;

use crate::templates;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VersionState {
    pub current_version: String,
    pub latest_version: Option<String>,
    pub has_update: bool,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
}

pub fn spawn_version_checker(app: &App) {
    let app_handle = app.handle().clone();
    tauri::async_runtime::spawn(async move {
        match check_on_boot(&app_handle).await {
            Ok(state) => {
                let app_state = app_handle.state::<crate::state::AppState>();
                app_state
                    .update(&app_handle, "latest_version", state.latest_version.clone())
                    .map(|_| debug!("Latest version updated: {:?}", state.latest_version))
                    .ok();
                app_state
                    .update(
                        &app_handle,
                        "has_update",
                        Some(state.has_update.to_string()),
                    )
                    .map(|_| debug!("Version state updated: {state:?}"))
                    .ok();

                if state.has_update {
                    if let Ok(turbo) = templates::update_indicator::render_update(&state) {
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

pub async fn check_on_boot(app_handle: &AppHandle) -> Result<VersionState, String> {
    let current_version = app_handle.package_info().version.to_string();
    let app_state = app_handle.state::<crate::state::AppState>();
    let mut state = app_state.get_version_state(app_handle);

    if state.has_update {
        if let Some(latest_version) = &state.latest_version {
            let current_clean = current_version.trim_start_matches('v');
            let latest_clean = latest_version.trim_start_matches('v');

            if latest_clean != current_clean {
                state.current_version = current_version;
                return Ok(state);
            }
        }
    }

    let (latest_version, has_update) = check_for_update(&current_version).await?;
    let updated_state = VersionState {
        current_version,
        latest_version: Some(latest_version),
        has_update,
    };

    Ok(updated_state)
}

pub async fn check_for_update(current_version: &str) -> Result<(String, bool), String> {
    let client = Client::new();

    let response = client
        .get("https://api.github.com/repos/brand-it/reelix/releases/latest")
        .header("User-Agent", "Reelix")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch latest release: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("GitHub API returned status: {}", response.status()));
    }

    let release: GitHubRelease = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse GitHub response: {e}"))?;

    let latest_version = release.tag_name.trim_start_matches('v').to_string();
    let current_clean = current_version.trim_start_matches('v');

    let has_update = latest_version != current_clean;

    Ok((latest_version, has_update))
}
