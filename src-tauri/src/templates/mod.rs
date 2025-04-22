use crate::state::AppState;
use include_dir::{include_dir, Dir};
use serde::Serialize;
use std::error::Error;
use std::fmt;
use tauri::State;
use tera::Context;
use tera::Tera;

pub mod disk_titles;
pub mod disks;
pub mod movies;
pub mod search;
pub mod seasons;
pub mod the_movie_db;
pub mod tvs;

type ErrorHandler = fn(&tera::Error) -> ApiError;

#[derive(Serialize, Debug)]
pub struct ApiError {
    pub code: u16,
    pub message: String,
    pub api_key: Option<String>,
}
impl Error for ApiError {}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error {}: {}", self.code, self.message)
    }
}

pub fn render(
    tera: &Tera,
    template_path: &str,
    context: &Context,
    on_error: Option<ErrorHandler>,
) -> Result<String, ApiError> {
    match tera.render(template_path, context) {
        Ok(result) => Ok(result),
        Err(e) => {
            eprintln!("Template rendering error: {:#?}", e);
            // Custom error handler if provided
            if let Some(handler) = on_error {
                Err(handler(&e))
            } else {
                Err(ApiError {
                    code: 500,
                    message: format!("An error occurred during template rendering: {e}"),
                    api_key: None,
                })
            }
        }
    }
}

// Embed the `templates` directory into the binary
pub static TEMPLATES_DIR: Dir = include_dir!("templates");

pub fn add_templates_from_dir(tera: &mut Tera, dir: &Dir) {
    for file in dir.files() {
        if let Some(path) = file.path().to_str() {
            let content = file
                .contents_utf8()
                .expect("Failed to read file content as UTF-8");
            let name = path.replace("templates/", ""); // Strip the base path for Tera
            println!("Adding template: {}", name);
            tera.add_raw_template(&name, content)
                .expect("Failed to add template");
        }
    }

    for subdir in dir.dirs() {
        add_templates_from_dir(tera, subdir);
    }
}

pub fn render_error(state: &State<'_, AppState>, error_message: &str) -> Result<String, ApiError> {
    let api_error = ApiError {
        code: 500,
        message: error_message.to_owned(),
        api_key: None,
    };

    let context = Context::from_serialize(&api_error)
        .expect("Failed to serialize API error context in the_movie_db command");
    render(&state.tera, "error.html.turbo", &context, None)
}
