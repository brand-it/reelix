/// Upload information extraction service.
///
/// This module provides functionality to extract and prepare upload metadata
/// from ripped title videos before they are queued for upload to the Reelix Manager.
///
/// # Architecture
///
/// The upload flow works as follows:
/// 1. A disc is ripped via `rip_movie` command
/// 2. After ripping completes, `spawn_upload` is called for each title_video
/// 3. `extract_upload_info` gathers all necessary metadata from the ripped video
/// 4. The video is added to the persistent upload queue
/// 5. `tus_uploader` handles the actual upload to Reelix Manager
///
/// # Data Sources
///
/// This function reads from three sources:
/// - `UploadedState`: The persistent upload queue state
/// - `TitleVideo`: Contains video metadata (path, type, TMDB info)
/// - `Job`: The rip job to check for multi-part titles
///
/// # Example Usage
///
/// The `extract_upload_info` function is called from `spawn_upload` with an
/// `AppHandle`, `TitleVideo`, and `Job` reference. It returns Option<UploadInfo>
/// containing all metadata needed to queue the upload to the Reelix Manager
/// via `tus_uploader`.
///
/// # Failure Cases
///
/// Returns `None` when:
/// - `UploadedState` is not available in the app handle (should never happen in normal operation)
///
/// # Thread Safety
///
/// All inputs are wrapped in `Arc<RwLock<>>` for thread-safe access across the
/// Tauri async runtime. The function acquires read locks as needed.
use crate::state::job_state::Job;
use crate::state::title_video::{TitleVideo, Video};
use crate::state::uploaded_state::UploadedState;
use crate::state::AppState;
use log::error;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tauri::{AppHandle, Manager};

/// Upload information extracted from a title video.
///
/// This struct contains all the metadata needed to queue a video for upload
/// to the Reelix Manager. It is created once per video after ripping completes
/// and is used to:
/// - Add the video to the persistent upload queue
/// - Track the upload progress
/// - Finalize the upload with TMDB metadata
///
/// # Fields
///
/// - `uploaded_state`: Reference to the global upload queue state
/// - `path`: Full filesystem path to the ripped MKV file
/// - `upload_type`: Whether this is a movie or TV show upload
/// - `upload_id`: Existing upload ID for resuming (always `None` for new uploads)
/// - `tmdb_id`: The MovieDB/Reelix Manager ID for the movie or TV show
/// - `season_number`: Season number for TV episodes (`None` for movies)
/// - `episode_number`: Episode number for TV episodes (`None` for movies)
pub struct UploadInfo {
    pub uploaded_state: UploadedState,
    pub path: PathBuf,
    pub upload_type: crate::state::upload_state::UploadType,
    pub upload_id: Option<String>,
    pub tmdb_id: Option<u32>,
    pub season_number: Option<u32>,
    pub episode_number: Option<u32>,
}

/// Extract upload preparation data from a title video.
///
/// This is the primary entry point for preparing a ripped video for upload.
/// It consolidates all the metadata needed from various sources into a single
/// `UploadInfo` struct.
///
/// # Arguments
///
/// * `app_handle` - Tauri app handle for accessing global state
/// * `title_video` - The ripped title video containing metadata
/// * `rip_job` - The rip job used to check for multi-part titles
///
/// # Returns
///
/// * `Some(UploadInfo)` - Successfully extracted all upload metadata
/// * `None` - Failed to access `UploadedState` (logs error)
///
/// # Locking Behavior
///
/// This function acquires multiple read locks on the input `Arc<RwLock<>>` values:
/// - `rip_job`: To check if the title has multiple parts
/// - `title_video`: To get the video path and metadata
///
/// Locks are held for minimal time and released before returning.
pub fn extract_upload_info(
    app_handle: &AppHandle,
    title_video: &Arc<RwLock<TitleVideo>>,
    rip_job: &Arc<RwLock<Job>>,
) -> Option<UploadInfo> {
    let uploaded_state = match app_handle.try_state::<UploadedState>() {
        Some(state) => {
            let state_ref = state.inner();
            UploadedState {
                queue: Arc::clone(&state_ref.queue),
            }
        }
        None => {
            error!("Failed to get UploadedState");
            return None;
        }
    };

    let multiple_parts = rip_job
        .read()
        .expect("Failed to get rip_job reader")
        .has_multiple_parts(
            &title_video
                .read()
                .expect("To get title_video read lock for multiple_parts check"),
        );

    let state: tauri::State<AppState> = app_handle.state::<AppState>();
    let path = title_video
        .read()
        .expect("Failed to get title_video reader")
        .video_path(&state, multiple_parts);

    let upload_type = match &title_video.read().expect("Failed to get title_video reader").video {
        Video::Movie(_) => crate::state::upload_state::UploadType::Movie,
        Video::Tv(_) => crate::state::upload_state::UploadType::TvShow,
    };

    // Extract TMDB metadata for upload tracking
    let (upload_id, tmdb_id, season_number, episode_number) = {
        let video_guard = title_video
            .read()
            .expect("Failed to get title_video reader");
        match &video_guard.video {
            Video::Movie(movie) => (
                None, // No upload_id for new uploads
                Some(movie.movie.id),
                None, // No season for movies
                None, // No episode for movies
            ),
            Video::Tv(tv) => (
                None, // No upload_id for new uploads
                Some(tv.tv.id.into()),
                Some(tv.season.season_number),
                Some(tv.episode.episode_number),
            ),
        }
    };

    Some(UploadInfo {
        uploaded_state,
        path,
        upload_type,
        upload_id,
        tmdb_id,
        season_number,
        episode_number,
    })
}
