use super::helpers::{
    add_episode_to_title, mark_title_rippable, remove_episode_from_title, rename_movie_file,
    rename_tv_file, set_optical_disk_as_movie, set_optical_disk_as_season,
};
use crate::commands::helpers::RipError;
use crate::models::movie_db::MovieResponse;
use crate::models::optical_disk_info::{DiskContent, DiskId, TvSeasonContent};
use crate::models::title_info::TitleInfo;
use crate::services::plex::{create_season_episode_dir, find_tv};
use crate::services::{self, disk_manager, zip_directory};
use crate::services::{
    makemkvcon,
    plex::{create_movie_dir, find_movie, find_season},
};
use crate::state::AppState;
use crate::templates::{self};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{Emitter, Manager, State};
use tauri_plugin_notification::NotificationExt;
use templates::render_error;

#[derive(Serialize, Deserialize)]
pub struct DiskTitle {
    title_id: u32,
}

#[derive(Serialize, Deserialize)]
pub struct Part {
    number: u32,
    title_id: u32,
}

#[derive(Serialize, Deserialize)]
pub struct Episode {
    episode_number: u32,
    title: String,
    disk_titles: Vec<DiskTitle>,
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct RipInfo {
    directory: PathBuf,
    titles: Vec<TitleInfo>,
    content: DiskContent,
}

#[tauri::command]
pub fn assign_episode_to_title(
    mvdb_id: u32,
    season_number: u32,
    episode_number: u32,
    title_id: u32,
    part: u16,
    app_state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, templates::Error> {
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
    set_optical_disk_as_season(&optical_disk, &tv, &season);
    match add_episode_to_title(&app_state, &optical_disk, episode, &part, &title_id) {
        Ok(_) => debug!("Added {title_id} to {mvdb_id} {season_number} {episode_number}"),
        Err(e) => return Err(e),
    }

    templates::seasons::render_title_selected(&app_state, season)
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
    let episode = match season
        .episodes
        .iter()
        .find(|e| e.episode_number == episode_number)
    {
        Some(episode) => episode,
        None => return templates::render_error("Failed to find episode to add to title"),
    };
    match remove_episode_from_title(&app_state, &optical_disk, episode, &title_id) {
        Ok(_) => debug!("Removed {title_id} to {mvdb_id} {season_number} {episode_number}"),
        Err(e) => return Err(e),
    }

    templates::seasons::render_title_selected(&app_state, season)
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
    spawn_rip(app_handle, disk_id);
    templates::disks::render_toast_progress(&None, &None)
}

#[tauri::command]
pub fn rip_movie(
    disk_id: u32,
    title_id: u32,
    mvdb_id: u32,
    app_state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, templates::Error> {
    // Make sure it is a DiskID object
    let disk_id = DiskId::from(disk_id);
    // Assign Optical Disk Title as movie type and mvdb ID
    let optical_disk = match app_state.find_optical_disk_by_id(&disk_id) {
        Some(optical_disk) => optical_disk,
        None => return render_error("Failed to find Optical Disk"),
    };
    // Create Dir from Movie and Make Sure Movie Exists in MVDB
    match find_movie(&app_handle, mvdb_id) {
        Ok(movie) => set_optical_disk_as_movie(&optical_disk, movie),
        Err(e) => return render_error(&e.message),
    };

    mark_title_rippable(optical_disk, title_id);
    spawn_rip(app_handle, disk_id);

    templates::disks::render_toast_progress(&None, &None)
}

fn emit_render_cards(
    state: &State<'_, AppState>,
    app_handle: &tauri::AppHandle,
    movie: &MovieResponse,
) {
    let result =
        templates::movies::render_cards(state, movie).expect("Failed to render movies/cards.html");
    app_handle
        .emit("disks-changed", result)
        .expect("Failed to emit disks-changed");
}

fn notify_movie_success(app_handle: &tauri::AppHandle, movie: &MovieResponse) {
    app_handle
        .notification()
        .builder()
        .title("Finished Ripping")
        .body(movie.title_year())
        .show()
        .unwrap();
}

fn notify_movie_backup_success(app_handle: &tauri::AppHandle, movie: &MovieResponse) {
    app_handle
        .notification()
        .builder()
        .title("Backup Finished Ripping")
        .body(format!(
            "Was Able to backup movie but not create MKV files for you {}",
            movie.title_year()
        ))
        .show()
        .unwrap();
}

fn notify_failure(app_handle: &tauri::AppHandle, error: &RipError) {
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

fn delete_dir(dir: &Path) {
    if let Err(error) = fs::remove_dir_all(dir) {
        error!("Failed to delete directory {}: {}", dir.display(), error);
    };
}

fn spawn_upload(app_handle: &tauri::AppHandle, file_path: &Path, rip_info: &RipInfo) {
    let app_handle = app_handle.clone();
    let path = file_path.to_owned();
    let directory = rip_info.directory.to_owned();

    tauri::async_runtime::spawn(async move {
        match services::ftp_uploader::upload(&app_handle, &path).await {
            Ok(_m) => {
                notify_movie_upload_success(&app_handle, &path);
                delete_dir(&directory);
            }
            Err(e) => notify_movie_upload_failure(&app_handle, &path, &e),
        };
    });
}

fn rename_ripped_title(
    app_handle: &tauri::AppHandle,
    title: &TitleInfo,
    disk_id: &DiskId,
    rip_titles: &[TitleInfo],
) -> Result<PathBuf, RipError> {
    debug!("Ripped title {}", title.id);
    let state = app_handle.state::<AppState>();
    match state.find_optical_disk_by_id(disk_id) {
        Some(optical_disk) => {
            let locked_disk = optical_disk.read().unwrap();
            match locked_disk.content.as_ref().unwrap() {
                DiskContent::Movie(movie) => rename_movie_file(title, movie),
                DiskContent::Tv(season) => rename_tv_file(title, season, rip_titles),
            }
        }
        None => Err(RipError {
            title: "Rip Failure".to_string(),
            message: "Optical Disk missing, can't access critical info to rename movie/tv show"
                .to_string(),
        }),
    }
}

async fn rip_title(
    app_handle: &tauri::AppHandle,
    disk_id: &DiskId,
    title: &TitleInfo,
    rip_info: &RipInfo,
) -> Result<PathBuf, RipError> {
    match makemkvcon::rip_title(app_handle, disk_id, &title.id, &rip_info.directory).await {
        Ok(_) => rename_ripped_title(app_handle, title, disk_id, &rip_info.titles),
        Err(e) => Err(RipError {
            title: "Rip Failure".into(),
            message: e,
        }),
    }
}

async fn back_disk(
    app_handle: &tauri::AppHandle,
    disk_id: &DiskId,
    rip_info: &RipInfo,
) -> Result<(), RipError> {
    match makemkvcon::back_disk(app_handle, disk_id, &rip_info.directory).await {
        Ok(_) => Ok(()),
        Err(e) => Err(RipError {
            title: "Backup Failure".into(),
            message: e,
        }),
    }
}

fn notify_tv_success(app_handle: &tauri::AppHandle, season: &TvSeasonContent, title: &TitleInfo) {
    app_handle
        .notification()
        .builder()
        .title("TV Show Completed".to_string())
        .body(format!(
            "{} {} {}",
            season.tv.title_year(),
            season.season.name,
            title.describe_content()
        ))
        .show()
        .unwrap();
}

fn build_info(app_handle: &tauri::AppHandle, disk_id: &DiskId) -> RipInfo {
    let state = app_handle.state::<AppState>();
    let optical_disk = state.find_optical_disk_by_id(disk_id).unwrap();
    {
        let locked_disk = optical_disk.read().unwrap();
        match locked_disk.content.as_ref().unwrap() {
            DiskContent::Movie(movie) => {
                let dir = create_movie_dir(movie);
                let titles = locked_disk
                    .titles
                    .lock()
                    .unwrap()
                    .iter()
                    .filter(|t| t.rip)
                    .cloned()
                    .collect::<Vec<_>>();
                RipInfo {
                    directory: dir,
                    titles,
                    content: DiskContent::Movie(movie.clone()),
                }
            }
            DiskContent::Tv(season) => {
                let dir = create_season_episode_dir(season);
                let titles = locked_disk
                    .titles
                    .lock()
                    .unwrap()
                    .iter()
                    .filter(|t| t.rip)
                    .cloned()
                    .collect::<Vec<_>>();
                RipInfo {
                    directory: dir,
                    titles,
                    content: DiskContent::Tv(season.clone()),
                }
            }
        }
    }
}

fn eject_disk(state: &State<'_, AppState>, disk_id: &DiskId) {
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

async fn process_titles(
    state: &State<'_, AppState>,
    app_handle: &tauri::AppHandle,
    disk_id: &DiskId,
    rip_info: &RipInfo,
) -> bool {
    let mut success = false;
    for title in &rip_info.titles {
        match rip_title(app_handle, disk_id, title, rip_info).await {
            Ok(file_path) => {
                success = true;
                match rip_info.content {
                    DiskContent::Tv(ref season) => {
                        notify_tv_success(app_handle, season, title);
                    }
                    DiskContent::Movie(ref movie) => {
                        notify_movie_success(app_handle, movie);
                        emit_render_cards(state, app_handle, movie);
                        spawn_upload(app_handle, &file_path, rip_info);
                    }
                };
            }
            Err(error) => {
                match rip_info.content {
                    DiskContent::Tv(ref _season) => {}
                    DiskContent::Movie(ref movie) => {
                        emit_render_cards(state, app_handle, movie);
                        match back_disk(app_handle, disk_id, rip_info).await {
                            Ok(_) => {
                                let dst_string =
                                    format!("{}/backup.zip", rip_info.directory.to_string_lossy());
                                let dst_file = Path::new(&dst_string);
                                match zip_directory::zip_dir(
                                    &rip_info.directory,
                                    dst_file,
                                    zip::CompressionMethod::Deflated,
                                ) {
                                    Ok(()) => {
                                        notify_movie_backup_success(app_handle, movie);
                                        spawn_upload(app_handle, dst_file, rip_info);
                                        delete_dir(&rip_info.directory);
                                    }
                                    Err(error) => {
                                        debug!("{error}");
                                        notify_failure(
                                            app_handle,
                                            &RipError {
                                                title: "Backup Failed".into(),
                                                message: format!(
                                                    "Was unable to zip Backup {}",
                                                    rip_info.directory.to_string_lossy()
                                                ),
                                            },
                                        );
                                    }
                                }
                            }
                            Err(error) => notify_failure(app_handle, &error),
                        };
                    }
                };

                notify_failure(app_handle, &error);
            }
        };
    }
    success
}

fn spawn_rip(app_handle: tauri::AppHandle, disk_id: DiskId) {
    tauri::async_runtime::spawn(async move {
        let rip_info = build_info(&app_handle, &disk_id);
        let state = app_handle.state::<AppState>();
        let success = process_titles(&state, &app_handle, &disk_id, &rip_info).await;
        if success {
            eject_disk(&state, &disk_id);
        }
    });
}
