use core::panic;

use super::helpers::{rename_movie_file, render_error, set_optical_disk_as_movie};
use crate::{
    models::{
        movie_db::MovieResponse,
        optical_disk_info::{DiskContent, DiskId},
    },
    services::{
        makemkvcon,
        plex::{create_dir, find_movie},
        template,
    },
    state::AppState,
};
use serde::{Deserialize, Serialize};
use tauri::{Manager, State};
use tauri_plugin_notification::NotificationExt;
use tera::Context;

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

#[derive(Serialize, Deserialize)]
pub struct SeasonData {
    season_number: u32,
    episodes: Vec<Episode>,
}

#[tauri::command]
pub fn rip_season(_season_data: SeasonData) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub fn rip_one(
    disk_id: u32,
    title_id: u32,
    mvdb_id: u32,
    app_state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, template::ApiError> {
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
    let movie_dir = match find_movie(&app_handle, mvdb_id) {
        Ok(movie) => {
            let dir = create_dir(&movie);
            set_optical_disk_as_movie(&optical_disk, movie);
            dir
        }
        Err(e) => return render_error(&app_state, &e.message),
    };

    spawn_movie_rip(app_handle, disk_id, title_id, movie_dir);

    template::render(
        &app_state.tera,
        "disks/toast_progress.html.turbo",
        &Context::new(),
        None,
    )
}

fn spawn_movie_rip(
    app_handle: tauri::AppHandle,
    disk_id: DiskId,
    title_id: u32,
    movie_dir: std::path::PathBuf,
) {
    tauri::async_runtime::spawn(async move {
        match makemkvcon::rip_title(&app_handle, &disk_id, &title_id, &movie_dir).await {
            Ok(_p) => {
                let state = app_handle.state::<AppState>();
                let optical_disk = state.find_optical_disk_by_id(&disk_id).unwrap();
                let title = optical_disk
                    .read()
                    .unwrap()
                    .titles
                    .lock()
                    .unwrap()
                    .iter()
                    .find(|t| t.id == title_id)
                    .unwrap()
                    .to_owned();
                let locked_disk = match optical_disk.read() {
                    Ok(disk) => disk,
                    Err(_e) => panic!("AHHHHHHH"),
                };
                let content = match &locked_disk.content {
                    Some(c) => c,
                    None => panic!("No content"),
                };
                let movie = match content {
                    DiskContent::Movie(m) => m,
                    DiskContent::Tv(_t) => panic!("MOVIE OH NO"),
                };
                let file_path = match rename_movie_file(&movie, &title) {
                    Ok(name) => name.to_string_lossy().to_string(),
                    Err(e) => {
                        return app_handle
                            .notification()
                            .builder()
                            .title("Reelix")
                            .body(format!("Failed to Rename File {}", e))
                            .show()
                            .unwrap();
                    }
                };
                app_handle
                    .notification()
                    .builder()
                    .title("Reelix")
                    .body(format!("Finished Ripping {}", &file_path))
                    .show()
                    .unwrap();
            }
            Err(e) => {
                app_handle
                    .notification()
                    .builder()
                    .title("Reelix")
                    .body(format!("Error Ripping {}", &e))
                    .show()
                    .unwrap();
            }
        }
    });
}
