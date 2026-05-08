use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::{Arc, RwLock};

/// Represents a video file that needs to be uploaded
/// Represents a video file that needs to be uploaded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingUpload {
    pub video_path: String,
    pub upload_type: UploadType,
    pub upload_id: Option<String>,
    /// TMDB ID for the movie or TV show (used during finalize)
    pub tmdb_id: Option<u32>,
    /// Season number for TV uploads
    pub season_number: Option<u32>,
    /// Episode number for TV uploads
    pub episode_number: Option<u32>,
}

// Custom PartialEq and Eq that only compare video_path
impl PartialEq for PendingUpload {
    fn eq(&self, other: &Self) -> bool {
        self.video_path == other.video_path
    }
}

impl Eq for PendingUpload {}

// Custom Hash that only hashes video_path
impl std::hash::Hash for PendingUpload {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.video_path.hash(state);
    }
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
                // Check if we already have an entry for this video_path
                let video_path = upload.video_path.clone();
                let existing = guard.iter().find(|u| u.video_path == video_path);
                
                if let Some(existing_upload) = existing {
                    // Prefer the entry with a non-null upload_id
                    if existing_upload.upload_id.is_none() && upload.upload_id.is_some() {
                        // Clone the entry to remove before dropping immutable borrow
                        let to_remove = existing_upload.clone();
                        guard.remove(&to_remove);
                        guard.insert(upload);
                        debug!("Updated upload_id for {video_path} from loaded pending uploads");
                    }
                    // Otherwise keep the existing entry
                } else {
                    guard.insert(upload);
                }
            }
        }
        queue
    }

    /// Add a video to the upload queue
    pub fn add(
        &self,
        video_path: String,
        upload_type: UploadType,
        upload_id: Option<String>,
        tmdb_id: Option<u32>,
        season_number: Option<u32>,
        episode_number: Option<u32>,
    ) -> Result<(), String> {
        let mut added = false;
        
        if let Ok(mut guard) = self.pending.write() {
            // Check if entry already exists
            let existing_path = guard.iter().find(|u| u.video_path == video_path)
                .map(|u| u.video_path.clone());
            
            if existing_path.is_some() {
                // Update upload_id/TMDB fields if we have new ones and the existing ones are None
                if upload_id.is_some() || tmdb_id.is_some() || season_number.is_some() || episode_number.is_some() {
                    // Check if we need to update
                    let needs_update = guard.iter().any(|u| {
                        u.video_path == video_path &&
                        (u.upload_id.is_none() || u.tmdb_id.is_none())
                    });
                    
                    if needs_update {
                        // Find and replace the entry with updated fields
                        let to_remove: Vec<_> = guard.iter()
                            .filter(|u| u.video_path == video_path)
                            .cloned()
                            .collect();
                        
                        for old_entry in to_remove {
                            guard.remove(&old_entry);
                            let mut new_entry = old_entry.clone();
                            if let Some(id) = &upload_id {
                                new_entry.upload_id = Some(id.clone());
                            }
                            if let Some(id) = tmdb_id {
                                new_entry.tmdb_id = Some(id);
                            }
                            if let Some(season) = season_number {
                                new_entry.season_number = Some(season);
                            }
                            if let Some(ep) = episode_number {
                                new_entry.episode_number = Some(ep);
                            }
                            guard.insert(new_entry);
                        }
                        debug!("Updated upload metadata for {video_path}");
                    } else {
                        debug!("File already in upload queue: {video_path}");
                    }
                } else {
                    debug!("File already in upload queue: {video_path}");
                }
            } else {
                // New entry
                let upload = PendingUpload {
                    video_path: video_path.clone(),
                    upload_type,
                    upload_id,
                    tmdb_id,
                    season_number,
                    episode_number,
                };
                guard.insert(upload);
                added = true;
                debug!("Added {video_path} to upload queue");
            }
        } else {
            return Err("Failed to acquire write lock on queue".to_string());
        }

        if added {
            // Persist to store only for new entries
        }
        
        Ok(())
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
.add("test.mkv".to_string(), UploadType::Movie, None, None, None, None)
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
.add("test.mkv".to_string(), UploadType::Movie, None, None, None, None)
.unwrap();
        assert!(queue.has_pending());
    }
    
    #[test]
    fn test_tmdb_fields() {
        let queue = UploadQueue::new();
        
        queue
.add(
            "test.mkv".to_string(),
            UploadType::TvShow,
            Some("upload-123".to_string()),
            Some(12345),
            Some(1),
            Some(5),
        )
.unwrap();
        
        let pending = queue.get_pending();
        assert_eq!(pending.len(), 1);
        let upload = &pending[0];
        assert_eq!(upload.tmdb_id, Some(12345));
        assert_eq!(upload.season_number, Some(1));
        assert_eq!(upload.episode_number, Some(5));
    }
}

