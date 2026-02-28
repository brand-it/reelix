use crate::services;
use crate::state::background_process_state::BackgroundProcessState;
use crate::state::job_state::{emit_progress, JobStatus, JobType};
use crate::state::title_video::{self, TitleVideo};
use crate::state::upload_state::{PendingUpload, UploadType};
use crate::state::uploaded_state::UploadedState;
use crate::state::AppState;
use crate::the_movie_db;
use log::{error, info, warn};
use std::path::Path;
use std::sync::{Arc, RwLock};
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

/// Resume uploads on boot - processes pending uploads sequentially
/// This function runs asynchronously and does not block the boot process
pub async fn resume_pending_uploads(app_handle: AppHandle) {
    info!("Starting upload recovery process");

    // Get the UploadedState
    let uploaded_state = match app_handle.try_state::<UploadedState>() {
        Some(state) => {
            let state_ref = state.inner();
            Arc::new(UploadedState {
                queue: Arc::clone(&state_ref.queue),
            })
        }
        None => {
            error!("Failed to get UploadedState for recovery");
            return;
        }
    };

    // Get pending uploads
    let pending_uploads = uploaded_state.get_pending();

    if pending_uploads.is_empty() {
        info!("No pending uploads to resume");
        return;
    }

    info!("Found {} pending uploads to resume", pending_uploads.len());

    // Process each upload sequentially (one at a time)
    for pending_upload in pending_uploads {
        // Check if file still exists before attempting upload
        let path = Path::new(&pending_upload.video_path);
        if !path.exists() {
            warn!("Skipping non-existent file: {}", pending_upload.video_path);
            // Remove from queue
            if let Err(e) = uploaded_state.remove_upload(&app_handle, &pending_upload.video_path) {
                error!("Failed to remove non-existent file from queue: {e}");
            }
            continue;
        }

        info!("Processing upload: {}", pending_upload.video_path);

        // Try to reconstruct TitleVideo with TMDB metadata (blocking TMDB calls offloaded)
        match reconstruct_title_video_with_tmdb(&pending_upload, &app_handle).await {
            Ok(title_video) => {
                // Upload the video using the standard upload function
                upload_video(
                    &app_handle,
                    &pending_upload.video_path,
                    &title_video,
                    &uploaded_state,
                )
                .await;
            }
            Err(e) => {
                error!(
                    "Failed to reconstruct video metadata for {}: {}",
                    pending_upload.video_path, e
                );
            }
        }
    }

    info!("Upload recovery process completed");
}

/// Reconstruct a TitleVideo from a pending upload using TMDB API
async fn reconstruct_title_video_with_tmdb(
    pending_upload: &PendingUpload,
    app_handle: &AppHandle,
) -> Result<Arc<RwLock<TitleVideo>>, String> {
    let pending_upload = pending_upload.clone();
    let app_handle = app_handle.clone();

    tokio::task::spawn_blocking(move || {
        reconstruct_title_video_with_tmdb_blocking(&pending_upload, &app_handle)
    })
    .await
    .map_err(|e| format!("TMDB reconstruction task failed: {e}"))?
}

fn reconstruct_title_video_with_tmdb_blocking(
    pending_upload: &PendingUpload,
    app_handle: &AppHandle,
) -> Result<Arc<RwLock<TitleVideo>>, String> {
    let path = Path::new(&pending_upload.video_path);

    match pending_upload.upload_type {
        UploadType::Movie => reconstruct_movie_with_tmdb_blocking(path, app_handle),
        UploadType::TvShow => reconstruct_tv_with_tmdb_blocking(path, app_handle),
    }
}

