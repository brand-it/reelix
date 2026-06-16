use crate::progress_tracker::Base;
use crate::services::reelix_manager::ReelixManager;
use crate::state::job_state::{emit_progress, Job};
use crate::state::title_video::{TitleVideo, Video};
use crate::state::uploaded_state::UploadedState;
use crate::state::AppState;
use log::{debug, error, info, warn};
use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::sync::{Arc, RwLock};
use tauri::{AppHandle, Manager};

const CHUNK_SIZE: usize = 1024 * 1024; // 1MB chunks for tus uploads

/// Upload a video file using tus protocol to Reelix Manager
pub async fn upload(
    app_handle: &AppHandle,
    job: &Arc<RwLock<Job>>,
    title_video: &Arc<RwLock<TitleVideo>>,
    stored_upload_id: Option<String>,
) -> Result<(), String> {
    info!("tus_uploader::upload called");
    
    let state = app_handle.state::<AppState>();
    let manager = ReelixManager::new(&state);
    
    let multiple_parts = job
        .read()
        .expect("Failed to acquire read lock on job")
        .has_multiple_parts(&title_video.read().unwrap());
    
    let local_file_path = title_video
        .read()
        .unwrap()
        .video_path(&state, multiple_parts);

    if !local_file_path.exists() {
        return Err(format!("File not found: {}", local_file_path.display()));
    }

    // Get file metadata
    let metadata = local_file_path
        .metadata()
        .map_err(|e| format!("Failed to get file metadata: {e}"))?;
    let file_size = metadata.len();
    let filename = local_file_path
        .file_name()
        .ok_or("Failed to get filename")?
        .to_string_lossy()
        .to_string();

    debug!("Starting tus upload of {} ({}) to {}", 
        filename, 
        file_size, 
        state.get_manager_host().unwrap_or_default()
    );

    // Extract TMDB metadata for finalize
    let (tmdb_id, media_type, season_number, episode_number) = {
        let tv_guard = title_video.read().unwrap();
        match &tv_guard.video {
            Video::Movie(movie) => (
                movie.movie.id,
                "movie".to_string(),
                None,
                None,
            ),
            Video::Tv(tv) => (
                tv.tv.id.into(),
                "tv".to_string(),
                Some(tv.season.season_number),
                Some(tv.episode.episode_number),
            ),
        }
    };

    // Check for existing uploads with the same filename
    let upload_id = {
        // If we have a stored upload_id, try to use it first
        if let Some(stored_id) = stored_upload_id {
            info!("Trying to resume with stored upload ID: {stored_id}");
            
            // Query the specific upload session by ID using GraphQL
            match manager.get_upload_session_by_id(&stored_id).await {
                Ok(Some(session)) => {
                    info!("Stored upload ID is valid, current offset: {}/{} bytes", 
                            session.upload_offset, session.upload_length);
                    stored_id
                },
                Ok(None) => {
                    warn!("Stored upload ID not found, searching for active uploads with matching filename");
                    // Fall through to search for active uploads by filename
                    let existing_uploads = match manager.get_active_uploads().await {
                        Ok(uploads) => uploads,
                        Err(e) => {
                            debug!("Failed to fetch active uploads: {}", e.message);
                            Vec::new()
                        }
                    };
                    
                    // Look for an existing upload with matching filename and size, pick the one with highest offset
                    // Only consider uploads with status "uploading" or "ready_to_finalize" (not already finalized)
                    let existing = existing_uploads.iter()
                        .filter(|u| u.filename == filename 
                                && u.upload_length_u64() == file_size
                                && (u.status == "uploading" || u.status == "ready_to_finalize"))
                        .max_by_key(|u| u.upload_offset_u64());
                    
                    if let Some(existing_upload) = existing {
                        info!("Found existing upload for {} with ID {} (offset: {}/{} bytes)", 
                              filename, existing_upload.id, existing_upload.upload_offset_u64(), existing_upload.upload_length_u64());
                        existing_upload.id.clone()
                    } else {
                        // No existing upload found, create a new one
                        info!("Creating new tus upload for {filename} ({file_size} bytes)");
                        manager.create_tus_upload(file_size, &filename)
                            .await
                            .map_err(|e| format!("Failed to create tus upload: {}", e.message))? 
                    }
                },
                Err(e) => {
                    warn!("Failed to query stored upload ID: {}, searching for active uploads", e.message);
                    // Fall back to searching for active uploads
                    let existing_uploads = match manager.get_active_uploads().await {
                        Ok(uploads) => uploads,
                        Err(e) => {
                            debug!("Failed to fetch active uploads: {}", e.message);
                            Vec::new()
                        }
                    };
                    
                    // Look for an existing upload with matching filename and size, pick the one with highest offset
                    // Only consider uploads with status "uploading" or "ready_to_finalize" (not already finalized)
                    let existing = existing_uploads.iter()
                        .filter(|u| u.filename == filename 
                                && u.upload_length_u64() == file_size
                                && (u.status == "uploading" || u.status == "ready_to_finalize"))
                        .max_by_key(|u| u.upload_offset_u64());
                    
                    if let Some(existing_upload) = existing {
                        info!("Found existing upload for {} with ID {} (offset: {}/{} bytes)", 
                              filename, existing_upload.id, existing_upload.upload_offset_u64(), existing_upload.upload_length_u64());
                        existing_upload.id.clone()
                    } else {
                        // No existing upload found, create a new one
                        info!("Creating new tus upload for {filename} ({file_size} bytes)");
                        manager.create_tus_upload(file_size, &filename)
                            .await
                            .map_err(|e| format!("Failed to create tus upload: {}", e.message))? 
                    }
                },
            }
        } else {
            // No stored upload_id, query server for active uploads
            let existing_uploads = match manager.get_active_uploads().await {
                Ok(uploads) => uploads,
                Err(e) => {
                    debug!("Failed to fetch active uploads: {}", e.message);
                    Vec::new()
                }
            };
            
            // Look for an existing upload with matching filename and size, pick the one with highest offset
            // Only consider uploads with status "uploading" or "ready_to_finalize" (not already finalized)
            let existing = existing_uploads.iter()
                .filter(|u| u.filename == filename 
                        && u.upload_length_u64() == file_size
                        && (u.status == "uploading" || u.status == "ready_to_finalize"))
                .max_by_key(|u| u.upload_offset_u64());
            
            if let Some(existing_upload) = existing {
                info!("Found existing upload for {} with ID {} (offset: {}/{} bytes)", 
                      filename, existing_upload.id, existing_upload.upload_offset_u64(), existing_upload.upload_length_u64());
                existing_upload.id.clone()
            } else {
                // No existing upload found, create a new one
                info!("Creating new tus upload for {filename} ({file_size} bytes)");
                manager.create_tus_upload(file_size, &filename)
                    .await
                    .map_err(|e| format!("Failed to create tus upload: {}", e.message))? 
            }
        }
    };
    
    debug!("Using tus upload ID: {upload_id}");
    
    // Persist upload state to store for recovery
    let uploaded_state = app_handle.state::<UploadedState>();
    let upload_type = match &title_video.read().unwrap().video {
        Video::Movie(_) => crate::state::upload_state::UploadType::Movie,
        Video::Tv(_) => crate::state::upload_state::UploadType::TvShow,
    };
    if let Err(e) = uploaded_state.add_upload(
        app_handle,
        local_file_path.to_string_lossy().to_string(),
        upload_type.clone(),
        Some(upload_id.clone()),
        Some(tmdb_id),
        season_number,
        episode_number,
    ) {
        warn!("Failed to persist upload state: {e}");
    }
    // Open file for reading
    let file = File::open(&local_file_path)
        .map_err(|e| format!("Failed to open file: {e}"))?;
    let mut reader = BufReader::new(file);

    // Setup progress tracking
    let tracker = new_tracker();
    job.write()
        .expect("Failed to acquire write lock on job")
        .update_title(&title_video.read().unwrap().clone());
    job.write()
        .expect("Failed to acquire write lock on job")
        .subtitle = Some(format!("Uploading {filename}"));
    job.read()
        .expect("Failed to acquire read lock on job")
        .emit_progress_change(app_handle);

    // Upload in chunks
    let mut buffer = vec![0u8; CHUNK_SIZE];
    let mut offset: u64 = 0;

    // Get current upload offset (works for both new and existing uploads)
    match manager.get_upload_offset(&upload_id).await {
        Ok(resume_offset) => {
            if resume_offset > 0 {
                info!("Resuming upload from offset: {resume_offset}");
                // Set initial progress based on already-uploaded bytes
                let initial_percent = (resume_offset as f64 / file_size as f64) * 100.0;
                tracker.set_progress(initial_percent);
                // Emit initial progress so UI shows correct percentage
                job.write()
                    .expect("Failed to acquire write lock on job")
                    .update_progress(&tracker);
                emit_progress(app_handle, job, false);
            } else {
                debug!("Starting fresh upload from offset 0");
            }
            offset = resume_offset;
            
            reader.seek(std::io::SeekFrom::Start(offset))
                .map_err(|e| format!("Failed to seek to offset {offset}: {e}"))?;
        }
        Err(e) => {
            warn!("Failed to get upload offset: {}, starting from 0", e.message);
        }
    }

    loop {
        let bytes_read = reader
            .read(&mut buffer)
            .map_err(|e| format!("Failed to read file: {e}"))?;

        if bytes_read == 0 {
            break;
        }

        // Upload chunk
        let new_offset = manager.upload_chunk(
            &upload_id,
            offset,
            &buffer[..bytes_read],
        )
        .await
        .map_err(|e| format!("Failed to upload chunk: {}", e.message))?;

        offset = new_offset;

        // Update progress
        let percent = (offset as f64 / file_size as f64) * 100.0;
        tracker.set_progress(percent);
        
        job.write()
            .expect("Failed to acquire write lock on job")
            .update_progress(&tracker);
        emit_progress(app_handle, job, false);
    }
    // Check upload status before proceeding

    match manager.get_upload_session_by_id(&upload_id).await {
        Ok(Some(session)) => {
            debug!("Upload session status: {}, offset: {}/{}", session.status, session.upload_offset_u64(), session.upload_length_u64());
            
            // If already finalized or completed, skip the upload
            if session.status == "finalized" || session.status == "completed" {
                info!("Upload already finalized with status: {}", session.status);
                if let Err(e) = uploaded_state.remove_upload(
                    app_handle,
                    local_file_path.to_string_lossy().as_ref(),
                ) {
                    warn!("Failed to remove completed upload from queue: {e}");
                }
                return Ok(());
            }
            
            // If ready_to_finalize, proceed to finalize
            if session.status == "ready_to_finalize" {
                info!("Upload ready to finalize (offset: {}/{})", session.upload_offset_u64(), session.upload_length_u64());
            }
        }
        Ok(None) => {
            warn!("Upload session not found after upload completed, assuming success");
        }
        Err(e) => {
            warn!("Failed to check upload session status: {}", e.message);
        }
    }

    // Finalize the upload with TMDB metadata
    match manager.finalize_upload(&upload_id, tmdb_id, &media_type, season_number, episode_number).await {
        Ok(_response) => {
            info!("Upload finalized successfully");
            
            // Remove from upload queue on success
            if let Err(e) = uploaded_state.remove_upload(
                app_handle,
                local_file_path.to_string_lossy().as_ref(),
            ) {
                warn!("Failed to remove upload from queue: {e}");
            }
            
            job.write()
                .expect("Failed to acquire write lock on job")
                .update_status(crate::state::job_state::JobStatus::Finished);
            job.read()
                .expect("Failed to acquire read lock on job")
                .emit_progress_change(app_handle);
            
            // Delete the local file after successful upload
            if let Err(e) = std::fs::remove_file(&local_file_path) {
                warn!("Failed to delete local file after upload: {e}");
            }
            
            Ok(())
        }
        Err(e) => {
            error!("Failed to finalize upload: {}", e.message);
            Err(format!("Failed to finalize upload: {}", e.message))
        }
    }
}

fn new_tracker() -> Base {
    Base::new(None)
}
