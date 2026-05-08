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
    let resp = manager
        .async_request()
        .patch(&format!("/files/{upload_id}"))
        .header("Tus-Resumable", "1.0.0".into())
        .header("Content-Type", "application/offset+octet-stream".into())
        .header("Upload-Offset", offset.to_string())
        .body(data.to_vec())
        .send()
        .await?;

    let new_offset = resp
        .headers
        .get("Upload-Offset")
        .and_then(|s| s.parse::<u64>().ok())
        .ok_or_else(|| Error::new("No Upload-Offset header in tus PATCH response"))?;

    Ok(new_offset)
}