/// Reconstruct movie metadata using TMDB API
fn reconstruct_movie_with_tmdb_blocking(
    path: &Path,
    app_handle: &AppHandle,
) -> Result<Arc<RwLock<TitleVideo>>, String> {
    let filename = path
        .file_stem()
        .ok_or_else(|| "No filename found".to_string())?
        .to_string_lossy();

    // Parse movie name and year from filename
    let (title, year_str) = parse_movie_filename(&filename)?;
    let year: u32 = year_str
        .parse()
        .map_err(|_| "Invalid year format".to_string())?;

    // Search TMDB for the movie
    let state = app_handle.state::<AppState>();
    let api_key = state.lock_the_movie_db_key().to_string();

    if api_key.is_empty() {
        return Err("TMDB API key not configured".to_string());
    }

    let movie_db = the_movie_db::TheMovieDb::new(&api_key, "en-US");

    // Search for the movie using dedicated search_movie endpoint with year filter
    let search_results = movie_db
        .search_movie(&title, Some(year), 1)
        .map_err(|e| format!("TMDB movie search failed: {}", e.message))?;

    // Get the first result (should be the best match)
    let movie_result = search_results
        .results
        .first()
        .ok_or_else(|| format!("No TMDB movie match found for {title} ({year}"))?;

    // Get full movie details
    let movie_id = movie_result.id;
    let movie_response = movie_db
        .movie(movie_id)
        .map_err(|e| format!("Failed to get movie details: {}", e.message))?;

    // Parse edition and part from filename if present
    let (edition, part) = parse_edition_and_part(&filename);

    let movie = title_video::MoviePartEdition {
        movie: movie_response,
        part,
        edition,
    };

    // Create TitleInfo with the original filename
    let title_info = crate::models::title_info::TitleInfo {
        id: 0,
        name: None,
        chapter_count: None,
        duration: None,
        size: None,
        bytes: None,
        angle: None,
        source_file_name: None,
        segment_count: None,
        segment_map: None,
        filename: Some(path.file_name().unwrap().to_string_lossy().to_string()),
        lang: None,
        language: None,
        description: None,
    };

    let video = title_video::Video::Movie(Box::new(movie));
    let title_video = title_video::TitleVideo {
        id: title_video::TitleVideoId::new(),
        title: Some(title_info),
        video,
    };

    info!("Successfully reconstructed metadata for {title} using TMDB");
    Ok(Arc::new(RwLock::new(title_video)))
}

/// Reconstruct TV show metadata using TMDB API
fn reconstruct_tv_with_tmdb_blocking(
    path: &Path,
    app_handle: &AppHandle,
) -> Result<Arc<RwLock<TitleVideo>>, String> {
    // Parse TV show information from path
    // Expected format: /path/to/TV Shows/ShowName (Year)/Season XX/ShowName - SXXEXX - Episode.mkv
    let (show_name, year_str, season_number, episode_number) = parse_tv_path(path)?;
    let year: u32 = year_str
        .parse()
        .map_err(|_| "Invalid year format".to_string())?;

    info!("Reconstructing TV show: {show_name} ({year}), S{season_number:02}E{episode_number:02}");

    // Search TMDB for the TV show
    let state = app_handle.state::<AppState>();
    let api_key = state.lock_the_movie_db_key().to_string();

    if api_key.is_empty() {
        return Err("TMDB API key not configured".to_string());
    }

    let movie_db = the_movie_db::TheMovieDb::new(&api_key, "en-US");

    // Search for the TV show using dedicated search_tv endpoint with year filter
    let search_results = movie_db
        .search_tv(&show_name, Some(year), 1)
        .map_err(|e| format!("TMDB TV search failed: {}", e.message))?;

    // Get the first result (should be the best match)
    let tv_result = search_results
        .results
        .first()
        .ok_or_else(|| format!("No TMDB TV show found for {show_name} ({year})"))?;

    // Get full TV show details
    let tv_id = tv_result.id;
    let tv_response = movie_db
        .tv(tv_id)
        .map_err(|e| format!("Failed to get TV show details: {}", e.message))?;

    // Verify the season exists in the TV show
    let season_exists = tv_response
        .seasons
        .iter()
        .any(|s| s.season_number == season_number);

    if !season_exists {
        return Err(format!(
            "Season {season_number} not found in TV show {show_name}"
        ));
    }

    // Get season details with episodes
    let season_response = movie_db
        .season(tv_id, season_number)
        .map_err(|e| format!("Failed to get season details: {}", e.message))?;

    // Find the specific episode
    let episode = season_response
        .episodes
        .iter()
        .find(|e| e.episode_number == episode_number)
        .ok_or_else(|| {
            format!("Episode {episode_number} not found in Season {season_number} of {show_name}")
        })?
        .clone();

    // Parse part information from filename if present
    let filename = path
        .file_stem()
        .ok_or_else(|| "No filename found".to_string())?
        .to_string_lossy();
    let part = parse_tv_part(&filename).unwrap_or(1);

    let tv_show = title_video::TvSeasonEpisode {
        episode,
        season: season_response,
        tv: tv_response,
        part,
    };

    // Create TitleInfo with the original filename
    let title_info = crate::models::title_info::TitleInfo {
        id: 0,
        name: None,
        chapter_count: None,
        duration: None,
        size: None,
        bytes: None,
        angle: None,
        source_file_name: None,
        segment_count: None,
        segment_map: None,
        filename: Some(path.file_name().unwrap().to_string_lossy().to_string()),
        lang: None,
        language: None,
        description: None,
    };

    let video = title_video::Video::Tv(Box::new(tv_show));
    let title_video = title_video::TitleVideo {
        id: title_video::TitleVideoId::new(),
        title: Some(title_info),
        video,
    };

    info!(
        "Successfully reconstructed TV metadata for {show_name} S{season_number:02}E{episode_number:02} using TMDB"
    );
    Ok(Arc::new(RwLock::new(title_video)))
}

