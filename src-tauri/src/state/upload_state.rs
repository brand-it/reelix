use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::{Arc, RwLock};

/// Represents a video file that needs to be uploaded
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PendingUpload {
    pub video_path: String,
    pub upload_type: UploadType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum UploadType {
    Movie,
    TvShow,
}

/// Manages the in-memory queue of pending uploads
/// Persistence is handled via Tauri's store mechanism
#[derive(Clone)]
pub struct UploadQueue {
    pending: Arc<RwLock<HashSet<PendingUpload>>>,
}

impl UploadQueue {
    /// Create a new empty UploadQueue
    pub fn new() -> Self {
        Self {
            pending: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Create from existing pending uploads
    pub fn from_pending(pending: Vec<PendingUpload>) -> Self {
        let queue = Self::new();
        if let Ok(mut guard) = queue.pending.write() {
            for upload in pending {
                guard.insert(upload);
            }
        }
        queue
    }

    /// Add a video to the upload queue
    pub fn add(&self, video_path: String, upload_type: UploadType) -> Result<(), String> {
        let upload = PendingUpload {
            video_path: video_path.clone(),
            upload_type,
        };

        if let Ok(mut guard) = self.pending.write() {
            if guard.insert(upload) {
                debug!("Added {video_path} to upload queue");
            } else {
                debug!("File already in upload queue: {video_path}");
            }
            Ok(())
        } else {
            Err("Failed to acquire write lock on upload queue".to_string())
        }
    }

    /// Remove a video from the upload queue
    pub fn remove(&self, video_path: &str) -> Result<(), String> {
        if let Ok(mut guard) = self.pending.write() {
            let initial_len = guard.len();
            guard.retain(|upload| upload.video_path != video_path);

            if guard.len() < initial_len {
                debug!("Removed {video_path} from upload queue");
            }
            Ok(())
        } else {
            Err("Failed to acquire write lock on upload queue".to_string())
        }
    }

    /// Get all pending uploads as a vector
    pub fn get_pending(&self) -> Vec<PendingUpload> {
        self.pending
            .read()
            .map(|guard| guard.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Check if the queue has any pending uploads
    #[allow(dead_code)]
    pub fn has_pending(&self) -> bool {
        self.pending
            .read()
            .map(|guard| !guard.is_empty())
            .unwrap_or(false)
    }

    /// Get the count of pending uploads
    #[allow(dead_code)]
    pub fn count(&self) -> usize {
        self.pending.read().map(|guard| guard.len()).unwrap_or(0)
    }

    /// Clear all pending uploads
    #[allow(dead_code)]
    pub fn clear(&self) -> Result<(), String> {
        if let Ok(mut guard) = self.pending.write() {
            guard.clear();
            Ok(())
        } else {
            Err("Failed to acquire write lock on upload queue".to_string())
        }
    }
}

impl Default for UploadQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_remove() {
        let queue = UploadQueue::new();

        queue
            .add("test.mkv".to_string(), UploadType::Movie)
            .unwrap();
        assert_eq!(queue.count(), 1);

        queue.remove("test.mkv").unwrap();
        assert_eq!(queue.count(), 0);
    }

    #[test]
    fn test_has_pending() {
        let queue = UploadQueue::new();
        assert!(!queue.has_pending());

        queue
            .add("test.mkv".to_string(), UploadType::Movie)
            .unwrap();
        assert!(queue.has_pending());
    }
}
