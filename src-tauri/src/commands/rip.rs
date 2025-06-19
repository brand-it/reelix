use std::fs;
use std::path::PathBuf;

use super::helpers::{
    add_episode_to_title, mark_title_rippable, remove_episode_from_title, rename_movie_file,
    rename_tv_file, set_optical_disk_as_movie, set_optical_disk_as_season,
};
use crate::models::movie_db::MovieResponse;
use crate::models::optical_disk_info::{DiskContent, DiskId};
use crate::services;
use crate::services::plex::{create_season_episode_dir, find_tv};
use crate::services::{
    makemkvcon,
    plex::{create_movie_dir, find_movie, find_season},
};
use crate::state::AppState;
use crate::templates::{self};
use serde::{Deserialize, Serialize};
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

#[tauri::command]
pub fn assign_episode_to_title(
    mvdb_id: u32,
    season_number: u32,
    episode_number: u32,
    title_id: u32,
    part: u16,
    app_state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, templates::ApiError> {
    let optical_disk = match app_state.selected_disk() {
        Some(disk) => disk,
        None => return render_error(&app_state, "No current selected disk"),
    };
    let tv = match find_tv(&app_handle, mvdb_id) {
        Ok(tv) => tv,
        Err(e) => return render_error(&app_state, &e.message),
    };

    let season = match find_season(&app_handle, mvdb_id, season_number) {
        Ok(season) => season,
        Err(e) => return render_error(&app_state, &e.message),
    };

    let episode = match season
        .episodes
        .iter()
        .find(|e| e.episode_number == episode_number)
    {
        Some(episode) => episode,
        None => return templates::render_error(&app_state, "Could not find episode to assign"),
    };
    set_optical_disk_as_season(&optical_disk, &tv, &season);
    match add_episode_to_title(&app_state, &optical_disk, episode, &part, &title_id) {
        Ok(_) => println!(
            "Added {} to {} {} {}",
            title_id, mvdb_id, season_number, episode_number
        ),
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
) -> Result<String, templates::ApiError> {
    let optical_disk = match app_state.selected_disk() {
        Some(d) => d,
        None => return render_error(&app_state, "No current selected disk"),
    };
    let season = match find_season(&app_handle, mvdb_id, season_number) {
        Ok(season) => season,
        Err(e) => return render_error(&app_state, &e.message),
    };
    let episode = match season
        .episodes
        .iter()
        .find(|e| e.episode_number == episode_number)
    {
        Some(episode) => episode,
        None => {
            return templates::render_error(&app_state, "Failed to find episode to add to title")
        }
    };
    match remove_episode_from_title(&app_state, &optical_disk, episode, &title_id) {
        Ok(_) => println!(
            "Removed {} to {} {} {}",
            title_id, mvdb_id, season_number, episode_number
        ),
        Err(e) => return Err(e),
    }

    templates::seasons::render_title_selected(&app_state, season)
}

#[tauri::command]
pub fn rip_season(
    app_handle: tauri::AppHandle,
    app_state: State<'_, AppState>,
) -> Result<String, templates::ApiError> {
    let disk_id = app_state
        .selected_optical_disk_id
        .read()
        .unwrap()
        .to_owned();
    let disk_id = match disk_id {
        Some(id) => id,
        None => {
            println!("No optical disk is currently selected.");
            return templates::render_error(&app_state, "No selected disk");
        }
    };
    spawn_rip(app_handle, disk_id);
    templates::disks::render_toast_progress(&app_state, &None, &None)
}

#[tauri::command]
pub fn rip_movie(
    disk_id: u32,
    title_id: u32,
    mvdb_id: u32,
    app_state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, templates::ApiError> {
    // Make sure it is a DiskID object
    let disk_id = match DiskId::try_from(disk_id) {
        Ok(id) => id,
        Err(_e) => return render_error(&app_state, "Failed to Parse Disk ID"),
    };
    // Assign Optical Disk Title as movie type and mvdb ID
    let optical_disk = match app_state.find_optical_disk_by_id(&disk_id) {
        Some(optical_disk) => optical_disk,
        None => return render_error(&app_state, "Failed to find Optical Disk"),
    };
    // Create Dir from Movie and Make Sure Movie Exists in MVDB
    match find_movie(&app_handle, mvdb_id) {
        Ok(movie) => set_optical_disk_as_movie(&optical_disk, movie),
        Err(e) => return render_error(&app_state, &e.message),
    };

    mark_title_rippable(optical_disk, title_id);
    spawn_rip(app_handle, disk_id);

    templates::disks::render_toast_progress(&app_state, &None, &None)
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
        .body(&movie.title_year())
        .show()
        .unwrap();
}

fn notify_movie_failure(app_handle: &tauri::AppHandle, movie: &MovieResponse, error: &str) {
    app_handle
        .notification()
        .builder()
        .title(format!("Failure {}", movie.title_year(),))
        .body(format!("Failed to rename title {}", error))
        .show()
        .unwrap();
}

fn notify_movie_upload_success(app_handle: &tauri::AppHandle, file_path: &PathBuf) {
    app_handle
        .notification()
        .builder()
        .title(format!("Finished Upload Movie"))
        .body(format!("File Path {}", file_path.to_string_lossy()))
        .show()
        .unwrap();
}

fn notify_movie_upload_failure(app_handle: &tauri::AppHandle, file_path: &PathBuf, error: &str) {
    println!(
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

fn delete_file_and_dir(file_path: &PathBuf) {
    if let Err(response) = fs::remove_file(file_path) {
        println!(
            "Failed to delete file {} {:?} ",
            file_path.display(),
            response
        );
    };

    if let Some(parent_dir) = file_path.parent() {
        if parent_dir.is_dir() {
            if let Err(error) = fs::remove_dir(parent_dir) {
                eprintln!(
                    "Failed to delete directory {}: {}",
                    parent_dir.display(),
                    error
                );
            }
        }
    };
}

fn spawn_upload(app_handle: &tauri::AppHandle, file_path: &PathBuf) {
    let app_handle = app_handle.clone();
    let file_path = file_path.clone();

    tauri::async_runtime::spawn(async move {
        let state = app_handle.state::<AppState>();
        match services::ftp_uploader::upload(&state, &file_path).await {
            Ok(_m) => {
                notify_movie_upload_success(&app_handle, &file_path);
                delete_file_and_dir(&file_path);
            }
            Err(e) => notify_movie_upload_failure(&app_handle, &file_path, &e),
        };
    });
}

fn spawn_rip(app_handle: tauri::AppHandle, disk_id: DiskId) {
    tauri::async_runtime::spawn(async move {
        let state = app_handle.state::<AppState>();
        let optical_disk = state.find_optical_disk_by_id(&disk_id).unwrap();
        let (dir, rip_titles) = {
            let locked_disk = optical_disk.read().unwrap();
            match locked_disk.content.as_ref().unwrap() {
                DiskContent::Movie(movie) => {
                    let dir = create_movie_dir(&movie);
                    let titles = locked_disk
                        .titles
                        .lock()
                        .unwrap()
                        .iter()
                        .filter(|t| t.rip)
                        .cloned()
                        .collect::<Vec<_>>();
                    (dir, titles)
                }
                DiskContent::Tv(season) => {
                    let dir = create_season_episode_dir(&season);
                    let titles = locked_disk
                        .titles
                        .lock()
                        .unwrap()
                        .iter()
                        .filter(|t| t.rip)
                        .cloned()
                        .collect::<Vec<_>>();
                    (dir, titles)
                }
            }
        };

        for title in &rip_titles {
            match makemkvcon::rip_title(&app_handle, &disk_id, &title.id, &dir).await {
                Ok(_) => {
                    println!("Ripped title {}", title.id);
                    let state = app_handle.state::<AppState>();
                    if let Some(optical_disk) = state.find_optical_disk_by_id(&disk_id) {
                        let locked_disk = optical_disk.read().unwrap();
                        match locked_disk.content.as_ref().unwrap() {
                            DiskContent::Movie(movie) => match rename_movie_file(&title, &movie) {
                                Ok(file_path) => {
                                    notify_movie_success(&app_handle, movie);
                                    emit_render_cards(&state, &app_handle, movie);
                                    spawn_upload(&app_handle, &file_path);
                                }
                                Err(error) => {
                                    emit_render_cards(&state, &app_handle, movie);
                                    notify_movie_failure(&app_handle, movie, &error);
                                }
                            },
                            DiskContent::Tv(season) => {
                                match rename_tv_file(&title, &season, &rip_titles) {
                                    Ok(file_path) => {
                                        app_handle
                                            .notification()
                                            .builder()
                                            .title(format!(
                                                "{} {} Completed",
                                                season.tv.title_year(),
                                                season.season.name
                                            ))
                                            .body(format!(
                                                "File Path {}",
                                                &file_path.to_string_lossy().to_string()
                                            ))
                                            .show()
                                            .unwrap();
                                    }
                                    Err(e) => {
                                        app_handle
                                            .notification()
                                            .builder()
                                            .title(format!(
                                                "Failure {} {}",
                                                season.tv.title_year(),
                                                season.season.name
                                            ))
                                            .body(format!("Failed to rename title {}", e))
                                            .show()
                                            .unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    let optical_disk = state.find_optical_disk_by_id(&disk_id).unwrap();
                    let locked_disk = optical_disk.read().unwrap();
                    app_handle
                        .notification()
                        .builder()
                        .title(format!("Failure {}", locked_disk.name))
                        .body(format!("Error Ripping {}", e))
                        .show()
                        .unwrap();
                }
            }
        }
    });
}