/// Parse TV show path to extract show name, year, season, and episode
/// Expected format: /path/to/ShowName (Year)/Season XX/ShowName - SXXEXX - Episode.mkv
fn parse_tv_path(path: &Path) -> Result<(String, String, u32, u32), String> {
    // Get the filename
    let filename = path
        .file_stem()
        .ok_or_else(|| "No filename found".to_string())?
        .to_string_lossy();

    // Parse season and episode from filename using SXXEXX pattern
    let (_, season_number, episode_number) = parse_tv_filename(&filename)?;

    // Get the parent directory (should be Season XX)
    let season_dir = path
        .parent()
        .ok_or_else(|| "No parent directory found".to_string())?;

    // Get the show directory (should be ShowName (Year))
    let show_dir = season_dir
        .parent()
        .ok_or_else(|| "No show directory found".to_string())?;

    // Extract show name and year from the show directory name
    let show_dir_name = show_dir
        .file_name()
        .ok_or_else(|| "No show directory name found".to_string())?
        .to_string_lossy();

    // Parse show name and year from directory name: "ShowName (Year)"
    let (show_name, year) = parse_show_name_and_year(&show_dir_name)?;

    Ok((show_name, year, season_number, episode_number))
}

/// Parse show name and year from directory name
/// Expected format: "ShowName (Year)"
fn parse_show_name_and_year(dir_name: &str) -> Result<(String, String), String> {
    if let Some(year_start) = dir_name.rfind('(') {
        if let Some(year_end) = dir_name.rfind(')') {
            if year_end > year_start {
                let show_name = dir_name[..year_start].trim().to_string();
                let year = dir_name[year_start + 1..year_end].trim().to_string();

                // Validate year is 4 digits
                if year.len() == 4 && year.chars().all(|c| c.is_ascii_digit()) {
                    return Ok((show_name, year));
                }
            }
        }
    }

    Err(format!(
        "Could not parse show name and year from: {dir_name}"
    ))
}

/// Parse part information from TV filename
/// Returns part number if present (e.g., -pt1, -pt2)
fn parse_tv_part(filename: &str) -> Option<u16> {
    // Look for -ptX pattern
    if let Some(pos) = filename.rfind("-pt") {
        let after_pt = &filename[pos + 3..];
        // Extract digits after -pt
        let digits: String = after_pt
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if let Ok(part_num) = digits.parse::<u16>() {
            return Some(part_num);
        }
    }
    None
}

/// Parse edition and part information from filename
/// Returns (edition, part)
fn parse_edition_and_part(filename: &str) -> (Option<String>, Option<u16>) {
    let mut edition = None;
    let mut part = None;

    // Look for {edition-XXX} pattern
    if let Some(start) = filename.find("{edition-") {
        if let Some(end) = filename[start..].find('}') {
            let edition_text = &filename[start + 9..start + end];
            edition = Some(edition_text.to_string());
        }
    }

    // Look for -ptX pattern
    if let Some(pos) = filename.rfind("-pt") {
        let after_pt = &filename[pos + 3..];
        // Extract digits after -pt
        let digits: String = after_pt
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if let Ok(part_num) = digits.parse::<u16>() {
            part = Some(part_num);
        }
    }

    (edition, part)
}

