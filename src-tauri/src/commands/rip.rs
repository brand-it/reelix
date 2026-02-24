use crate::models::optical_disk_info::{DiskId, OpticalDiskInfo};
use crate::services::ftp_uploader;
use crate::services::plex::find_tv;
use crate::services::{self, disk_manager};
use crate::services::{
    makemkvcon,
    plex::{find_movie, find_season},
};
use crate::standard_error::StandardError;
use crate::state::background_process_state::BackgroundProcessState;
use crate::state::job_state::{emit_progress, Job, JobStatus, JobType};
use crate::state::title_video::{self, TitleVideo, Video};
use crate::state::uploaded_state::UploadedState;
use crate::state::{background_process_state, AppState};
use crate::templates::toast::{Toast, ToastVariant};
use crate::templates::{self};
use log::{debug, error, warn};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tauri::{Emitter, Manager, State};
use tauri_plugin_notification::NotificationExt;
use templates::render_error;

#[tauri::command]
pub fn assign_episode_to_title(
    mvdb_id: u32,
    season_number: u32,
    episode_number: u32,
    title_id: u32,
    part: u16,
    background_process_state: State<'_, background_process_state::BackgroundProcessState>,
    app_handle: tauri::AppHandle,
) -> Result<String, templates::Error> {
    let app_state = app_handle.state::<AppState>();
    let optical_disk = match app_state.selected_disk() {
        Some(disk) => disk,
        None => return render_error("No current selected disk"),
    };
    let tv = match find_tv(&app_handle, mvdb_id) {
        Ok(tv) => tv,
        Err(e) => return render_error(&e.message),
    };

    let season = match find_season(&app_handle, mvdb_id, season_number) {
        Ok(season) => season,
        Err(e) => return render_error(&e.message),
    };

    let episode = match season
        .episodes
        .iter()
        .find(|e| e.episode_number == episode_number)
    {
        Some(episode) => episode,
        None => return templates::render_error("Could not find episode to assign"),
    };
    let disk_id = optical_disk.read().expect("failed to lock optical_disk").id;
    let job = match background_process_state.find_job(
        Some(disk_id),
        &Some(JobType::Ripping),
        &[JobStatus::Pending],
    ) {
        Some(job) => job,
        None => {
            let optical_disk_info = optical_disk.read().unwrap().clone();
            let job = background_process_state.new_job(
                JobType::Ripping,
                JobStatus::Pending,
                Some(optical_disk_info),
            );
            background_process_state.emit_jobs_changed(&app_handle);
            job
        }
    };
    let title_video = job.read().unwrap().find_tv_title_video(
        mvdb_id,
        season_number,
        episode_number,
        title_id,
        Some(part),
    );

    let title_info = match optical_disk.read().unwrap().find_title_by_id(title_id) {
        Some(title) => title,
        None => {
            return render_error("Failed to find Title on Optical Disk to Assign Episode");
        }
    };

    match title_video {
        Some(_) => {
            return templates::render_error("Episode already assigned to title");
        }
        None => {
            let tv_season_episode = title_video::TvSeasonEpisode {
                tv: tv.clone(),
                season: season.clone(),
                episode: episode.clone(),
                part: Some(part),
            };
            let title_video = TitleVideo {
                id: title_video::TitleVideoId::new(),
                video: Video::Tv(Box::new(tv_season_episode)),
                title: Some(title_info.clone()),
            };
            job.write()
                .expect("Failed to lock job for write")
                .title_videos
                .push(Arc::new(RwLock::new(title_video)));
        }
    };

    templates::seasons::render_title_selected(&app_handle, &tv, season)
}

