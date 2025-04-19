use crate::models::movie_db::{MovieResponse, SeasonEpisode, SeasonResponse};
use crate::models::optical_disk_info::{DiskContent, OpticalDiskInfo};
use crate::models::title_info::TitleInfo;
use crate::services::plex::{create_movie_dir, create_season_episode_dir};
use crate::services::the_movie_db::Error;
use crate::services::{template, the_movie_db};
use crate::state::{get_api_key, AppState};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock, RwLockWriteGuard};
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
        title.rip = true
    }
}

pub fn mark_title_rippable(optical_disk: Arc<RwLock<OpticalDiskInfo>>, title_id: u32) {
    let locked_disk = optical_disk.write().unwrap();
    let mut titles = locked_disk.titles.lock().unwrap();
    let title = titles.iter_mut().find(|t| t.id == title_id).unwrap();
    title.rip = true;
}

pub fn remove_episode_from_title(title: &mut TitleInfo, episode: &SeasonEpisode) {
    if let Some(index) = title.content.iter().position(|e| e.id == episode.id) {
        title.content.remove(index);
        if title.content.len() < 1 {
            title.part = None;
            title.rip = false
        }
    } else {
        println!("episode not associated with title");
    }
}

pub fn rename_movie_file(title: &TitleInfo, movie: &MovieResponse) -> Result<PathBuf, String> {
    let dir = create_movie_dir(&movie);
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

/// Renames a ripped TV title file based on its episodes and part info.
///
/// - `title`: the TitleInfo for this rip, including filename, part, and content.
/// - `season`: metadata for the season (used for directory & naming).
/// - `all_titles`: slice of all TitleInfo objects being processed, to detect multi-part episodes.
pub fn rename_tv_file(
    title: &TitleInfo,
    season: &SeasonResponse,
    all_titles: &[TitleInfo],
) -> Result<PathBuf, String> {
    // Ensure the output directory exists and construct source path
    let dir = create_season_episode_dir(season);
    let filename = title
        .filename
        .as_ref()
        .ok_or_else(|| "Missing source filename".to_string())?;
    let from = dir.join(filename);

    // Check file existence
    if !fs::metadata(&from)
        .map_err(|_| "Failed to check if source file exists".to_string())?
        .is_file()
    {
        return Err("Source file does not exist".to_string());
    }

    // Clone episodes for detection, without assuming sorted order
    let episodes = title.content.clone();

    // Determine new file stem
    let file_stem = if episodes.len() > 1 {
        // Multi-episode title: compute min and max episode numbers and format range
        let (start, end) = episodes.iter().fold((u32::MAX, 0), |(min, max), e| {
            (
                std::cmp::min(min, e.episode_number),
                std::cmp::max(max, e.episode_number),
            )
        });
        format!(
            "{} - s{:02}e{:02}-e{:02}",
            season.title_year(),
            season.season_number,
            start,
            end
        )
    } else {
        // Single-episode title: decide between multi-part or named
        let ep = &episodes[0];
        let ep_num = ep.episode_number;
        // Count how many titles share this episode number
        let related = all_titles
            .iter()
            .filter(|t| t.content.len() == 1 && t.content[0].episode_number == ep_num)
            .count();
        if related > 1 {
            // True multi-part: append part number
            let part_num = title.part.unwrap_or(1);
            format!(
                "{} - s{:02}e{:02} - pt{}",
                season.title_year(),
                season.season_number,
                ep_num,
                part_num
            )
        } else {
            // Single episode: use the episode name
            format!(
                "{} - s{:02}e{:02} - {}",
                season.title_year(),
                season.season_number,
                ep_num,
                ep.name
            )
        }
    };

    // Preserve original extension
    let extension = from.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    let to = dir.join(format!("{}.{}", file_stem, extension));

    // Rename the file
    fs::rename(&from, &to).map_err(|_| "Failed to rename file".to_string())?;
    Ok(to)
}
