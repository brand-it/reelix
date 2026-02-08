use crate::models::optical_disk_info::{DiskId, OpticalDiskInfo};
use crate::services::drive_info::opticals;
use crate::services::makemkvcon;
use crate::state::background_process_state::BackgroundProcessState;
use crate::state::job_state::{Job, JobStatus, JobType};
use crate::state::title_video::Video;
use crate::state::AppState;
use crate::templates;
use log::debug;
use std::sync::{Arc, RwLock};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};

// pub fn list() {
//     let disks: Disks = Disks::new_with_refreshed_list();

//     for disk in &disks {
//         let fs_bytes = disk.file_system();
//         let fs_str = fs_bytes.to_str().expect("Failed to load fs_bytes");
//         debug!("#-------------------DISK---------------------#");
//         // Check if removable + known optical file system
//         if disk.is_removable() && (fs_str.contains("udf") || fs_str.contains("iso9660")) {
//             debug!("Likely optical media:");
//             debug!("  Name: {:?}", disk.name());
//             debug!("  Mount Point: {:?}", disk.mount_point());
//             debug!("  Available Space: {}", disk.available_space());
//             debug!("  Total Space: {}", disk.total_space());
//             debug!("  Kind: {}", disk.kind());
//             debug!("  File System: {:?}", disk.file_system());
//             debug!("  Is Removable: {}", disk.is_removable());
//             debug!("  Is Read Only: {}", disk.is_read_only());
//             debug!("  Usage: {:?}", disk.usage());
//         } else {
//             debug!("Non-optical or unrecognized: {:?}", disk.name());
//         }
//         debug!("#-------------------END DISK-----------------#");
//     }
// }

fn changes(
    current_opticals: &[OpticalDiskInfo],
    previous_opticals: &[OpticalDiskInfo],
) -> Vec<diff::Result<OpticalDiskInfo>> {
    let mut optics = Vec::new();
    diff::slice(previous_opticals, current_opticals)
        .into_iter()
        .for_each(|result| match result {
            diff::Result::Left(info) => optics.push(diff::Result::Left(info.clone())),
            diff::Result::Both(info, _) => {
                optics.push(diff::Result::Both(info.clone(), info.clone()))
            }
            diff::Result::Right(info) => optics.push(diff::Result::Right(info.clone())),
        });
    optics
}

pub async fn watch_for_changes(sender: broadcast::Sender<Vec<diff::Result<OpticalDiskInfo>>>) {
    let mut previous_opticals = Vec::new();
    debug!("Stared watching for changes to optical Disks....");
    loop {
        let current_opticals = opticals();

        if current_opticals != previous_opticals {
            let diff_result = changes(&current_opticals, &previous_opticals);

            match sender.send(diff_result) {
                Ok(num_receivers) => debug!("Broadcast sent to {num_receivers} receivers"),
                Err(_err) => debug!("Broadcast send failed"),
            }
            previous_opticals = current_opticals;
        }
        // Failure to sleep ever second means we use 100% of our CPU DUH
        // Hey future "human" improve this scanner system...or don't if it works why change it
        sleep(Duration::from_secs(5)).await;
    }
}

fn emit_disk_titles_change(app_handle: &AppHandle) {
    let app_state = app_handle.state::<AppState>();
    let background_process_state = app_handle.state::<BackgroundProcessState>();

    let result = templates::disk_titles::render_options(&app_state, &background_process_state)
        .expect("Failed to render disk_titles/options");
    app_handle
        .emit("disks-changed", result)
        .expect("Failed to emit emit_disk_titles_change");
}

fn unwrap_disk(disk: &Arc<RwLock<OpticalDiskInfo>>) -> OpticalDiskInfo {
    disk.read().expect("Failed to lock").clone()
}

fn contains(
    optical_disks: &[Arc<RwLock<OpticalDiskInfo>>],
    disk: &Arc<RwLock<OpticalDiskInfo>>,
) -> bool {
    optical_disks
        .iter()
        .any(|optical_disk| unwrap_disk(optical_disk) == unwrap_disk(disk))
}

