use log::debug;
use regex::Regex;
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
    check_for_update_with_client(
        current_version,
        &client,
        "https://api.github.com/repos/brand-it/reelix/releases/latest",
    )
    .await
}

async fn check_for_update_with_client(
    current_version: &str,
    client: &Client,
    api_url: &str,
) -> Result<(String, bool), String> {
    let response = client
        .get(api_url)
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

    let latest_version = extract_version(&release.tag_name)?;
    let current_clean = extract_version(current_version)?;
    debug!("Current version: {current_clean}, Latest version: {latest_version}");
    let has_update = latest_version != current_clean;

    Ok((latest_version, has_update))
}

fn extract_version(version_string: &str) -> Result<String, String> {
    let re = Regex::new(r"\d+\.\d+\.\d+").map_err(|e| format!("Regex error: {e}"))?;
    re.find(version_string)
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| format!("Could not extract version from: {version_string}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_release_deserialization() {
        let json = r#"{"tag_name": "v1.2.3"}"#;
        let release: GitHubRelease = serde_json::from_str(json).unwrap();

        assert_eq!(release.tag_name, "v1.2.3");
    }

    #[test]
    fn test_version_state_struct_with_update() {
        let state = VersionState {
            current_version: "1.0.0".to_string(),
            latest_version: Some("1.1.0".to_string()),
            has_update: true,
        };

        assert_eq!(state.current_version, "1.0.0");
        assert_eq!(state.latest_version, Some("1.1.0".to_string()));
        assert!(state.has_update);
    }

    #[test]
    fn test_version_state_struct_no_update() {
        let state = VersionState {
            current_version: "1.0.0".to_string(),
            latest_version: None,
            has_update: false,
        };

        assert_eq!(state.current_version, "1.0.0");
        assert_eq!(state.latest_version, None);
        assert!(!state.has_update);
    }

    // Unit tests for the version extraction and comparison logic
    #[test]
    fn test_extract_version_with_v_prefix() {
        let result = extract_version("v1.0.0");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "1.0.0");
    }

    #[test]
    fn test_extract_version_with_tag_prefix() {
        let result = extract_version("reelix-v0.34.1");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "0.34.1");
    }

    #[test]
    fn test_extract_version_plain_version() {
        let result = extract_version("1.5.0");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "1.5.0");
    }

    #[test]
    fn test_extract_version_invalid() {
        let result = extract_version("no-version-here");
        assert!(result.is_err());
    }

    #[test]
    fn test_version_logic_newer_version() {
        let current_version = extract_version("0.35.1").unwrap();
        let latest_tag = extract_version("reelix-v0.36.0").unwrap();

        let has_update = latest_tag != current_version;

        assert_eq!(latest_tag, "0.36.0");
        assert!(has_update);
    }

    #[test]
    fn test_version_logic_same_version() {
        let current_version = extract_version("1.5.0").unwrap();
        let latest_tag = extract_version("v1.5.0").unwrap();

        let has_update = latest_tag != current_version;

        assert_eq!(latest_tag, "1.5.0");
        assert!(!has_update);
    }

    #[test]
    fn test_version_logic_older_version() {
        let current_version = extract_version("0.35.1").unwrap();
        let latest_tag = extract_version("reelix-v0.34.1").unwrap();

        let has_update = latest_tag != current_version;

        assert_eq!(latest_tag, "0.34.1");
        assert!(has_update);
    }

    // Integration tests with mocked HTTP responses
    #[tokio::test]
    async fn test_check_for_update_with_client_newer_version() {
        use wiremock::{Mock, MockServer, ResponseTemplate};
        use wiremock::matchers::{method, path};

        let mock_server = MockServer::start().await;
        let mock_url = format!("{}/test", mock_server.uri());

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"tag_name": "reelix-v0.36.0"}),
            ))
            .mount(&mock_server)
            .await;

        let client = Client::new();
        let current_version = "0.35.1";

        let result = check_for_update_with_client(current_version, &client, &mock_url).await;

        assert!(result.is_ok());
        let (latest_version, has_update) = result.unwrap();
        assert_eq!(latest_version, "0.36.0");
        assert!(has_update);
    }

    #[tokio::test]
    async fn test_check_for_update_with_client_same_version() {
        use wiremock::{Mock, MockServer, ResponseTemplate};
        use wiremock::matchers::{method, path};

        let mock_server = MockServer::start().await;
        let mock_url = format!("{}/test", mock_server.uri());

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"tag_name": "v0.35.1"}),
            ))
            .mount(&mock_server)
            .await;

        let client = Client::new();
        let current_version = "0.35.1";

        let result = check_for_update_with_client(current_version, &client, &mock_url).await;

        assert!(result.is_ok());
        let (latest_version, has_update) = result.unwrap();
        assert_eq!(latest_version, "0.35.1");
        assert!(!has_update);
    }

    #[tokio::test]
    async fn test_check_for_update_with_client_api_error() {
        use wiremock::{Mock, MockServer, ResponseTemplate};
        use wiremock::matchers::{method, path};

        let mock_server = MockServer::start().await;
        let mock_url = format!("{}/test", mock_server.uri());

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        let client = Client::new();
        let current_version = "0.35.1";

        let result = check_for_update_with_client(current_version, &client, &mock_url).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("404"));
    }

    #[tokio::test]
    async fn test_check_for_update_with_client_invalid_json() {
        use wiremock::{Mock, MockServer, ResponseTemplate};
        use wiremock::matchers::{method, path};

        let mock_server = MockServer::start().await;
        let mock_url = format!("{}/test", mock_server.uri());

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(200).set_body_string("invalid json"))
            .mount(&mock_server)
            .await;

        let client = Client::new();
        let current_version = "0.35.1";

        let result = check_for_update_with_client(current_version, &client, &mock_url).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse"));
    }

    #[tokio::test]
    async fn test_check_for_update_with_client_invalid_version_format() {
        use wiremock::{Mock, MockServer, ResponseTemplate};
        use wiremock::matchers::{method, path};

        let mock_server = MockServer::start().await;
        let mock_url = format!("{}/test", mock_server.uri());

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"tag_name": "no-version-here"}),
            ))
            .mount(&mock_server)
            .await;

        let client = Client::new();
        let current_version = "0.35.1";

        let result = check_for_update_with_client(current_version, &client, &mock_url).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Could not extract version"));
    }
}