/// Reconstruct a TitleVideo from a pending upload (fallback without TMDB)
/// This tries to parse the filename and recreate the necessary metadata
#[allow(dead_code)]
async fn reconstruct_title_video(
    pending_upload: &PendingUpload,
    _app_handle: &AppHandle,
) -> Result<Arc<RwLock<TitleVideo>>, String> {
    let path = Path::new(&pending_upload.video_path);

    match pending_upload.upload_type {
        UploadType::Movie => {
            // Try to parse movie information from the path
            // Expected format: /path/to/Movies/MovieName (Year)/MovieName (Year).ext
            reconstruct_movie_video(path)
        }
        UploadType::TvShow => {
            // Try to parse TV show information from the path
            // Expected format: /path/to/TV Shows/ShowName/Season XX/ShowName - SXXEXX - Episode.ext
            reconstruct_tv_video(path)
        }
    }
}

/// Reconstruct movie video metadata from file path
#[allow(dead_code)]
fn reconstruct_movie_video(path: &Path) -> Result<Arc<RwLock<TitleVideo>>, String> {
    let filename = path
        .file_stem()
        .ok_or_else(|| "No filename found".to_string())?
        .to_string_lossy();

    // Parse movie name and year from filename
    // Expected format: "MovieName (Year)"
    let (title, year) = parse_movie_filename(&filename)?;

    // We need to create a minimal MovieResponse for upload
    // Since we're only reconstructing for upload, we don't need full metadata
    let movie_response = the_movie_db::MovieResponse {
        adult: false,
        backdrop_path: None,
        genres: vec![],
        homepage: String::new(),
        id: 0,
        imdb_id: String::new(),
        origin_country: vec![],
        original_language: String::new(),
        original_title: title.clone(),
        overview: String::new(),
        popularity: 0.0,
        poster_path: None,
        release_date: Some(format!("{year}-01-01")),
        revenue: 0,
        runtime: 0,
        title: title.clone(),
    };

    let movie = title_video::MoviePartEdition {
        movie: movie_response,
        part: None,
        edition: None,
    };

    // Create a minimal TitleInfo for the title
    let title_info = crate::models::title_info::TitleInfo {
        id: 0,
        name: None,
        chapter_count: None,
        duration: None,
        size: None,
        bytes: None,
        angle: None,
        source_file_name: None,
        segment_count: None,
        segment_map: None,
        filename: Some(path.file_name().unwrap().to_string_lossy().to_string()),
        lang: None,
        language: None,
        description: None,
    };

    let video = title_video::Video::Movie(Box::new(movie));
    let title_video = title_video::TitleVideo {
        id: title_video::TitleVideoId::new(),
        title: Some(title_info),
        video,
    };

    Ok(Arc::new(RwLock::new(title_video)))
}

/// Parse movie filename to extract title and year
#[allow(dead_code)]
fn parse_movie_filename(filename: &str) -> Result<(String, String), String> {
    // Look for pattern: "Title (Year)"
    if let Some(year_start) = filename.rfind('(') {
        if let Some(year_end) = filename.rfind(')') {
            if year_end > year_start {
                let title = filename[..year_start].trim().to_string();
                let year = filename[year_start + 1..year_end].trim().to_string();

                // Validate year is 4 digits
                if year.len() == 4 && year.chars().all(|c| c.is_ascii_digit()) {
                    return Ok((title, year));
                }
            }
        }
    }

    Err(format!(
        "Could not parse movie title and year from: {filename}"
    ))
}

