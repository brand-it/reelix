use crate::models::optical_disk_info::{DiskId, OpticalDiskInfo};
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
use crate::templates::{self};
use log::{debug, error, warn};
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
        &[JobStatus::Pending, JobStatus::Ready],
    ) {
        Some(job) => job,
        None => {
            let optical_disk_info = optical_disk.read().unwrap().clone();
            let job = background_process_state.new_job(JobType::Ripping, Some(optical_disk_info));
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
                video: Video::Tv(Box::new(tv_season_episode)),
                title: title_info.clone(),
            };
            job.write()
                .expect("Failed to lock job for write")
                .title_videos
                .push(Arc::new(RwLock::new(title_video)));
        }
    };

    templates::seasons::render_title_selected(&app_handle, season)
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

    templates::seasons::render_title_selected(&app_handle, season)
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
        &[JobStatus::Pending, JobStatus::Ready],
    );

    if is_new {
        background_process_state.emit_jobs_changed(&app_handle);
    }

    spawn_rip(app_handle, job);
    Ok("".to_string())
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
        &[JobStatus::Pending, JobStatus::Ready],
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

    let path = title_video
        .read()
        .expect("Failed to get title_video reader")
        .video_path(&app_handle.state::<AppState>());

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

fn spawn_upload(app_handle: &tauri::AppHandle, title_video: &Arc<RwLock<TitleVideo>>) {
    let app_handle = app_handle.clone();
    let title_video = title_video.clone();
    tauri::async_runtime::spawn(async move {
        let (uploaded_state, path, upload_type) =
            match extract_upload_info(&app_handle, &title_video) {
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
            &[JobStatus::Pending, JobStatus::Ready],
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
            title_video
                .read()
                .expect("Failed to get title_video reader")
                .rename_ripped_file(&app_state)
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
    let mut success = false;
    let title_videos = {
        let job_guard = job.read().expect("Failed to get job writer");
        job_guard.title_videos.clone()
    };
    for title in title_videos.iter() {
        job.write()
            .expect("Failed to get job writer")
            .update_title(&title.read().unwrap());
        job.read()
            .expect("Failed to get job reader")
            .emit_progress_change(app_handle);
        match rip_title(app_handle, &job, title).await {
            Ok(_) => {
                success = true;
                match &title.read().unwrap().video {
                    Video::Tv(season) => {
                        notify_tv_success(app_handle, season);
                    }
                    Video::Movie(movie) => {
                        notify_movie_success(app_handle, movie);
                        emit_render_cards(app_handle);
                        spawn_upload(app_handle, title);
                    }
                };
                job.write()
                    .expect("Failed to get job writer")
                    .update_status(JobStatus::Finished);
                templates::disks::emit_disk_change(app_handle);
            }
            Err(error) => {
                match &title.read().unwrap().video {
                    Video::Tv(_) => {}
                    Video::Movie(_) => {
                        emit_render_cards(app_handle);
                    }
                };
                job.write()
                    .expect("Failed to get job writer")
                    .update_status(JobStatus::Error);
                job.write().expect("Failed to get job writer").message =
                    Some(error.message.clone());
                job.write().expect("Failed to get job writer").subtitle = Some(error.title.clone());
                job.read()
                    .expect("Failed to get job reader")
                    .emit_progress_change(app_handle);
                templates::disks::emit_disk_change(app_handle);
                notify_failure(app_handle, &error);
            }
        };
    }
    success
}

fn spawn_rip(app_handle: tauri::AppHandle, job: Arc<RwLock<Job>>) {
    tauri::async_runtime::spawn(async move {
        job.write()
            .expect("Failed to get job writer")
            .update_status(JobStatus::Processing);
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
        &[JobStatus::Pending, JobStatus::Ready],
    ) {
        Some(job) => job,
        None => {
            let optical_disk_info = disk.clone();
            let job = background_process_state.new_job(JobType::Ripping, Some(optical_disk_info));
            background_process_state.emit_jobs_changed(&app_handle);
            job
        }
    }
}
