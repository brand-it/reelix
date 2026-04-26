//! Upload a chunk of data to an existing tus upload.
//!
//! API: PATCH /files/:id with tus headers
//! - Upload-Offset: current offset in bytes
//! - Content-Type: application/offset+octet-stream
//!
//! Returns Upload-Offset header with new offset

use super::error::Error;
use super::ReelixManager;

/// Upload a chunk of data to an existing tus upload
///
/// Returns the new offset after the upload
pub async fn execute(
    manager: &ReelixManager,
    upload_id: &str,
    offset: u64,
    data: &[u8],
) -> Result<u64, Error> {
    let url = format!("{}/files/{upload_id}", manager.host);

    let resp = manager
        .async_client
        .patch(&url)
        .header("Authorization", format!("Bearer {}", manager.token))
        .header("Tus-Resumable", "1.0.0")
        .header("Content-Type", "application/offset+octet-stream")
        .header("Upload-Offset", offset.to_string())
        .body(data.to_vec())
        .send()
        .await
        .map_err(|e| Error::new(format!("Tus PATCH request failed: {e}")))?;

    let status = resp.status();
    if status != 204 {
        let body = resp.text().await.unwrap_or_default();
        return Err(Error::new(format!(
            "Tus PATCH failed with status {status}: {body}"
        )));
    }

    let new_offset = resp
        .headers()
        .get("Upload-Offset")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .ok_or_else(|| Error::new("No Upload-Offset header in tus PATCH response"))?;

    Ok(new_offset)
}