/// Reconstruct TV show video metadata from file path
fn reconstruct_tv_video(path: &Path) -> Result<Arc<RwLock<TitleVideo>>, String> {
    let filename = path
        .file_stem()
        .ok_or_else(|| "No filename found".to_string())?
        .to_string_lossy();

    // Parse TV show information from filename
    // Expected format: "ShowName - SXXEXX - Episode Title"
    let (show_name, season, episode) = parse_tv_filename(&filename)?;

    // Create minimal TV show structures for upload
    let tv_response = the_movie_db::TvResponse {
        adult: false,
        backdrop_path: None,
        created_by: vec![],
        episode_run_time: vec![],
        first_air_date: None,
        genres: vec![],
        homepage: None,
        id: the_movie_db::TvId::from(0),
        in_production: false,
        languages: vec![],
        last_air_date: None,
        last_episode_to_air: None,
        name: show_name.clone(),
        networks: vec![],
        next_episode_to_air: None,
        number_of_episodes: 0,
        number_of_seasons: 0,
        origin_country: vec![],
        original_language: String::new(),
        original_name: show_name.clone(),
        overview: String::new(),
        popularity: 0.0,
        poster_path: None,
        production_companies: vec![],
        production_countries: vec![],
        seasons: vec![],
        spoken_languages: vec![],
        status: String::new(),
        tagline: String::new(),
        type_: String::new(),
        vote_average: 0.0,
        vote_count: 0,
    };

    let season_response = the_movie_db::SeasonResponse {
        _id: String::new(),
        air_date: None,
        episodes: vec![],
        name: format!("Season {season}"),
        overview: String::new(),
        id: 0,
        poster_path: None,
        season_number: season,
        vote_average: 0.0,
    };

    let episode_obj = the_movie_db::SeasonEpisode {
        air_date: None,
        episode_number: episode,
        episode_type: String::new(),
        id: 0,
        name: format!("Episode {episode}"),
        overview: String::new(),
        production_code: None,
        runtime: None,
        season_number: season,
        show_id: 0,
        still_path: None,
        vote_average: 0.0,
        vote_count: 0,
        crew: vec![],
        guest_stars: vec![],
    };

    let tv_show = title_video::TvSeasonEpisode {
        episode: episode_obj,
        season: season_response,
        tv: tv_response,
        part: 1,
    };

    // Create a minimal TitleInfo for the title
    let title_info = crate::models::title_info::TitleInfo {
        id: 0,
        name: None,
        chapter_count: None,
        duration: None,
        size: None,
        bytes: None,
        angle: None,
        source_file_name: None,
        segment_count: None,
        segment_map: None,
        filename: Some(path.file_name().unwrap().to_string_lossy().to_string()),
        lang: None,
        language: None,
        description: None,
    };

    let video = title_video::Video::Tv(Box::new(tv_show));
    let title_video = title_video::TitleVideo {
        id: title_video::TitleVideoId::new(),
        title: Some(title_info),
        video,
    };

    Ok(Arc::new(RwLock::new(title_video)))
}

/// Parse TV show filename to extract show name, season, and episode
#[allow(dead_code)]
fn parse_tv_filename(filename: &str) -> Result<(String, u32, u32), String> {
    // Look for pattern: SXXEXX
    let re = regex::Regex::new(r"[Ss](\d{2})[Ee](\d{2})").map_err(|e| e.to_string())?;

    if let Some(captures) = re.captures(filename) {
        let season: u32 = captures
            .get(1)
            .ok_or_else(|| "No season found".to_string())?
            .as_str()
            .parse()
            .map_err(|e| format!("Invalid season number: {e}"))?;

        let episode: u32 = captures
            .get(2)
            .ok_or_else(|| "No episode found".to_string())?
            .as_str()
            .parse()
            .map_err(|e| format!("Invalid episode number: {e}"))?;

        // Extract show name (everything before the season/episode marker)
        let show_name = if let Some(pos) = filename.find(&captures[0]) {
            filename[..pos]
                .trim()
                .trim_end_matches(" -")
                .trim()
                .to_string()
        } else {
            return Err("Could not extract show name".to_string());
        };

        return Ok((show_name, season, episode));
    }

    Err(format!("Could not parse TV show info from: {filename}"))
}

/// Upload a video file
async fn upload_video(
    app_handle: &AppHandle,
    video_path: &str,
    title_video: &Arc<RwLock<TitleVideo>>,
    uploaded_state: &Arc<UploadedState>,
) {
    let background_process_state = app_handle.state::<BackgroundProcessState>();

    let (job, is_new) = background_process_state.find_or_create_job(
        None,
        &None,
        &JobType::Uploading,
        &JobStatus::Pending,
    );

    job.write()
        .expect("Failed to get job writer")
        .title_videos
        .push(title_video.clone());
    job.write()
        .expect("Failed to get job writer")
        .update_status(JobStatus::Processing);
    job.write().expect("Failed to get job writer").subtitle =
        Some(format!("Resuming upload: {video_path}"));

    if is_new {
        background_process_state.emit_jobs_changed(app_handle);
    }
    job.read()
        .expect("Failed to get job reader")
        .emit_progress_change(app_handle);

    // Use the standard ftp_uploader::upload function
    match services::ftp_uploader::upload(app_handle, &job, title_video).await {
        Ok(_) => {
            info!("Successfully uploaded: {video_path}");
            notify_upload_success(app_handle, video_path);

            job.write()
                .expect("Failed to acquire write lock on job")
                .update_status(JobStatus::Finished);
            emit_progress(app_handle, &job, true);

            // Remove from upload queue on success
            if let Err(e) = uploaded_state.remove_upload(app_handle, video_path) {
                error!("Failed to remove video from upload queue: {e}");
            }

            // Delete the local file after successful upload
            delete_file(video_path);
        }
        Err(e) => {
            error!("Failed to upload {video_path}: {e}");

            job.write()
                .expect("Failed to get job writer")
                .update_status(JobStatus::Error);
            job.write().expect("Failed to get job writer").message = Some(e.clone());
            emit_progress(app_handle, &job, true);

            notify_upload_failure(app_handle, video_path, &e);
            // Keep in upload queue for retry on next boot
        }
    }
}

