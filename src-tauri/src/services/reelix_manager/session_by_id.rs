//! Get a specific upload session by ID.
//!
//! GraphQL query: `uploadSession(id: ID!)`

use log::debug;
use serde::Deserialize;

use super::error::Error;
use super::{ReelixManager, UploadSession};

const GQL_QUERY: &str = r#"
    query GetUploadSession($id: ID!) {
        uploadSession(id: $id) {
            id
            filename
            uploadLength
            uploadOffset
            status
        }
    }
"#;

/// Get a specific upload session by ID
pub async fn execute(manager: &ReelixManager, upload_id: &str) -> Result<Option<UploadSession>, Error> {
    let url = format!("{}/graphql", manager.host);

    let variables = serde_json::json!({
        "id": upload_id
    });

    let body = serde_json::json!({
        "query": GQL_QUERY,
        "variables": variables
    });

    let resp = manager
        .async_client
        .post(&url)
        .header("Authorization", format!("Bearer {}", manager.token))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| Error::new(format!("GraphQL upload session request failed: {e}")))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(Error::new(format!(
            "GraphQL upload session failed with status {status}: {body}"
        )));
    }

    #[derive(Deserialize)]
    struct Wrapper {
        data: SessionData,
    }

    #[derive(Deserialize)]
    struct SessionData {
        upload_session: Option<UploadSession>,
    }

    let raw_body = resp
        .text()
        .await
        .map_err(|e| Error::new(format!("Failed to read response body: {e}")))?;

    debug!("get_upload_session_by_id raw response for {upload_id}: {raw_body}");

    let wrapper: Wrapper = serde_json::from_str(&raw_body).map_err(|e| {
        Error::new(format!(
            "Failed to parse upload session response: {e}. Raw: {raw_body}"
        ))
    })?;
    Ok(wrapper.data.upload_session)
}
