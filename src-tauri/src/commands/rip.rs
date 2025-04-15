use crate::{
    models::optical_disk_info::DiskId,
    services::{
        converter::cast_to_u32,
        makemkvcon,
        plex::{create_dir, find_movie, rename_file},
        template,
    },
    state::AppState,
};
use serde::{Deserialize, Serialize};
use tauri::State;
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
    disk_id: &str,
    title_id: &str,
    mvdb_id: &str,
    app_state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, template::ApiError> {
    let title_id = cast_to_u32(title_id.to_string());
    let mvdb_id = cast_to_u32(mvdb_id.to_string());
    match DiskId::try_from(disk_id) {
        Ok(id) => match app_state.find_optical_disk_by_id(&id) {
            Some(optical_disk) => match find_movie(&app_handle, mvdb_id) {
                Ok(movie) => {
                    let movie_dir = create_dir(&movie);
                    optical_disk
                        .write()
                        .unwrap()
                        .set_movie_details(Some(movie.clone()));
                    tauri::async_runtime::spawn(async move {
                        let results =
                            makemkvcon::rip_title(&app_handle, &id, title_id, &movie_dir).await;
                        match results {
                            Ok(_r) => match rename_file(&app_handle, &movie, id, title_id) {
                                Ok(p) => {
                                    let file_path = p.to_string_lossy().to_string();
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
                            },
                            Err(message) => {
                                println!("failed {}", message);
                            }
                        }
                    });
                }
                Err(e) => eprintln!("Failure {}", e.message),
            },
            None => eprintln!("Failed to find optical disk"),
        },

        Err(e) => {
            eprintln!("Error parsing disk_id in rip_one: {}", e);
        }
    }

    template::render(
        &app_state.tera,
        "disks/toast_progress.html.turbo",
        &Context::new(),
        None,
    )
}
