//! Get all active upload sessions.
//!
//! GraphQL query: `uploadSessions`

use log::debug;
use serde::Deserialize;

use super::error::Error;
use super::{ReelixManager, UploadSession};

const GQL_QUERY: &str = r#"
    query {
        uploadSessions {
            id
            filename
            uploadLength
            uploadOffset
            status
        }
    }
"#;

/// Get all active upload sessions
pub async fn execute(manager: &ReelixManager) -> Result<Vec<UploadSession>, Error> {
    let url = format!("{}/graphql", manager.host);

    let body = serde_json::json!({
        "query": GQL_QUERY
    });

    let resp = manager
        .async_client
        .post(&url)
        .header("Authorization", format!("Bearer {}", manager.token))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| Error::new(format!("GraphQL upload sessions request failed: {e}")))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(Error::new(format!(
            "GraphQL upload sessions failed with status {status}: {body}"
        )));
    }

    #[derive(Deserialize)]
    struct Wrapper {
        data: SessionsData,
    }

    #[derive(Deserialize)]
    struct SessionsData {
        #[serde(rename = "uploadSessions")]
        upload_sessions: Vec<UploadSession>,
    }

    let raw_body = resp
        .text()
        .await
        .map_err(|e| Error::new(format!("Failed to read response body: {e}")))?;

    debug!("get_active_uploads raw response: {raw_body}");

    let wrapper: Wrapper = serde_json::from_str(&raw_body).map_err(|e| {
        Error::new(format!(
            "Failed to parse upload sessions response: {e}. Raw: {raw_body}"
        ))
    })?;
    Ok(wrapper.data.upload_sessions)
}
