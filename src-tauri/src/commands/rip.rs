use super::helpers::{
    add_episode_to_title, remove_episode_from_title, rename_movie_file, render_error,
    set_optical_disk_as_movie,
};
use crate::models::optical_disk_info::{DiskContent, DiskId};
use crate::services::{
    makemkvcon,
    plex::{create_dir, find_movie, find_season},
    template,
};
use crate::state::AppState;
use core::panic;
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

#[tauri::command]
pub fn assign_episode_to_title(
    mvdb_id: u32,
    season_number: u32,
    episode_number: u32,
    title_id: u32,
    part: u16,
    app_state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, template::ApiError> {
    let optical_disk = match app_state.selected_disk() {
        Some(d) => d,
        None => return render_error(&app_state, "No current selected disk"),
    };

    let locked_disk = match optical_disk.read() {
        Ok(disk) => disk,
        Err(e) => return render_error(&app_state, "Failed to read disk"),
    };
    let mut locked_titles = match locked_disk.titles.lock() {
        Ok(titles) => titles,
        Err(_e) => return render_error(&app_state, "Failed to lock titles"),
    };

    let title = match locked_titles.iter_mut().find(|t| t.id == title_id) {
        Some(t) => t,
        None => return render_error(&app_state, "Failed to find Title"),
    };

    let season = match find_season(&app_handle, mvdb_id, season_number) {
        Ok(season) => season,
        Err(e) => return render_error(&app_state, &e.message),
    };

    match season
        .episodes
        .iter()
        .find(|e| e.episode_number == episode_number)
    {
        Some(e) => add_episode_to_title(title, e, &part),
        None => return render_error(&app_state, "Failed to find episode number"),
    };
    println!(
        "Added {} to {} {} {}",
        title_id, mvdb_id, season_number, episode_number
    );
    println!("INspecting title {:?}", title);
    Ok("Success".to_string())
}

#[tauri::command]
pub fn withdraw_episode_from_title(
    mvdb_id: u32,
    season_number: u32,
    episode_number: u32,
    title_id: u32,
    app_state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, template::ApiError> {
    let optical_disk = match app_state.selected_disk() {
        Some(d) => d,
        None => return render_error(&app_state, "No current selected disk"),
    };

    let locked_disk = match optical_disk.read() {
        Ok(disk) => disk,
        Err(e) => return render_error(&app_state, "Failed to read disk"),
    };
    let mut locked_titles = match locked_disk.titles.lock() {
        Ok(titles) => titles,
        Err(_e) => return render_error(&app_state, "Failed to lock titles"),
    };

    let title = match locked_titles.iter_mut().find(|t| t.id == title_id) {
        Some(t) => t,
        None => return render_error(&app_state, "Failed to find Title"),
    };

    let season = match find_season(&app_handle, mvdb_id, season_number) {
        Ok(season) => season,
        Err(e) => return render_error(&app_state, &e.message),
    };

    match season
        .episodes
        .iter()
        .find(|e| e.episode_number == episode_number)
    {
        Some(e) => remove_episode_from_title(title, e),
        None => return render_error(&app_state, "Failed to find episode number"),
    };
    println!(
        "Removed {} to {} {} {}",
        title_id, mvdb_id, season_number, episode_number
    );
    Ok("Success".to_string())
}

// #[tauri::command]
// pub fn rip_season(_season_data: SeasonData) -> Result<(), String> {
//     Ok(())
// }

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
