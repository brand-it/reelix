use crate::models::movie_db::{MovieResponse, SeasonEpisode, SeasonResponse, TvResponse};
use crate::models::optical_disk_info::{DiskContent, OpticalDiskInfo, TvSeasonContent};
use crate::models::title_info::TitleInfo;
use crate::services::plex::{create_movie_dir, create_season_episode_dir};
use crate::state::AppState;
use crate::templates;
use log::debug;
use serde::{Deserialize, Serialize};
use std::fs::{self};
use std::path::PathBuf;
use std::sync::{Arc, RwLock, RwLockWriteGuard};
use tauri::State;

#[derive(Serialize, Deserialize, Clone)]
pub struct RipError {
    pub title: String,
    pub message: String,
}
impl std::fmt::Display for RipError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error {}: {}", self.title, self.message)
    }
}

pub fn save_query(state: &State<'_, AppState>, search: &str) {
    let mut query = state.query.lock().unwrap();
    *query = search.to_string();
}

pub fn set_optical_disk_as_movie(
    optical_disk: &Arc<RwLock<OpticalDiskInfo>>,
    movie: MovieResponse,
) {
    let mut locked_disk = optical_disk.write().unwrap();
    locked_disk.content = Some(DiskContent::Movie(Box::new(movie)));
}

pub fn set_optical_disk_as_season(
    optical_disk: &Arc<RwLock<OpticalDiskInfo>>,
    tv: &TvResponse,
    season: &SeasonResponse,
) {
    let mut locked_disk = optical_disk.write().unwrap();
    let tv_season_content = TvSeasonContent {
        season: season.clone(),
        tv: tv.clone(),
    };
    match locked_disk.content.as_ref() {
        Some(content) => {
            match content {
                DiskContent::Movie(_) => {
                    locked_disk.content = Some(DiskContent::Tv(Box::new(tv_season_content)));
                    clear_all_episodes_from_titles(&locked_disk);
                }
                DiskContent::Tv(content) => {
                    if content.season.id != season.id {
                        locked_disk.content = Some(DiskContent::Tv(Box::new(tv_season_content)));
                        clear_all_episodes_from_titles(&locked_disk);
                    }
                }
            };
        }
        None => locked_disk.content = Some(DiskContent::Tv(Box::new(tv_season_content))),
    };
}

pub fn add_episode_to_title(
    app_state: &State<'_, AppState>,
    optical_disk: &Arc<RwLock<OpticalDiskInfo>>,
    episode: &SeasonEpisode,
    part: &u16,
    title_id: &u32,
) -> Result<String, templates::ApiError> {
    match optical_disk.write() {
        Ok(locked_disk) => {
            let mut locked_titles = match locked_disk.titles.lock() {
                Ok(titles) => titles,
                Err(_e) => return templates::render_error(app_state, "Failed to lock titles"),
            };
            let title = match locked_titles.iter_mut().find(|t| &t.id == title_id) {
                Some(t) => t,
                None => return templates::render_error(app_state, "Failed to find Title"),
            };
            if title.content.iter().any(|e| e.id == episode.id) {
                debug!("episode already associated with title");
            } else {
                title.part = Some(*part);
                title.content.push(episode.clone());
                title.rip = true
            };
        }
        Err(_e) => return templates::render_error(app_state, "Failed to read disk"),
    };
    Ok("Success".to_string())
}

fn clear_all_episodes_from_titles(locked_disk: &RwLockWriteGuard<'_, OpticalDiskInfo>) {
    let mut locked_titles = locked_disk.titles.lock().unwrap();
    locked_titles.iter_mut().for_each(|t| t.content.clear());
}

pub fn mark_title_rippable(optical_disk: Arc<RwLock<OpticalDiskInfo>>, title_id: u32) {
    let locked_disk = optical_disk.write().unwrap();
    let mut titles = locked_disk.titles.lock().unwrap();
    let title = titles.iter_mut().find(|t| t.id == title_id).unwrap();
    title.rip = true;
}

