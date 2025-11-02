use crate::models::{movie_db::TvEpisode, title_info::TitleInfo};
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
// - CONTENT_ID must contain all dynamic content that can be updated without
//   reloading the entire page
// - ERROR_ID must be a container for error messages that can be updated
//   without reloading the entire page
//
// if you see a render_index function in any template file it should be using INDEX_ID as the target
// if you see a render_{something} function then it should be using CONTENT_ID or a target under CONTENT_ID
// you can update the ERROR_ID independently when needed within any template file.
// Use this as a guide when creating new templates and structuring existing ones.
pub const INDEX_ID: &str = "body"; // use action="update" target="body" to update entire page
pub const CONTENT_ID: &str = "content-browser";
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

pub fn find_previous_value(
    episode: &TvEpisode,
    part: &u16,
    titles: &Vec<TitleInfo>,
) -> Option<u32> {
    for title in titles {
        if title.part == Some(*part) && title.content.iter().any(|ep| ep.id == episode.id) {
            return Some(title.id);
        }
    }
    None
}

pub fn includes_episode(episode: &TvEpisode, title: &TitleInfo) -> bool {
    title.content.iter().any(|ep| ep.id == episode.id)
}

pub fn is_selected_title(episode: &TvEpisode, part: &u16, title: &TitleInfo) -> bool {
    title.part == Some(*part) && title.content.iter().any(|ep| ep.id == episode.id)
}
