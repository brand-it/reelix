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
    let url = format!("{}/graphql", manager.host);

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
            .async_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", manager.token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await;

        match resp_result {
            Ok(resp) => {
                let status = resp.status();
                if !status.is_success() {
                    let body_text = resp.text().await.unwrap_or_default();
                    return Err(Error::new(format!(
                        "GraphQL finalize failed with status {status}: {body_text}"
                    )));
                }

                let raw_body = resp
                    .text()
                    .await
                    .map_err(|e| Error::new(format!("Failed to read response body: {e}")))?;

                #[derive(Deserialize)]
                struct Wrapper {
                    data: FinalizeData,
                }

                #[derive(Deserialize)]
                struct FinalizeData {
                    #[serde(rename = "finalizeUpload")]
                    finalize_upload: FinalizeResponse,
                }

                let wrapper: Wrapper = serde_json::from_str(&raw_body).map_err(|e| {
                    Error::new(format!(
                        "Failed to parse finalize response: {e}. Raw: {raw_body}"
                    ))
                })?;

                return Ok(wrapper.data.finalize_upload);
            }
            Err(e) => {
                retry_count += 1;
                if retry_count >= max_retries {
                    return Err(Error::new(format!(
                        "GraphQL finalize request failed to {url} after {max_retries} retries: {e} (debug: {e:?})"
                    )));
                }

                let error_msg = e.to_string();
                let is_retryable = error_msg.contains("IncompleteMessage")
                    || error_msg.contains("connection reset")
                    || error_msg.contains("unexpected eof")
                    || error_msg.contains("broken pipe");

                if !is_retryable {
                    return Err(Error::new(format!(
                        "GraphQL finalize request failed to {url}: {e}"
                    )));
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(100 * retry_count)).await;
            }
        }
    }
}
