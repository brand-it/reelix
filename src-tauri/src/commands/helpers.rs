use crate::models::movie_db::{MovieResponse, SeasonEpisode, SeasonResponse};
use crate::models::optical_disk_info::{DiskContent, OpticalDiskInfo};
use crate::models::title_info::TitleInfo;
use crate::services::plex::create_dir;
use crate::services::the_movie_db::Error;
use crate::services::{template, the_movie_db};
use crate::state::{get_api_key, AppState};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock, RwLockWriteGuard};
use sysinfo::Disk;
use tauri::State;
use tera::Context;

pub fn render_tmdb_error(
    state: &State<AppState>,
    error_message: &str,
) -> Result<String, template::ApiError> {
    let api_key = get_api_key(state);
    let mut context = Context::new();
    context.insert("code", "500");
    context.insert("message", &format!("Error from TMDB: {}", error_message));
    context.insert("api_key", &api_key.to_owned());
    template::render(&state.tera, "the_movie_db/show.html.turbo", &context, None)
}

pub fn render_error(
    state: &State<'_, AppState>,
    error_message: &str,
) -> Result<String, template::ApiError> {
    let api_error = template::ApiError {
        code: 500,
        message: error_message.to_owned(),
        api_key: None,
    };

    let context = Context::from_serialize(&api_error)
        .expect("Failed to serialize API error context in the_movie_db command");
    template::render(&state.tera, "error.html.turbo", &context, None)
}

pub fn get_movie_certification(
    movie_db: the_movie_db::TheMovieDb,
    id: u32,
) -> Result<Option<String>, Error> {
    let release_dates = match movie_db.movie_release_dates(id) {
        Ok(resp) => resp,
        Err(e) => return Err(e),
    };

    Ok(release_dates
        .results
        .iter()
        .find(|entry| entry.iso_3166_1 == "US")
        .and_then(|us| us.release_dates.first())
        .map(|rd| rd.certification.trim().to_string()))
}

pub fn get_query(state: &State<'_, AppState>) -> String {
    state.query.lock().unwrap().to_string()
}

pub fn save_query(state: &State<'_, AppState>, search: &str) {
    let mut query = state.query.lock().unwrap();
    *query = search.to_string();
}

pub fn render_search_index(state: &State<'_, AppState>) -> Result<String, template::ApiError> {
    let mut context = Context::new();
    context.insert("optical_disks", &state.optical_disks);
    context.insert("selected_disk", &state.selected_disk());
    let binding_selected_disk_id = state
        .selected_optical_disk_id
        .read()
        .expect("failed to lock selected optical disk id");
    let guard_selected_disk_id = binding_selected_disk_id.as_ref();
    if guard_selected_disk_id.is_some() {
        let disk_id = guard_selected_disk_id.unwrap().clone();
        context.insert("selected_optical_disk_id", &disk_id);
    }
    template::render(&state.tera, "search/index.html.turbo", &context, None)
}

pub fn set_optical_disk_as_movie(
    optical_disk: &Arc<RwLock<OpticalDiskInfo>>,
    movie: MovieResponse,
) {
    let mut locked_disk = optical_disk.write().unwrap();
    locked_disk.content = Some(DiskContent::Movie(movie));
}

pub fn set_optical_disk_as_season(
    optical_disk: &Arc<RwLock<OpticalDiskInfo>>,
    season: &SeasonResponse,
) {
    let mut locked_disk = optical_disk.write().unwrap();
    match locked_disk.content.as_ref().unwrap() {
        DiskContent::Movie(_) => {
            locked_disk.content = Some(DiskContent::Tv(season.clone()));
            clear_all_episodes_from_titles(&locked_disk);
        }
        DiskContent::Tv(season) => {
            if season.id != season.id {
                locked_disk.content = Some(DiskContent::Tv(season.clone()));
                clear_all_episodes_from_titles(&locked_disk);
            }
        }
    };
}

fn clear_all_episodes_from_titles(locked_disk: &RwLockWriteGuard<'_, OpticalDiskInfo>) {
    let mut locked_titles = locked_disk.titles.lock().unwrap();
    locked_titles.iter_mut().for_each(|t| t.content.clear());
}

pub fn add_episode_to_title(title: &mut TitleInfo, episode: &SeasonEpisode, part: &u16) {
    if title.content.iter().any(|e| e.id == episode.id) {
        println!("episode already associated with title");
    } else {
        title.part = Some(part.clone());
        title.content.push(episode.clone());
    }
}

pub fn remove_episode_from_title(title: &mut TitleInfo, episode: &SeasonEpisode) {
    if let Some(index) = title.content.iter().position(|e| e.id == episode.id) {
        title.content.remove(index);
        if title.content.len() < 1 {
            title.part = None;
        }
    } else {
        println!("episode not associated with title");
    }
}

pub fn rename_movie_file(movie: &MovieResponse, title: &TitleInfo) -> Result<PathBuf, String> {
    let dir = create_dir(&movie);
    let filename = title.filename.as_ref().unwrap();
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
