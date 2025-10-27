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

// Docs on how to build templates
// https://askama.readthedocs.io/en/stable/creating_templates.html
#[derive(Template)]
#[template(path = "error.html")]
pub struct GenericError<'a> {
    pub message: &'a str,
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

// #[derive(Template)]
// #[template(path = "index.html")]
// pub struct Index<'a> {
//     pub name: &'a str,
// }

// #[derive(Template)]
// #[template(path = "generic_error.html")]
// pub struct GenericError<'a> {
//     pub message: &'a str,
// }

// #[derive(Template)]
// #[template(path = "github/authorize.html")]
// pub struct GithubAuthorize<'a> {
//     pub device_code: &'a DeviceCode,
// }

// #[derive(Template)]
// #[template(path = "github/authorize_success.html")]
// pub struct GithubAuthorizeSuccess<'a> {
//     pub message: &'a String,
// }

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
