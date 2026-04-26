//! Get the current offset of a tus upload.
//!
//! API: HEAD /files/:id with tus headers
//! Returns Upload-Offset header indicating bytes already uploaded

use super::error::Error;
use super::ReelixManager;

/// Get the current offset of an upload (how many bytes have been uploaded)
pub async fn execute(manager: &ReelixManager, upload_id: &str) -> Result<u64, Error> {
    let url = format!("{}/files/{upload_id}", manager.host);

    let resp = manager
        .async_client
        .head(&url)
        .header("Authorization", format!("Bearer {}", manager.token))
        .header("Tus-Resumable", "1.0.0")
        .send()
        .await
        .map_err(|e| Error::new(format!("Tus HEAD request failed: {e}")))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(Error::new(format!(
            "Tus HEAD failed with status {status}: {body}"
        )));
    }

    let upload_offset = resp
        .headers()
        .get("Upload-Offset")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .ok_or_else(|| Error::new("No Upload-Offset header in tus HEAD response"))?;

    Ok(upload_offset)
}