#[tauri::command]
pub fn withdraw_episode_from_title(
    mvdb_id: u32,
    season_number: u32,
    episode_number: u32,
    title_id: u32,
    app_state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, templates::Error> {
    let optical_disk = match app_state.selected_disk() {
        Some(d) => d,
        None => return render_error("No current selected disk"),
    };

    let season = match find_season(&app_handle, mvdb_id, season_number) {
        Ok(season) => season,
        Err(e) => return render_error(&e.message),
    };

    let tv = match find_tv(&app_handle, mvdb_id) {
        Ok(tv) => tv,
        Err(e) => return render_error(&e.message),
    };

    let job = find_or_create_job(&app_handle, &optical_disk.read().unwrap());
    let title_video = job.read().unwrap().find_tv_title_video(
        mvdb_id,
        season_number,
        episode_number,
        title_id,
        None,
    );
    match title_video {
        Some(title_video) => {
            job.write()
                .expect("Failed to lock job for write")
                .title_videos
                .retain(|tv| !Arc::ptr_eq(tv, &title_video));
        }
        None => {
            return render_error("Failed to find Episode to Withdraw from Title");
        }
    }

    templates::seasons::render_title_selected(&app_handle, &tv, season)
}

#[derive(Deserialize)]
pub struct EpisodeSwap {
    pub from: u32,
    pub to: u32,
}

#[tauri::command]
pub fn reorder_tv_episodes_on_ftp(
    mvdb_id: u32,
    season_number: u32,
    swaps: Vec<EpisodeSwap>,
    app_state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, templates::Error> {
    let tv = match find_tv(&app_handle, mvdb_id) {
        Ok(tv) => tv,
        Err(e) => return render_error(&e.message),
    };

    let season = match find_season(&app_handle, mvdb_id, season_number) {
        Ok(season) => season,
        Err(e) => return render_error(&e.message),
    };

    let filtered_swaps: Vec<(u32, u32)> = swaps
        .into_iter()
        .filter(|swap| swap.from != swap.to)
        .map(|swap| (swap.from, swap.to))
        .collect();

    if filtered_swaps.is_empty() {
        let toast = Toast::new(
            "No episode changes",
            "Pick at least one different episode destination to reorder files.",
            ToastVariant::Info,
        );
        let toast_stream = templates::toast::render_toast_append(toast)?;
        return Ok(toast_stream);
    }

    let mut unique_sources = std::collections::HashSet::new();
    let mut unique_targets = std::collections::HashSet::new();
    for (from, to) in &filtered_swaps {
        if !unique_sources.insert(*from) {
            return render_error(&format!("Episode {from} is listed more than once"));
        }
        if !unique_targets.insert(*to) {
            return render_error(&format!("Episode {to} is targeted more than once"));
        }
    }

    let renamed_count =
        match ftp_uploader::reorder_tv_episode_files(&tv, &season, &filtered_swaps, &app_state) {
            Ok(count) => count,
            Err(message) => return render_error(&message),
        };

    let toast = Toast::success(
        "Episode files reordered",
        format!("Renamed {renamed_count} file(s) on FTP."),
    );
    let toast_stream = templates::toast::render_toast_append(toast)?;
    let season_stream = templates::seasons::render_show(&app_handle, &tv, &season)?;

    Ok(format!("{toast_stream}{season_stream}"))
}

#[tauri::command]
pub fn rip_season(
    app_handle: tauri::AppHandle,
    app_state: State<'_, AppState>,
) -> Result<String, templates::Error> {
    let disk_id = app_state
        .selected_optical_disk_id
        .read()
        .unwrap()
        .to_owned();
    let disk_id = match disk_id {
        Some(id) => id,
        None => {
            debug!("No optical disk is currently selected.");
            return templates::render_error("No selected disk");
        }
    };

    let optical_disk = match app_state.find_optical_disk_by_id(&disk_id) {
        Some(optical_disk) => optical_disk,
        None => return render_error("Failed to find Optical Disk"),
    };
    let background_process_state = app_handle.state::<BackgroundProcessState>();
    let (job, is_new) = background_process_state.find_or_create_job(
        Some(disk_id),
        &Some(optical_disk),
        &JobType::Ripping,
        &JobStatus::Pending,
    );

    if is_new {
        background_process_state.emit_jobs_changed(&app_handle);
    }

    job.write()
        .expect("Failed to get job writer")
        .update_status(JobStatus::Processing);

    let season_update = {
        let job_guard = job.read().expect("Failed to get job reader");
        let tv_and_season = job_guard.title_videos.iter().find_map(|title_video| {
            let title_video_guard = title_video.read().ok()?;
            match &title_video_guard.video {
                Video::Tv(tv_season_episode) => Some((
                    tv_season_episode.tv.clone(),
                    tv_season_episode.season.clone(),
                )),
                Video::Movie(_) => None,
            }
        });

        match tv_and_season {
            Some((tv, season)) => templates::seasons::render_show(&app_handle, &tv, &season)?,
            None => String::new(),
        }
    };

    spawn_rip(app_handle, job);
    Ok(season_update)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn rip_movie(
    disk_id: u32,
    title_id: u32,
    mvdb_id: u32,
    part: Option<u16>,
    edition: Option<String>,
    app_state: State<'_, AppState>,
    background_process_state: State<'_, background_process_state::BackgroundProcessState>,
    app_handle: tauri::AppHandle,
) -> Result<String, templates::Error> {
    let disk_id = DiskId::from(disk_id);
    let optical_disk = match app_state.find_optical_disk_by_id(&disk_id) {
        Some(optical_disk) => optical_disk,
        None => return render_error("Failed to find Optical Disk"),
    };

    let title_info = match optical_disk.read().unwrap().find_title_by_id(title_id) {
        Some(title) => title,
        None => {
            return render_error("Failed to find Title on Optical Disk to Rip");
        }
    };

    let (job, is_new) = background_process_state.find_or_create_job(
        Some(disk_id),
        &Some(optical_disk),
        &JobType::Ripping,
        &JobStatus::Pending,
    );

    if is_new {
        background_process_state.emit_jobs_changed(&app_handle);
    }

    let movie = match find_movie(&app_handle, mvdb_id) {
        Ok(movie) => movie,
        Err(e) => return render_error(&e.message),
    };

    let movie_part_edition = crate::state::title_video::MoviePartEdition {
        movie: movie.clone(),
        part,
        edition,
    };

    match job
        .write()
        .expect("Failed to lock job")
        .add_title_video(title_info, Video::Movie(Box::new(movie_part_edition)))
    {
        Ok(_) => {}
        Err(e) => {
            return render_error(&e.message);
        }
    };
    job.read()
        .expect("Failed to lock job for read")
        .emit_progress_change(&app_handle);
    spawn_rip(app_handle, job);
    Ok("".to_string())
}

#[tauri::command]
pub fn set_auto_rip(
    disk_id: u32,
    mvdb_id: u32,
    enable: bool,
    app_state: State<'_, AppState>,
    background_process_state: State<'_, background_process_state::BackgroundProcessState>,
    app_handle: tauri::AppHandle,
) -> Result<String, templates::Error> {
    let disk_id = DiskId::from(disk_id);

    if enable {
        let optical_disk = match app_state.find_optical_disk_by_id(&disk_id) {
            Some(optical_disk) => optical_disk,
            None => return render_error("Failed to find Optical Disk"),
        };

        let (job, is_new) = background_process_state.find_or_create_job(
            Some(disk_id),
            &Some(optical_disk),
            &JobType::Ripping,
            &JobStatus::Pending,
        );

        if is_new {
            debug!("Created new job for auto-rip: {}", job.read().unwrap().id);
        } else {
            debug!(
                "Found existing job for auto-rip: {}",
                job.read().unwrap().id
            );
        }

        let movie = match find_movie(&app_handle, mvdb_id) {
            Ok(movie) => movie,
            Err(e) => return render_error(&e.message),
        };

        let movie_part_edition = crate::state::title_video::MoviePartEdition {
            movie: movie.clone(),
            part: None,
            edition: None,
        };

        match job
            .write()
            .expect("Failed to lock job")
            .add_incomplete_video(Video::Movie(Box::new(movie_part_edition)))
        {
            Ok(_) => {}
            Err(e) => {
                return render_error(&e.message);
            }
        };
        background_process_state.emit_jobs_changed(&app_handle);
    } else if let Some(job) = background_process_state.find_job(
        Some(disk_id),
        &Some(JobType::Ripping),
        &[JobStatus::Pending],
    ) {
        let job_id = job.read().expect("Failed to lock job for read").id;
        background_process_state.delete_job(job_id);
        debug!("Deleted job {job_id} for auto-rip disable");
        background_process_state.emit_jobs_changed(&app_handle);
    } else {
        debug!("No existing job found for auto-rip disable, nothing to delete");
    };

    templates::movies::render_cards(&app_handle)
}

fn emit_render_cards(app_handle: &tauri::AppHandle) {
    let result =
        templates::movies::render_cards(app_handle).expect("Failed to render movies/cards.html");
    app_handle
        .emit("disks-changed", result)
        .expect("Failed to emit disks-changed");
}

fn notify_movie_success(
    app_handle: &tauri::AppHandle,
    movie: &crate::state::title_video::MoviePartEdition,
) {
    app_handle
        .notification()
        .builder()
        .title(format!("Finished Ripping {}", movie.movie.title))
        .body(movie.movie.title_year())
        .show()
        .unwrap();
}

fn notify_failure(app_handle: &tauri::AppHandle, error: &StandardError) {
    app_handle
        .notification()
        .builder()
        .title(error.title.clone())
        .body(error.message.clone())
        .show()
        .unwrap();
}

fn notify_movie_upload_success(app_handle: &tauri::AppHandle, file_path: &Path) {
    app_handle
        .notification()
        .builder()
        .title("Finished Upload Movie".to_string())
        .body(format!("File Path {}", file_path.to_string_lossy()))
        .show()
        .unwrap();
}

fn notify_movie_upload_failure(app_handle: &tauri::AppHandle, file_path: &Path, error: &str) {
    debug!(
        "failed to upload: {} {}",
        file_path.to_string_lossy(),
        error
    );
    app_handle
        .notification()
        .builder()
        .title("Failed to Upload")
        .body(format!("{} {}", file_path.to_string_lossy(), error))
        .show()
        .unwrap();
}

/// Extract upload preparation data from a title_video
fn extract_upload_info(
    app_handle: &tauri::AppHandle,
    title_video: &Arc<RwLock<TitleVideo>>,
    rip_job: &Arc<RwLock<Job>>,
) -> Option<(
    UploadedState,
    PathBuf,
    crate::state::upload_state::UploadType,
)> {
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

    let path = title_video
        .read()
        .expect("Failed to get title_video reader")
        .video_path(&app_handle.state::<AppState>(), multiple_parts);

    let upload_type = {
        let video_guard = title_video
            .read()
            .expect("Failed to get title_video reader");
        match &video_guard.video {
            Video::Movie(_) => crate::state::upload_state::UploadType::Movie,
            Video::Tv(_) => crate::state::upload_state::UploadType::TvShow,
        }
    };

    Some((uploaded_state, path, upload_type))
}

fn spawn_upload(
    app_handle: &tauri::AppHandle,
    rip_job: &Arc<RwLock<Job>>,
    title_video: &Arc<RwLock<TitleVideo>>,
) {
    let app_handle = app_handle.clone();
    let rip_job = rip_job.clone();
    let title_video = title_video.clone();
    tauri::async_runtime::spawn(async move {
        let (uploaded_state, path, upload_type) =
            match extract_upload_info(&app_handle, &title_video, &rip_job) {
                Some(info) => info,
                None => return,
            };

        // Add to persistent upload queue before starting
        if let Err(e) =
            uploaded_state.add_upload(&app_handle, path.to_string_lossy().to_string(), upload_type)
        {
            error!("Failed to add video to upload queue: {e}");
            return;
        }

        let background_process_state = app_handle.state::<BackgroundProcessState>();
        let (job, is_new) = background_process_state.find_or_create_job(
            None,
            &None,
            &JobType::Uploading,
            &JobStatus::Pending,
        );

        if is_new {
            background_process_state.emit_jobs_changed(&app_handle);
        }

        job.write()
            .expect("Failed to get job writer")
            .title_videos
            .push(title_video.clone());
        job.write()
            .expect("Failed to get job writer")
            .update_status(JobStatus::Processing);
        job.write().expect("Failed to get job writer").subtitle =
            Some("Uploading Video".to_string());
        job.read()
            .expect("Failed to get job reader")
            .emit_progress_change(&app_handle);

        match services::ftp_uploader::upload(&app_handle, &job, &title_video).await {
            Ok(_m) => {
                notify_movie_upload_success(&app_handle, &path);
                job.write()
                    .expect("Failed to acquire write lock on job")
                    .update_status(JobStatus::Finished);
                emit_progress(&app_handle, &job, true);

                // Remove from upload queue on success
                if let Err(e) = uploaded_state.remove_upload(&app_handle, &path.to_string_lossy()) {
                    error!("Failed to remove video from upload queue: {e}");
                }

                delete_file(&path);
            }
            Err(e) => {
                job.write()
                    .expect("Failed to get job writer")
                    .update_status(JobStatus::Error);
                job.write().expect("Failed to get job writer").message = Some(e.clone());
                emit_progress(&app_handle, &job, true);
                notify_movie_upload_failure(&app_handle, &path, &e);
                // Keep in upload queue on failure for retry on next boot
            }
        };
    });
}

fn delete_file(file_path: &Path) {
    if let Err(error) = fs::remove_file(file_path) {
        error!("Failed to delete file {}: {}", file_path.display(), error);
    };
}

async fn rip_title(
    app_handle: &tauri::AppHandle,
    job: &Arc<RwLock<Job>>,
    title_video: &Arc<RwLock<TitleVideo>>,
) -> Result<PathBuf, StandardError> {
    match makemkvcon::rip_title(app_handle, job, title_video).await {
        Ok(_) => {
            let app_state = app_handle.state::<AppState>();
            let job_reader = job.read().expect("Failed to get job reader");
            title_video
                .read()
                .expect("Failed to get title_video reader")
                .rename_ripped_file(&app_state, &job_reader)
                .map_err(|e| StandardError {
                    title: "Rename Failure".into(),
                    message: e,
                })
        }
        Err(e) => Err(StandardError {
            title: "Rip Failure".into(),
            message: e,
        }),
    }
}

// this never worked I will work on making this work later
// async fn back_disk(
//     app_handle: &tauri::AppHandle,
//     job: &Arc<RwLock<Job>>,
//     disk_id: &DiskId,
//     rip_info: &RipInfo,
// ) -> Result<(), StandardError> {
//     match makemkvcon::back_disk(app_handle, job, disk_id, &rip_info.directory).await {
//         Ok(_) => Ok(()),
//         Err(e) => Err(StandardError {
//             title: "Backup Failure".into(),
//             message: e,
//         }),
//     }
// }

fn notify_tv_success(app_handle: &tauri::AppHandle, title: &title_video::TvSeasonEpisode) {
    app_handle
        .notification()
        .builder()
        .title(format!("Episode Created for {}", title.tv.name))
        .body(title.title().to_string())
        .show()
        .unwrap();
}

// fn build_info(app_handle: &tauri::AppHandle, disk_id: &DiskId) -> JobInfo {
//     let state = app_handle.state::<AppState>();
//     let optical_disk = state.find_optical_disk_by_id(disk_id).unwrap();
//     {
//         let locked_disk = optical_disk.read().unwrap();
//         match locked_disk.content.as_ref().unwrap() {
//             DiskContent::Movie(movie) => {
//                 let dir = create_movie_dir(movie);
//                 let titles = locked_disk
//                     .titles
//                     .lock()
//                     .unwrap()
//                     .iter()
//                     .filter(|t| t.rip)
//                     .cloned()
//                     .collect::<Vec<_>>();
//                 JobInfo {
//                     directory: dir,
//                     titles,
//                     content: DiskContent::Movie(movie.clone()),
//                     disk: locked_disk.clone(),
//                 }
//             }
//             DiskContent::Tv(season) => {
//                 let dir = create_season_episode_dir(season);
//                 let titles = locked_disk
//                     .titles
//                     .lock()
//                     .unwrap()
//                     .iter()
//                     .filter(|t| t.rip)
//                     .cloned()
//                     .collect::<Vec<_>>();
//                 RipInfo {
//                     directory: dir,
//                     titles,
//                     content: DiskContent::Tv(season.clone()),
//                     disk: locked_disk.clone(),
//                 }
//             }
//         }
//     }
// }

fn eject_disk(app_handle: &tauri::AppHandle, disk_id: &DiskId) {
    let state = app_handle.state::<AppState>();
    match state.find_optical_disk_by_id(disk_id) {
        Some(disk) => match disk.read() {
            Ok(locked_disk) => disk_manager::eject(&locked_disk.mount_point),
            Err(_) => debug!("Failed to eject disk"),
        },
        None => {
            debug!("failed to find disk to eject")
        }
    }
}

async fn process_titles(app_handle: &tauri::AppHandle, job: Arc<RwLock<Job>>) -> bool {
    let mut any_success = false;
    let mut has_error = false;
    let title_videos = {
        let job_guard = job.read().expect("Failed to get job writer");
        job_guard.title_videos.clone()
    };
    for title in title_videos.iter() {
        // Set current title video ID for progress tracking
        job.write()
            .expect("Failed to get job writer")
            .current_title_video_id = Some(title.read().unwrap().id);
        job.write()
            .expect("Failed to get job writer")
            .update_title(&title.read().unwrap());
        job.read()
            .expect("Failed to get job reader")
            .emit_progress_change(app_handle);
        match rip_title(app_handle, &job, title).await {
            Ok(_) => {
                any_success = true;
                match &title.read().unwrap().video {
                    Video::Tv(season) => {
                        notify_tv_success(app_handle, season);
                        spawn_upload(app_handle, &job, title);
                    }
                    Video::Movie(movie) => {
                        notify_movie_success(app_handle, movie);
                        emit_render_cards(app_handle);
                        spawn_upload(app_handle, &job, title);
                    }
                };
                job.read()
                    .expect("Failed to get job reader")
                    .emit_progress_change(app_handle);
            }
            Err(error) => {
                has_error = true;
                match &title.read().unwrap().video {
                    Video::Tv(_) => {}
                    Video::Movie(_) => {
                        emit_render_cards(app_handle);
                    }
                };
                job.write().expect("Failed to get job writer").message =
                    Some(error.message.clone());
                job.write().expect("Failed to get job writer").subtitle = Some(error.title.clone());
                job.read()
                    .expect("Failed to get job reader")
                    .emit_progress_change(app_handle);
                notify_failure(app_handle, &error);
            }
        };
    }

    // Mark job as finished/error only after ALL titles are processed
    if has_error {
        job.write()
            .expect("Failed to get job writer")
            .update_status(JobStatus::Error);
    } else if any_success {
        job.write()
            .expect("Failed to get job writer")
            .update_status(JobStatus::Finished);
    }

    // Final UI update
    job.read()
        .expect("Failed to get job reader")
        .emit_progress_change(app_handle);
    templates::disks::emit_disk_change(app_handle);

    any_success
}

pub fn spawn_rip(app_handle: tauri::AppHandle, job: Arc<RwLock<Job>>) {
    tauri::async_runtime::spawn(async move {
        job.write()
            .expect("Failed to get job writer")
            .update_status(JobStatus::Processing);
        let has_tv_titles = {
            let job_guard = job.read().expect("Failed to get job reader");
            job_guard.title_videos.iter().any(|title_video| {
                title_video
                    .read()
                    .map(|guard| matches!(guard.video, Video::Tv(_)))
                    .unwrap_or(false)
            })
        };
        if !has_tv_titles {
            templates::disks::emit_disk_change(&app_handle);
        }
        job.read()
            .expect("Failed to get job reader")
            .emit_progress_change(&app_handle);
        let success = process_titles(&app_handle, job.clone()).await;
        if success {
            match &job.read().expect("Failed to get job reader").disk {
                Some(disk) => eject_disk(&app_handle, &disk.id),
                None => warn!("No disk found in job after ripping nothing to eject"),
            };
        }
    });
}

// fn after_process_titles(
//     app_handle: &tauri::AppHandle,
//     disk_id: &DiskId,
//     job: &Arc<RwLock<Job>>,
//     success: bool,
// ) {
//     if success {
//         job.write()
//             .expect("Failed to get job writer")
//             .update_status(JobStatus::Finished);
//         eject_disk(app_handle, disk_id);
//     } else {
//         job.write()
//             .expect("Failed to get job writer")
//             .update_status(JobStatus::Error);
//     }
// }

fn find_or_create_job(app_handle: &tauri::AppHandle, disk: &OpticalDiskInfo) -> Arc<RwLock<Job>> {
    let background_process_state = app_handle.state::<BackgroundProcessState>();

    let disk_id = disk.id;
    match background_process_state.find_job(
        Some(disk_id),
        &Some(JobType::Ripping),
        &[JobStatus::Pending],
    ) {
        Some(job) => job,
        None => {
            let optical_disk_info = disk.clone();
            let job = background_process_state.new_job(
                JobType::Ripping,
                JobStatus::Pending,
                Some(optical_disk_info),
            );
            background_process_state.emit_jobs_changed(app_handle);
            job
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::title_video::TvSeasonEpisode;
    use crate::the_movie_db::{SeasonEpisode, SeasonResponse, TvResponse};

    fn create_mock_tv_episode(id: u32, episode_number: u32) -> SeasonEpisode {
        SeasonEpisode {
            id,
            episode_number,
            episode_type: "standard".to_string(),
            name: format!("Episode {episode_number}"),
            overview: "Test episode".to_string(),
            air_date: Some("2020-01-01".to_string()),
            production_code: None,
            runtime: Some(45),
            season_number: 1,
            show_id: 100,
            still_path: None,
            vote_average: 8.0,
            vote_count: 100,
            crew: vec![],
            guest_stars: vec![],
        }
    }

    fn create_mock_season() -> SeasonResponse {
        SeasonResponse {
            _id: "test_season".to_string(),
            id: 1,
            season_number: 1,
            name: "Season 1".to_string(),
            overview: "Test season".to_string(),
            poster_path: None,
            air_date: Some("2020-01-01".to_string()),
            episodes: vec![
                create_mock_tv_episode(1, 1),
                create_mock_tv_episode(2, 2),
                create_mock_tv_episode(3, 3),
            ],
            vote_average: 8.5,
        }
    }

    fn create_mock_tv() -> TvResponse {
        TvResponse {
            adult: false,
            backdrop_path: None,
            created_by: vec![],
            episode_run_time: vec![45],
            first_air_date: Some("2020-01-01".to_string()),
            genres: vec![],
            homepage: None,
            id: 100,
            in_production: false,
            languages: vec!["en".to_string()],
            last_air_date: None,
            last_episode_to_air: None,
            name: "Test Show".to_string(),
            networks: vec![],
            next_episode_to_air: None,
            number_of_episodes: 20,
            number_of_seasons: 2,
            origin_country: vec!["US".to_string()],
            original_language: "en".to_string(),
            original_name: "Test Show".to_string(),
            overview: "Test overview".to_string(),
            popularity: 100.0,
            poster_path: None,
            production_companies: vec![],
            production_countries: vec![],
            seasons: vec![],
            spoken_languages: vec![],
            status: "Returning Series".to_string(),
            tagline: "Test tagline".to_string(),
            type_: "Scripted".to_string(),
            vote_average: 8.5,
            vote_count: 1000,
        }
    }

    #[test]
    fn test_episode_ids_are_unique() {
        let season = create_mock_season();

        // Verify each episode has a unique ID
        assert_eq!(season.episodes[0].id, 1);
        assert_eq!(season.episodes[1].id, 2);
        assert_eq!(season.episodes[2].id, 3);

        // Verify IDs are different
        assert_ne!(season.episodes[0].id, season.episodes[1].id);
        assert_ne!(season.episodes[1].id, season.episodes[2].id);
    }

    #[test]
    fn test_episode_numbers_match_ids() {
        let season = create_mock_season();

        // For our mock data, episode number should match ID
        assert_eq!(season.episodes[0].episode_number, 1);
        assert_eq!(season.episodes[1].episode_number, 2);
        assert_eq!(season.episodes[2].episode_number, 3);
    }

    #[test]
    fn test_tv_season_episode_creation() {
        let tv = create_mock_tv();
        let season = create_mock_season();
        let episode = season.episodes[0].clone();

        let tv_season_episode = TvSeasonEpisode {
            tv: tv.clone(),
            season: season.clone(),
            episode: episode.clone(),
            part: Some(1),
        };

        assert_eq!(tv_season_episode.episode.id, 1);
        assert_eq!(tv_season_episode.part, Some(1));
        assert_eq!(tv_season_episode.tv.id, 100);
        assert_eq!(tv_season_episode.season.season_number, 1);
    }

    #[test]
    fn test_different_parts_for_same_episode() {
        let tv = create_mock_tv();
        let season = create_mock_season();
        let episode = season.episodes[0].clone();

        let part1 = TvSeasonEpisode {
            tv: tv.clone(),
            season: season.clone(),
            episode: episode.clone(),
            part: Some(1),
        };

        let part2 = TvSeasonEpisode {
            tv: tv.clone(),
            season: season.clone(),
            episode: episode.clone(),
            part: Some(2),
        };

        // Same episode, different parts
        assert_eq!(part1.episode.id, part2.episode.id);
        assert_ne!(part1.part, part2.part);
    }

    #[test]
    fn test_multiple_episodes_maintain_independence() {
        let tv = create_mock_tv();
        let season = create_mock_season();

        let episode1 = TvSeasonEpisode {
            tv: tv.clone(),
            season: season.clone(),
            episode: season.episodes[0].clone(),
            part: Some(1),
        };

        let episode2 = TvSeasonEpisode {
            tv: tv.clone(),
            season: season.clone(),
            episode: season.episodes[1].clone(),
            part: Some(1),
        };

        let episode3 = TvSeasonEpisode {
            tv: tv.clone(),
            season: season.clone(),
            episode: season.episodes[2].clone(),
            part: Some(1),
        };

        // Verify each episode has unique ID
        assert_eq!(episode1.episode.id, 1);
        assert_eq!(episode2.episode.id, 2);
        assert_eq!(episode3.episode.id, 3);

        // Verify they're all different
        assert_ne!(episode1.episode.id, episode2.episode.id);
        assert_ne!(episode2.episode.id, episode3.episode.id);
        assert_ne!(episode1.episode.id, episode3.episode.id);
    }

    #[test]
    fn test_episode_assignment_parameters() {
        // Test parameter types used in assign_episode_to_title
        let mvdb_id: u32 = 100;
        let season_number: u32 = 1;
        let episode_number: u32 = 1;
        let title_id: u32 = 5;
        let part: u16 = 1;

        assert!(mvdb_id > 0);
        assert!(season_number > 0);
        assert!(episode_number > 0);
        assert!(title_id > 0);
        assert!(part > 0);
    }

    #[test]
    fn test_job_type_ripping() {
        let _job_type = JobType::Ripping;
        let _loading_type = JobType::Loading;

        // Test passes if enums can be created successfully
    }

    #[test]
    fn test_job_status_pending() {
        let _pending = JobStatus::Pending;
        let _processing = JobStatus::Processing;

        // Test passes if enums can be created successfully
    }
}
