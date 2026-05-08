//! Create a new tus upload session.
//!
//! API: POST /files with tus headers
//! - Upload-Length: file size in bytes
//! - Upload-Metadata: base64-encoded filename
//! - Tus-Resumable: 1.0.0

use base64::Engine;

use super::error::Error;
use super::ReelixManager;

/// Create a new tus upload session
///
/// Returns the upload ID which should be used for subsequent upload operations
pub async fn execute(manager: &ReelixManager, file_size: u64, filename: &str) -> Result<String, Error> {
    let metadata = format!(
        "filename {}",
        base64::engine::general_purpose::STANDARD.encode(filename)
    );

    let resp = manager
        .async_request()
        .post("/files")
        .header("Tus-Resumable", "1.0.0".into())
        .header("Upload-Length", file_size.to_string())
        .header("Upload-Metadata", metadata)
        .send()
        .await?;

    // Extract upload ID from Location header
    let location = resp
        .headers
        .get("Location")
        .cloned()
        .ok_or_else(|| Error::new("No Location header in tus create response"))?;

    // Location is typically /files/:uid, extract the uid
    let upload_id = location
        .strip_prefix("/files/")
        .ok_or_else(|| Error::new(format!("Invalid Location header: {location}")))?;

    Ok(upload_id.to_string())
}
