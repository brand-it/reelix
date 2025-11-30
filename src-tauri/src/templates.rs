use crate::{
    models::{movie_db::TvEpisode, title_info::TitleInfo},
    state::{job_state::Job, title_video::Video},
};
use askama::Template;
use log::debug;
use serde::Serialize;

// Trait to add render_html method to any Template implementor
pub trait InlineTemplate: Template {
    fn render_html(&self) -> String {
        render_html(self)
    }
}

// Blanket implementation for all Template types
impl<T: Template> InlineTemplate for T {}

pub mod disk_titles;
pub mod disks;
pub mod ftp_settings;
pub mod movies;
pub mod search;
pub mod seasons;
pub mod the_movie_db;
pub mod tvs;

// Common DOM IDs
// To help organize the targets for turbo stream updates I have defined
// some common DOM IDs here that can be used across multiple templates.
// Some rules that each of these IDs should follow:
// - INDEX_ID must contain content-browser and error IDs
//   reloading the entire page
// - ERROR_ID must be a container for error messages that can be updated
//   without reloading the entire page
//
// if you see a render_index function in any template file it should be using INDEX_ID as the target
// if you see a render_{something} function then it should be using CONTENT_ID or a target under CONTENT_ID
// you can update the ERROR_ID independently when needed within any template file.
// Use this as a guide when creating new templates and structuring existing ones.
pub const INDEX_ID: &str = "body"; // use action="update" target="body" to update entire page
pub const ERROR_ID: &str = "error";
// Sub-IDs for specific sections within the content
pub const SEARCH_SUGGESTION_ID: &str = "search-suggestion";
pub const SEARCH_RESULTS_ID: &str = "search-results";
pub const DISK_TOAST_PROGRESS_DOM_ID: &str = "disk-progress-footer";
pub const MOVIE_CARDS_SELECTOR_DOM_ID: &str = "movie-cards-selector";
pub const SEASONS_PARTS_SELECTOR_CLASS: &str = "seasons-parts-selector"; // targets="{{ .seasons-parts-selector }}" for multiple elements
pub const DISK_SELECTOR_DOM_ID: &str = "disk-selector";
pub const DISK_TOAST_PROGRESS_SUMMARY_DOM_ID: &str = "disk-progress-summary-footer";
pub const DISK_TOAST_PROGRESS_DETAILS_DOM_ID: &str = "disk-progress-details-footer";
// Docs on how to build templates
// https://askama.readthedocs.io/en/stable/creating_templates.html
#[derive(Template)]
#[template(path = "error.html")]
pub struct GenericError<'a> {
    pub message: &'a str,
}

impl GenericError<'_> {
    pub fn dom_id(&self) -> &'static str {
        ERROR_ID
    }
}

#[derive(Template)]
#[template(path = "error.turbo.html")]
pub struct GenericErrorTurbo<'a> {
    pub generic_error: &'a GenericError<'a>,
}

#[derive(Serialize, Debug)]
pub struct Error {
    pub message: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error {}", self.message)
    }
}
impl std::error::Error for Error {}

pub fn render<T: Template>(template: T) -> Result<String, Error> {
    match template.render() {
        Ok(result) => Ok(result),
        Err(e) => {
            debug!("Template rendering error: {e:#?}");
            let error = Error {
                message: format!("An error occurred during template rendering: {e}"),
            };
            Err(error)
        }
    }
}

pub fn render_html<T: Template>(template: T) -> String {
    match template.render() {
        Ok(result) => result,
        Err(e) => {
            let template_name = std::any::type_name::<T>();
            debug!("Template rendering error {template_name}: {e:#?}");
            format!("<!-- An error occurred during template rendering {template_name}: {e} -->")
        }
    }
}

pub fn render_error(message: &str) -> Result<String, Error> {
    let generic_error = GenericError { message };

    render(&generic_error)
}

// Helper functions

/// Finds the associated TitleVideo for a given episode and part.
///
/// Purpose:
/// - Searches through a list of TitleVideos to find one that matches the given episode and part.
/// - Skips any TitleVideo that is a Movie (explicitly ignored in the match).
/// - Used to determine which TitleVideo (if any) is associated with a specific episode and part number.
/// - Returns the episode id if a match is found, otherwise returns None.
///
/// This is useful for linking UI selections or previous state to the correct TitleVideo entry.
///
/// Example usage:
/// ```rust
/// if let Some(id) = find_previous_value(&episode, &part, &job) {
///     // Found the associated TitleVideo for this episode/part
/// }
/// ```
pub fn find_previous_value(episode: &TvEpisode, part: &u16, job: &Job) -> Option<u32> {
    for title_video in job.title_videos.iter() {
        match &title_video.read().unwrap().video {
            Video::Tv(tv) => {
                if tv.part == Some(*part) && tv.episode.id == episode.id {
                    return Some(episode.id);
                }
            }
            Video::Movie(_) => { /* skip movies */ }
        }
    }
    None
}

/// Checks if a job contains a TitleVideo that matches both the given episode and title.
///
/// How it works:
/// - Iterates through all TitleVideos in the job.
/// - For each TitleVideo, acquires a read lock and checks:
///   - If the TitleVideo is a TV episode (`Video::Tv`), compares both the episode id and title id.
///   - If both match, returns true.
///   - Skips movies (`Video::Movie`).
/// - Returns false if no matching TitleVideo is found.
///
/// Usage:
/// - Use this to determine if a specific episode is already associated with a given title in a job.
pub fn job_contains_episode_for_title(
    episode: &TvEpisode,
    title_info: &TitleInfo,
    job: &Job,
) -> bool {
    job.title_videos.iter().any(|title_video| {
        let title_video = title_video.read().unwrap();
        match &title_video.video {
            Video::Tv(tv) => tv.episode.id == episode.id && title_video.title.id == title_info.id,
            Video::Movie(_) => false,
        }
    })
}

/// Checks if the given episode, part, and title are currently selected in the job's title_videos.
///
/// How it works:
/// - Iterates through all TitleVideos in the job.
/// - For each TitleVideo, acquires a read lock and checks:
///   - If the TitleVideo is a TV episode (`Video::Tv`), compares:
///     - The part number matches the given part.
///     - The episode id matches the given episode.
///     - The title id matches the given title.
///   - If all match, returns true (this title is selected for this episode/part).
///   - If the TitleVideo is a movie (`Video::Movie`), always returns false.
///     - This is because movies are never "selected" in the UI—they are always ripped directly.
///     - The concept of selection only applies to TV episodes and their parts, not movies.
///     - Movies cannot be in a state where selection matters, so this function will never return true for a movie.
/// - Returns false if no matching TitleVideo is found.
///
/// Usage:
/// - Use this to determine if a specific episode/part/title combination is currently selected in a job.
pub fn is_selected_title(
    episode: &TvEpisode,
    part: &u16,
    title_info: &TitleInfo,
    job: &Job,
) -> bool {
    job.title_videos.iter().any(|title_video| {
        let title_video = title_video.read().unwrap();
        match &title_video.video {
            Video::Tv(tv) => {
                tv.part == Some(*part)
                    && tv.episode.id == episode.id
                    && title_video.title.id == title_info.id
            }
            // Movies are never selected—they are always ripped directly, so this is always false.
            Video::Movie(_) => false,
        }
    })
}