async fn load_titles(app_handle: &AppHandle, job: &Arc<RwLock<Job>>) {
    let state: tauri::State<'_, AppState> = app_handle.state::<AppState>();
    let background_process_state: tauri::State<
        '_,
        crate::state::background_process_state::BackgroundProcessState,
    > = app_handle.state();
    job.write()
        .expect("failed to lock job for write")
        .update_status(JobStatus::Processing);
    job.read()
        .expect("failed to lock job for read")
        .emit_progress_change(app_handle);
    let results = match makemkvcon::title_info(app_handle, job).await {
        Ok(run_result) => run_result,
        Err(message) => {
            debug!("failed to load titles: {message}");
            job.write()
                .expect("failed to lock job for write")
                .update_status(JobStatus::Error);
            job.write()
                .expect("failed to lock job for write")
                .update_message(&format!("Failed to load titles: {message}"));
            job.read()
                .expect("failed to lock job for read")
                .emit_progress_change(app_handle);
            return;
        }
    };

    let disk_id = job
        .read()
        .expect("failed to lock job for read")
        .disk
        .as_ref()
        .expect("There should of been a disk")
        .id;

    match state.find_optical_disk_by_id(&disk_id) {
        Some(disk) => {
            let locked_disk = disk.write().expect("Failed to grab disk");
            locked_disk
                .titles
                .lock()
                .expect("failed to get titles")
                .extend(results.title_infos);
        }
        None => debug!("Disk not found in state."),
    }
    job.write()
        .expect("failed to lock job for write")
        .update_status(JobStatus::Finished);
    job.read()
        .expect("failed to lock job for read")
        .emit_progress_change(app_handle);
    templates::disks::emit_disk_change(app_handle);

    if let Some(auto_rip_job) = background_process_state.find_job(
        Some(disk_id),
        &Some(JobType::Ripping),
        &[JobStatus::Pending],
    ) {
        auto_rip_if_ready(app_handle, &state, disk_id, auto_rip_job);
    }
}

fn auto_rip_if_ready(
    app_handle: &AppHandle,
    state: &tauri::State<'_, AppState>,
    disk_id: crate::models::optical_disk_info::DiskId,
    auto_rip_job: Arc<RwLock<Job>>,
) {
    debug!("auto_rip_if_ready: checking disk {disk_id}");
    let job_ref = auto_rip_job.read().expect("Failed to lock job for read");

    if !job_ref.has_incomplete_titles() {
        debug!(
            "auto_rip_if_ready: no incomplete titles on job {job_id}",
            job_id = job_ref.id
        );
        return;
    }

    drop(job_ref);

    if let Some(disk) = state.find_optical_disk_by_id(&disk_id) {
        let disk_lock = disk.read().expect("Failed to lock disk for read");
        let titles = disk_lock.titles.lock().expect("Failed to lock titles");
        debug!(
            "auto_rip_if_ready: found {count} titles for disk {disk_id}",
            count = titles.len()
        );
        let job_write = auto_rip_job.write().expect("Failed to lock job for write");
        let mut matched = 0usize;

        for title_video in &job_write.title_videos {
            let mut tv = title_video.write().expect("Failed to lock title_video");
            if tv.title.is_none() {
                if let Video::Movie(movie_edition) = &tv.video {
                    let runtime_range = movie_edition.runtime_range();
                    let movie_title = movie_edition.movie.title.clone();
                    let best_match = titles.iter().find(|title| {
                        title.within_range(&Some(runtime_range.clone())) && title.has_chapters()
                    });

                    if let Some(matched_title) = best_match {
                        tv.title = Some(matched_title.clone());
                        matched += 1;
                        debug!(
                            "auto_rip_if_ready: matched title_id={title_id} for movie {movie_title}",
                            title_id = matched_title.id
                        );
                    } else {
                        debug!(
                            "auto_rip_if_ready: no match found for movie {movie_title}"
                        );
                    }
                }
            }
        }

        debug!("auto_rip_if_ready: matched {matched} titles, starting rip");
    }

    crate::commands::rip::spawn_rip(app_handle.clone(), auto_rip_job);
}

