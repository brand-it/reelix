//! Finalize a tus upload by associating it with TMDB metadata.
//!
//! GraphQL mutation: `finalizeUpload(input: FinalizeUploadInput!)`

use serde::{Deserialize, Serialize};
use serde_json::json;

use super::error::Error;
use super::ReelixManager;

/// Input for the finalizeUpload mutation
#[derive(Debug, Serialize, Deserialize)]
pub struct FinalizeUploadInput {
    #[serde(rename = "uploadId")]
    pub upload_id: String,
    #[serde(rename = "tmdbId")]
    pub tmdb_id: u32,
    #[serde(rename = "mediaType")]
    pub media_type: String,
    #[serde(rename = "seasonNumber")]
    pub season_number: Option<u32>,
    #[serde(rename = "episodeNumber")]
    pub episode_number: Option<u32>,
}

/// Response from the finalizeUpload mutation
#[derive(Debug, Serialize, Deserialize)]
pub struct FinalizeResponse {
    #[serde(rename = "videoBlob")]
    pub video_blob: Option<serde_json::Value>,
    #[serde(rename = "destinationPath")]
    pub destination_path: Option<String>,
    pub errors: Vec<String>,
}

const GQL_MUTATION: &str = r#"
    mutation finalizeUpload($input: FinalizeUploadInput!) {
        finalizeUpload(input: $input) {
            videoBlob {
                id
            }
            destinationPath
            errors
        }
    }
"#;

/// Execute the finalize upload mutation
///
/// Associates a completed tus upload with TMDB metadata, moving it from
/// temporary storage to the media library. Includes retry logic for
/// transient network errors.
pub async fn execute(
    manager: &ReelixManager,
    upload_id: &str,
    tmdb_id: u32,
    media_type: &str,
    season_number: Option<u32>,
    episode_number: Option<u32>,
) -> Result<FinalizeResponse, Error> {
    let input = FinalizeUploadInput {
        upload_id: upload_id.to_string(),
        tmdb_id,
        media_type: media_type.to_string(),
        season_number,
        episode_number,
    };

    let variables = json!({ "input": input });
    let body = json!({
        "query": GQL_MUTATION,
        "variables": variables
    });

    // Retry logic for transient network errors (e.g., incomplete responses)
    let max_retries = 3;
    let mut retry_count = 0;

    loop {
        let resp_result = manager
            .async_request()
            .path("/graphql")
            .json(body.clone())
            .send()
            .await;

        match resp_result {
            Ok(resp) => {

                #[derive(Deserialize)]
                struct Wrapper {
                    data: FinalizeData,
                }

                #[derive(Deserialize)]
                struct FinalizeData {
                    #[serde(rename = "finalizeUpload")]
                    finalize_upload: FinalizeResponse,
                }

                let wrapper: Wrapper = serde_json::from_str(&resp.body).map_err(|e| {
                    Error::new(format!(
                        "Failed to parse finalize response: {e}. Raw: {}",
                        resp.body
                    ))
                })?;

                return Ok(wrapper.data.finalize_upload);
            }
            Err(e) => {
                retry_count += 1;
                if retry_count >= max_retries {
                    return Err(Error::new(format!(
                        "GraphQL finalize request failed after {max_retries} retries: {} (debug: {:?})", e.message, e
                    )));
                }
                let is_retryable = e.message.contains("IncompleteMessage")
                    || e.message.contains("connection reset")
                    || e.message.contains("unexpected eof")
                    || e.message.contains("broken pipe");

                if !is_retryable {
                    return Err(Error::new(format!(
                        "GraphQL finalize request failed: {}", e.message
                    )));
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(100 * retry_count)).await;
            }
        }
    }
}
