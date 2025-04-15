use super::the_movie_db;
use crate::models::movie_db;
use crate::models::optical_disk_info::DiskId;
use crate::state::{get_api_key, AppState};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub fn create_dir(movie: &movie_db::MovieResponse) -> PathBuf {
    let home_dir = dirs::home_dir().expect("failed to find home dir");
    let dir = home_dir.join("Movies").join(movie.title_year());
    let message = format!("Failed to create {}", dir.display());
    if !dir.exists() {
        fs::create_dir_all(&dir).expect(&message);
    }
    dir
}

pub fn find_movie(
    app_handle: &AppHandle,
    id: u32,
) -> Result<movie_db::MovieResponse, the_movie_db::Error> {
    let state: tauri::State<AppState> = app_handle.state::<AppState>();
    let api_key = get_api_key(&state);

    let language = "en-US";
    let movie_db = the_movie_db::TheMovieDb::new(&api_key, &language);
    movie_db.movie(id)
}

pub fn rename_file(
    app_handle: &AppHandle,
    movie: &movie_db::MovieResponse,
    disk_id: DiskId,
    title_id: u32,
) -> Result<PathBuf, String> {
    let state: tauri::State<'_, AppState> = app_handle.state::<AppState>();

    match state.find_optical_disk_by_id(&disk_id) {
        Some(optical_disk) => {
            let locked_disk = optical_disk
                .read()
                .expect("failed to lock disk in rename_file");
            let dir = create_dir(&movie);

            let titles = &locked_disk
                .titles
                .lock()
                .expect("failed to lock titles in rename_file");
            let title = titles.iter().find(|t| t.id == title_id);

            let filename = title
                .expect("Failed to find title in rename_file")
                .filename
                .clone()
                .expect(&format!("failed to find file name for {}", title_id));
            let from = dir.join(filename);
            match fs::exists(&from) {
                Ok(exist) => {
                    if exist {
                        let extension = from.extension().and_then(|ext| ext.to_str()).unwrap_or("");
                        let to = dir.join(format!("{}.{}", movie.title_year(), extension));
                        match fs::rename(from, &to) {
                            Ok(_r) => return Ok(to),
                            Err(_e) => return Err("Failed to rename file".to_string()),
                        }
                    } else {
                        return Err("File does not exist failed to rename".to_string());
                    }
                }
                Err(_e) => return Err("failed to check if from file exists".to_string()),
            }
        }
        None => return Err("failed to rename disk".to_string()),
    };
}
