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
/// GraphQL query: `uploadSessions`
pub async fn execute(manager: &ReelixManager) -> Result<Vec<UploadSession>, Error> {
    let body = serde_json::json!({
        "query": GQL_QUERY
    });

    let resp = manager
        .async_request()
        .path("/graphql")
        .json(body)
        .send()
        .await?;

    debug!("get_active_uploads raw response: {}", resp.body);

    #[derive(Deserialize)]
    struct Wrapper {
        data: SessionsData,
    }

    #[derive(Deserialize)]
    struct SessionsData {
        #[serde(rename = "uploadSessions")]
        upload_sessions: Vec<UploadSession>,
    }

    let wrapper: Wrapper = serde_json::from_str(&resp.body).map_err(|e| {
        Error::new(format!(
            "Failed to parse upload sessions response: {e}. Raw: {}",
            resp.body
        ))
    })?;
    Ok(wrapper.data.upload_sessions)
}
