use crate::services::semantic_version::SemanticVersion;
use crate::standard_error::StandardError;
use log::error;
use regex::Regex;
use serde::Deserialize;
use tauri_plugin_http::reqwest::Client;

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
}

pub async fn fetch_latest_release_version() -> SemanticVersion {
    let client = Client::new();
    match check_for_update_with_client(
        &client,
        "https://api.github.com/repos/brand-it/reelix/releases/latest",
    )
    .await
    {
        Ok(version) => version,
        Err(e) => {
            error!("Failed to check for latest version: {e}");
            SemanticVersion::none()
        }
    }
}

async fn check_for_update_with_client(
    client: &Client,
    api_url: &str,
) -> Result<SemanticVersion, StandardError> {
    let response = client
        .get(api_url)
        .header("User-Agent", "Reelix")
        .send()
        .await
        .map_err(|e| StandardError::new("Failed to fetch latest release".into(), e.to_string()))?;

    if !response.status().is_success() {
        return Err(StandardError::new(
            "GitHub API Error".into(),
            response.status().to_string(),
        ));
    }

    let release: GitHubRelease = response
        .json()
        .await
        .map_err(|e| StandardError::new("Failed to parse GitHub response".into(), e.to_string()))?;

    Ok(extract_version(&release.tag_name))
}

fn extract_version(version_string: &str) -> SemanticVersion {
    let re = match Regex::new(r"\d+\.\d+\.\d+") {
        Ok(r) => r,
        Err(_) => return SemanticVersion::none(),
    };
    match re.find(version_string) {
        Some(m) => {
            let version_str = m.as_str();
            match SemanticVersion::parse(version_str) {
                Ok(version) => version,
                Err(_) => SemanticVersion::none(),
            }
        }
        None => SemanticVersion::none(),
    }
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
    fn test_extract_version_with_v_prefix() {
        let result = extract_version("v1.0.0");
        assert_eq!(result.to_string(), "1.0.0");
    }

    #[test]
    fn test_extract_version_with_tag_prefix() {
        let result = extract_version("reelix-v0.34.1");
        assert_eq!(result.to_string(), "0.34.1");
    }

    #[test]
    fn test_extract_version_plain_version() {
        let result = extract_version("1.5.0");
        assert_eq!(result.to_string(), "1.5.0");
    }

    #[test]
    fn test_extract_version_invalid() {
        let result = extract_version("no-version-here");
        assert_eq!(result, SemanticVersion::none());
    }

    #[tokio::test]
    async fn test_check_for_update_with_client_newer_version() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        let mock_url = format!("{}/test", mock_server.uri());

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"tag_name": "reelix-v0.36.0"})),
            )
            .mount(&mock_server)
            .await;

        let client = Client::new();

        let result = check_for_update_with_client(&client, &mock_url).await;

        assert!(result.is_ok());
        let latest_version = result.expect("expected Ok result");
        assert_eq!(latest_version, SemanticVersion::parse("0.36.0").unwrap());
    }

    #[tokio::test]
    async fn test_check_for_update_with_client_same_version() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        let mock_url = format!("{}/test", mock_server.uri());

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"tag_name": "v0.35.1"})),
            )
            .mount(&mock_server)
            .await;

        let client = Client::new();

        let result = check_for_update_with_client(&client, &mock_url).await;

        assert!(result.is_ok());
        let latest_version = result.expect("expected Ok result");
        assert_eq!(latest_version, SemanticVersion::parse("0.35.1").unwrap());
    }

    #[tokio::test]
    async fn test_check_for_update_with_client_api_error() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        let mock_url = format!("{}/test", mock_server.uri());

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        let client = Client::new();

        let result = check_for_update_with_client(&client, &mock_url).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("404"));
    }

    #[tokio::test]
    async fn test_check_for_update_with_client_invalid_json() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        let mock_url = format!("{}/test", mock_server.uri());

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(200).set_body_string("invalid json"))
            .mount(&mock_server)
            .await;

        let client = Client::new();

        let result = check_for_update_with_client(&client, &mock_url).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to parse"));
    }

    #[tokio::test]
    async fn test_check_for_update_with_client_invalid_version_format() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        let mock_url = format!("{}/test", mock_server.uri());

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"tag_name": "no-version-here"})),
            )
            .mount(&mock_server)
            .await;

        let client = Client::new();

        let result = check_for_update_with_client(&client, &mock_url).await;

        assert!(result.is_ok());
        assert_eq!(result.expect("expected Ok result"), SemanticVersion::none());
    }
}
