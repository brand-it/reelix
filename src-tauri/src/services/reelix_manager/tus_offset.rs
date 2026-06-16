//! Get the current offset of a tus upload.
//!
//! API: HEAD /files/:id with tus headers
//! Returns Upload-Offset header indicating bytes already uploaded

use super::error::Error;
use super::ReelixManager;

/// Get the current offset of an upload (how many bytes have been uploaded)
pub async fn execute(manager: &ReelixManager, upload_id: &str) -> Result<u64, Error> {
    let resp = manager
        .async_request()
        .head(&format!("/files/{upload_id}"))
        .header("Tus-Resumable", "1.0.0".into())
        .send()
        .await?;

    let upload_offset = resp
        .headers
        .get("Upload-Offset")
        .and_then(|s| s.parse::<u64>().ok())
        .ok_or_else(|| Error::new("No Upload-Offset header in tus HEAD response"))?;

    Ok(upload_offset)
}