pub fn remove_episode_from_title(
    app_state: &State<'_, AppState>,
    optical_disk: &Arc<RwLock<OpticalDiskInfo>>,
    episode: &SeasonEpisode,
    title_id: &u32,
) -> Result<String, templates::ApiError> {
    match optical_disk.write() {
        Ok(locked_disk) => {
            let mut locked_titles = match locked_disk.titles.lock() {
                Ok(titles) => titles,
                Err(_e) => return templates::render_error(app_state, "Failed to lock titles"),
            };
            let title = match locked_titles.iter_mut().find(|t| &t.id == title_id) {
                Some(t) => t,
                None => return templates::render_error(app_state, "Failed to find Title"),
            };

            if let Some(index) = title.content.iter().position(|e| e.id == episode.id) {
                title.content.remove(index);
                if title.content.is_empty() {
                    title.part = None;
                    title.rip = false
                }
            } else {
                debug!("episode not associated with title");
            };
        }
        Err(_e) => return templates::render_error(app_state, "Failed to read disk"),
    };
    Ok("success".to_string())
}

pub fn rename_movie_file(title: &TitleInfo, movie: &MovieResponse) -> Result<PathBuf, RipError> {
    let dir = create_movie_dir(movie);
    let filename = title.filename.as_ref().unwrap();
    let from = dir.join(filename);
    match fs::exists(&from) {
        Ok(exist) => {
            if exist {
                let extension = from.extension().and_then(|ext| ext.to_str()).unwrap_or("");
                let to = dir.join(format!("{}.{}", movie.title_year(), extension));
                match fs::rename(from, &to) {
                    Ok(_r) => Ok(to),
                    Err(_e) => Err(RipError {
                        title: "Rip Failure".to_string(),
                        message: format!("Failed to rename file {filename}"),
                    }),
                }
            } else {
                Err(RipError {
                    title: "Rip Failure".to_string(),
                    message: format!("File does not exist failed to rename movie {}", movie.title),
                })
            }
        }
        Err(_e) => Err(RipError {
            title: "Rip Failure".to_string(),
            message: format!("failed to create MKV file for movie {}", movie.title),
        }),
    }
}

/// Renames a ripped TV title file based on its episodes and part info.
///
/// - `title`: the TitleInfo for this rip, including filename, part, and content.
/// - `season`: metadata for the season (used for directory & naming).
/// - `all_titles`: slice of all TitleInfo objects being processed, to detect multi-part episodes.
pub fn rename_tv_file(
    title: &TitleInfo,
    content: &TvSeasonContent,
    all_titles: &[TitleInfo],
) -> Result<PathBuf, RipError> {
    // Ensure the output directory exists and construct source path
    let dir = create_season_episode_dir(content);
    let filename = title.filename.as_ref().ok_or_else(|| RipError {
        title: "Rip Failure".into(),
        message: format!(
            "Missing source filename can't rename file for {}",
            content.season.name
        ),
    })?;
    let from = dir.join(filename);

    // Check file existence
    if !fs::metadata(&from)
        .map_err(|_| RipError {
            title: "Rip Failure".into(),
            message: format!("Failed to create MKV file for {}", content.season.name),
        })?
        .is_file()
    {
        return Err(RipError {
            title: "Rip Failure".into(),
            message: "Source file does not exist".into(),
        });
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
            content.tv.title_year(),
            content.season.season_number,
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
                content.tv.title_year(),
                content.season.season_number,
                ep_num,
                part_num
            )
        } else {
            // Single episode: use the episode name
            format!(
                "{} - s{:02}e{:02} - {}",
                content.tv.title_year(),
                content.season.season_number,
                ep_num,
                ep.name
            )
        }
    };

    // Preserve original extension
    let extension = from.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    let to = dir.join(format!("{file_stem}.{extension}"));

    // Rename the file
    fs::rename(&from, &to).map_err(|_| RipError {
        title: "TV Show Error".into(),
        message: format!("Failed to rename TV show from {from:?} to {to:?}"),
    })?;
    Ok(to)
}
