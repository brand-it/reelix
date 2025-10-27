use super::InlineTemplate;
use crate::models::movie_db;
use crate::models::optical_disk_info::OpticalDiskInfo;
use crate::services::ftp_uploader;
use crate::state::AppState;
use askama::Template;
use tauri::State;

#[derive(Template)]
#[template(path = "movies/cards.html")]
pub struct MoviesCards<'a> {
    pub selected_disk: &'a Option<OpticalDiskInfo>,
}

#[derive(Template)]
#[template(path = "movies/cards.turbo.html")]
pub struct MoviesCardsTurbo<'a> {
    pub movies_cards: &'a MoviesCards<'a>,
}
#[derive(Template)]
#[template(path = "movies/show.turbo.html")]
pub struct MoviesShowTurbo<'a> {
    pub movies_show: &'a MoviesShow<'a>,
}

#[derive(Template)]
#[template(path = "movies/show.html")]
pub struct MoviesShow<'a> {
    pub movie: &'a movie_db::MovieResponse,
    pub certification: &'a Option<String>,
    pub ripped: &'a bool,
    pub movies_cards: &'a MoviesCards<'a>,
}

pub fn render_show(
    app_state: &State<'_, AppState>,
    movie: &movie_db::MovieResponse,
    certification: &Option<String>,
) -> Result<String, super::Error> {
    let ripped = ftp_uploader::file_exists(&movie.to_file_path(), app_state);
    let selected_disk = match app_state.selected_disk() {
        Some(disk) => {
            let disk_lock = disk.read().unwrap();
            Some(disk_lock.clone())
        }
        None => None,
    };
    let template = MoviesShowTurbo {
        movies_show: &MoviesShow {
            movie,
            certification,
            ripped: &ripped,
            movies_cards: &MoviesCards {
                selected_disk: &selected_disk,
            },
        },
    };
    super::render(template)
}

pub fn render_cards(
    app_state: &State<'_, AppState>,
    movie: &movie_db::MovieResponse,
) -> Result<String, super::Error> {
    let selected_disk = match app_state.selected_disk() {
        Some(disk) => {
            let disk_lock = disk.read().unwrap();
            Some(disk_lock.clone())
        }
        None => None,
    };
    let template = MoviesCardsTurbo {
        movies_cards: &MoviesCards {
            selected_disk: &selected_disk,
        },
    };
    super::render(template)
}