fn notify_upload_success(app_handle: &AppHandle, file_path: &str) {
    let filename = Path::new(file_path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    app_handle
        .notification()
        .builder()
        .title("Upload Resumed Successfully")
        .body(format!("Uploaded: {filename}"))
        .show()
        .unwrap();
}

fn notify_upload_failure(app_handle: &AppHandle, file_path: &str, error: &str) {
    let filename = Path::new(file_path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    app_handle
        .notification()
        .builder()
        .title("Failed to Resume Upload")
        .body(format!("{filename}: {error}"))
        .show()
        .unwrap();
}

fn delete_file(file_path: &str) {
    if let Err(error) = std::fs::remove_file(file_path) {
        error!("Failed to delete file {file_path}: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_movie_filename() {
        let result = parse_movie_filename("Inception (2010)");
        assert!(result.is_ok());
        let (title, year) = result.unwrap();
        assert_eq!(title, "Inception");
        assert_eq!(year, "2010");
    }

    #[test]
    fn test_parse_movie_filename_with_spaces() {
        let result = parse_movie_filename("The Dark Knight (2008)");
        assert!(result.is_ok());
        let (title, year) = result.unwrap();
        assert_eq!(title, "The Dark Knight");
        assert_eq!(year, "2008");
    }

    #[test]
    fn test_parse_tv_filename() {
        let result = parse_tv_filename("Game of Thrones - S01E01 - Winter is Coming");
        assert!(result.is_ok());
        let (show, season, episode) = result.unwrap();
        assert_eq!(show, "Game of Thrones");
        assert_eq!(season, 1);
        assert_eq!(episode, 1);
    }

    #[test]
    fn test_parse_tv_filename_lowercase() {
        let result = parse_tv_filename("breaking bad - s05e14");
        assert!(result.is_ok());
        let (show, season, episode) = result.unwrap();
        assert_eq!(show, "breaking bad");
        assert_eq!(season, 5);
        assert_eq!(episode, 14);
    }

    #[test]
    fn test_parse_show_name_and_year() {
        let result = parse_show_name_and_year("Game of Thrones (2011)");
        assert!(result.is_ok());
        let (show, year) = result.unwrap();
        assert_eq!(show, "Game of Thrones");
        assert_eq!(year, "2011");
    }

    #[test]
    fn test_parse_show_name_and_year_with_extra_spaces() {
        let result = parse_show_name_and_year("Breaking Bad  (2008)");
        assert!(result.is_ok());
        let (show, year) = result.unwrap();
        assert_eq!(show, "Breaking Bad");
        assert_eq!(year, "2008");
    }

    #[test]
    fn test_parse_edition_and_part() {
        let (edition, part) = parse_edition_and_part("Movie (2020) {edition-Director's Cut}");
        assert_eq!(edition, Some("Director's Cut".to_string()));
        assert_eq!(part, None);

        let (edition, part) = parse_edition_and_part("Movie (2020) -pt1");
        assert_eq!(edition, None);
        assert_eq!(part, Some(1));

        let (edition, part) = parse_edition_and_part("Movie (2020) {edition-Extended} -pt2");
        assert_eq!(edition, Some("Extended".to_string()));
        assert_eq!(part, Some(2));
    }

    #[test]
    fn test_parse_tv_part() {
        assert_eq!(parse_tv_part("Show - S01E01 -pt1"), Some(1));
        assert_eq!(parse_tv_part("Show - S01E01 -pt2"), Some(2));
        assert_eq!(parse_tv_part("Show - S01E01"), None);
    }
}
