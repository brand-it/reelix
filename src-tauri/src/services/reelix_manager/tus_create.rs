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
    let url = format!("{}/files", manager.host);

    let resp = manager
        .async_client
        .post(&url)
        .header("Authorization", format!("Bearer {}", manager.token))
        .header("Content-Type", "application/json")
        .header("Tus-Resumable", "1.0.0")
        .header("Upload-Length", file_size.to_string())
        .header(
            "Upload-Metadata",
            format!(
                "filename {}",
                base64::engine::general_purpose::STANDARD.encode(filename)
            ),
        )
        .send()
        .await
        .map_err(|e| Error::new(format!("Tus create request failed: {e}")))?;

    let status = resp.status();
    if status.as_u16() != 201 && status.as_u16() != 200 {
        let body = resp.text().await.unwrap_or_default();
        return Err(Error::new(format!(
            "Tus create failed with status {status}: {body}"
        )));
    }

    // Extract upload ID from Location header
    let location = resp
        .headers()
        .get("Location")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| Error::new("No Location header in tus create response"))?;

    // Location is typically /files/:uid, extract the uid
    let upload_id = location
        .strip_prefix(format!("{}/files/", manager.host).as_str())
        .or_else(|| location.strip_prefix("/files/"))
        .ok_or_else(|| Error::new(format!("Invalid Location header: {location}")))?;

    Ok(upload_id.to_string())
}
