use crate::state::upload_state::{PendingUpload, UploadQueue, UploadType};
use log::debug;
use serde_json::json;
use std::sync::Arc;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

/// Manages the upload state using Tauri's store mechanism
/// This keeps the queue in memory and persists to "uploads.json"
pub struct UploadedState {
    pub queue: Arc<UploadQueue>,
}

impl UploadedState {
    /// Clone the Arc for the UploadedState
    #[allow(dead_code)]
    pub fn clone_arc(state: &Self) -> Arc<Self> {
        Arc::new(UploadedState {
            queue: Arc::clone(&state.queue),
        })
    }

    /// Create a new UploadedState and load pending uploads from store
    pub fn new(app_handle: &AppHandle) -> Result<Self, String> {
        let store = app_handle
            .store("uploads.json")
            .map_err(|e| format!("Failed to load uploads.json store: {e}"))?;

        // Load pending uploads from store
        let pending_uploads: Vec<PendingUpload> = if let Some(value) = store.get("pending") {
            serde_json::from_value(value.clone()).unwrap_or_default()
        } else {
            Vec::new()
        };

        let count = pending_uploads.len();
        let queue = Arc::new(UploadQueue::from_pending(pending_uploads));
        store.close_resource();

        if count > 0 {
            debug!("Loaded {count} pending uploads from store");
        }

        Ok(UploadedState { queue })
    }

    /// Add a video to the upload queue and persist to store
    pub fn add_upload(
        &self,
        app_handle: &AppHandle,
        video_path: String,
        upload_type: UploadType,
    ) -> Result<(), String> {
        // Add to queue
        self.queue.add(video_path.clone(), upload_type)?;

        // Persist to store
        self.persist_to_store(app_handle)?;
        debug!(
            "Added {video_path} to upload queue and persisted to store"
        );

        Ok(())
    }

    /// Remove a video from the upload queue and persist to store
    pub fn remove_upload(&self, app_handle: &AppHandle, video_path: &str) -> Result<(), String> {
        // Remove from queue
        self.queue.remove(video_path)?;

        // Persist to store
        self.persist_to_store(app_handle)?;
        debug!(
            "Removed {video_path} from upload queue and persisted to store"
        );

        Ok(())
    }

    /// Get all pending uploads
    pub fn get_pending(&self) -> Vec<PendingUpload> {
        self.queue.get_pending()
    }

    /// Check if there are any pending uploads
    #[allow(dead_code)]
    pub fn has_pending(&self) -> bool {
        self.queue.has_pending()
    }

    /// Get the count of pending uploads
    #[allow(dead_code)]
    pub fn count(&self) -> usize {
        self.queue.count()
    }

    /// Persist the current queue to the store
    fn persist_to_store(&self, app_handle: &AppHandle) -> Result<(), String> {
        let store = app_handle
            .store("uploads.json")
            .map_err(|e| format!("Failed to open uploads.json store: {e}"))?;

        let pending = self.queue.get_pending();
        store.set("pending", json!(pending));

        store
            .save()
            .map_err(|e| format!("Failed to save uploads.json store: {e}"))?;

        store.close_resource();
        Ok(())
    }

    /// Clear all pending uploads and persist to store
    #[allow(dead_code)]
    pub fn clear(&self, app_handle: &AppHandle) -> Result<(), String> {
        self.queue.clear()?;
        self.persist_to_store(app_handle)?;
        debug!("Cleared all pending uploads and persisted to store");
        Ok(())
    }
}
