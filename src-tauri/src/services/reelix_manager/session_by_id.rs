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
    let variables = serde_json::json!({
        "id": upload_id
    });

    let body = serde_json::json!({
        "query": GQL_QUERY,
        "variables": variables
    });

    let resp = manager
        .async_request()
        .path("/graphql")
        .json(body)
        .send()
        .await?;

    debug!("get_upload_session_by_id raw response for {upload_id}: {}", resp.body);

    #[derive(Deserialize)]
    struct Wrapper {
        data: SessionData,
    }

    #[derive(Deserialize)]
    struct SessionData {
        upload_session: Option<UploadSession>,
    }

    let wrapper: Wrapper = serde_json::from_str(&resp.body).map_err(|e| {
        Error::new(format!(
            "Failed to parse upload session response: {e}. Raw: {}",
            resp.body
        ))
    })?;
    Ok(wrapper.data.upload_session)
}