fn add_optical_disk(app_handle: &AppHandle, disk: &OpticalDiskInfo) {
    let state: tauri::State<'_, AppState> = app_handle.state::<AppState>();
    let optical_disk = Arc::new(RwLock::new(disk.clone()));
    let mut optical_disks = state
        .optical_disks
        .write()
        .expect("Failed to grab optical disks");
    if !contains(&optical_disks, &optical_disk) {
        optical_disks.push(optical_disk);
    }
}

fn remove_optical_disks(app_handle: &AppHandle, disk: &OpticalDiskInfo) {
    let state: tauri::State<'_, AppState> = app_handle.state::<AppState>();
    let mut optical_disks = state
        .optical_disks
        .write()
        .expect("Failed to grab optical disks");
    optical_disks.retain(|optical_disk_info| {
        let optical_disk = optical_disk_info
            .read()
            .expect("Failed to grab optical disk info");

        if *optical_disk == *disk {
            optical_disk.kill_process();
            false // Remove this disk
        } else {
            true // Keep this disk
        }
    });
}

pub fn set_default_selected_disk(app_handle: &AppHandle, disk_id: DiskId) {
    let state = app_handle.state::<AppState>();
    let mut selected_optical_disk_id = state
        .selected_optical_disk_id
        .write()
        .expect("failed to lock selected disk ID");
    if selected_optical_disk_id.is_none() {
        debug!("changed default selected optical disk to {disk_id}");
        *selected_optical_disk_id = Some(disk_id);
    }
}

pub fn clear_selected_disk(app_handle: &AppHandle, disk_id: DiskId) {
    let state = app_handle.state::<AppState>();
    let mut selected_optical_disk_id = state
        .selected_optical_disk_id
        .write()
        .expect("failed to lock selected disk ID");

    if selected_optical_disk_id.as_ref() == Some(&disk_id) {
        *selected_optical_disk_id = None;
    }
}

/// A separate async task that listens for changes and reacts to them.
pub async fn handle_changes(
    mut receiver: broadcast::Receiver<Vec<diff::Result<OpticalDiskInfo>>>,
    app_handle: AppHandle,
) {
    loop {
        debug!("Waiting for changes on Disk");
        match receiver.recv().await {
            Ok(event) => {
                debug!("Message received");
                for result in event {
                    match result {
                        diff::Result::Left(disk) => {
                            debug!("- {:?}", disk.name);
                            clear_selected_disk(&app_handle, disk.id);
                            remove_optical_disks(&app_handle, &disk);
                            templates::disks::emit_disk_change(&app_handle);
                            emit_disk_titles_change(&app_handle);
                        }
                        diff::Result::Both(disk, _) => {
                            debug!("? {:?}", disk.name);
                        }
                        diff::Result::Right(disk) => {
                            debug!("+ {:?}", disk.name);
                            add_optical_disk(&app_handle, &disk);
                            set_default_selected_disk(&app_handle, disk.id);
                            templates::disks::emit_disk_change(&app_handle);
                            let app_handle_clone = app_handle.clone();
                            tokio::spawn(async move {
                                let background_process_state =
                                    app_handle_clone.state::<BackgroundProcessState>();
                                let job = background_process_state
                                    .new_job(JobType::Loading, JobStatus::Pending, Some(disk.clone()));
                                background_process_state.emit_jobs_changed(&app_handle_clone);
                                job.write().expect("failed to lock job for write").title =
                                    Some(format!("Loading Titles for {}", disk.name));
                                job.read()
                                    .expect("failed to lock job for read")
                                    .emit_progress_change(&app_handle_clone);
                                load_titles(&app_handle_clone, &job).await;
                                emit_disk_titles_change(&app_handle_clone);
                                templates::disks::emit_disk_change(&app_handle_clone);
                            });
                        }
                    }
                }
            }
            Err(broadcast::error::RecvError::Lagged(count)) => {
                debug!("Dropped {count} messages due to lag.");
            }
            Err(broadcast::error::RecvError::Closed) => {
                debug!("Channel has closed.");
            }
        }
    }
}
