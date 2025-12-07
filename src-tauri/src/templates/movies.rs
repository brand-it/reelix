use super::InlineTemplate;
use crate::models::movie_db;
use crate::models::optical_disk_info::OpticalDiskInfo;
use crate::services::ftp_uploader;
use crate::state::background_process_state::{copy_job_state, BackgroundProcessState};
use crate::state::job_state::{Job, JobStatus};
use crate::state::title_video::Video;
use crate::state::{background_process_state, AppState};
use askama::Template;
use tauri::{Manager, State};

#[derive(Template)]
#[template(path = "movies/cards.html")]
pub struct MoviesCards<'a> {
    pub selected_disk: &'a Option<OpticalDiskInfo>,
    pub job: &'a Option<Job>,
    pub video: Option<&'a Video>,
}

impl MoviesCards<'_> {
    pub fn dom_id(&self) -> &'static str {
        super::MOVIE_CARDS_SELECTOR_DOM_ID
    }
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

impl MoviesShow<'_> {
    pub fn dom_id(&self) -> &'static str {
        super::SEARCH_RESULTS_ID
    }
}

pub fn render_show(
    app_state: &State<'_, AppState>,
    background_process_state: &State<'_, background_process_state::BackgroundProcessState>,
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

    let job = match &selected_disk {
        Some(disk) => background_process_state
            .find_job(Some(disk.id), &None, &[JobStatus::Processing])
            .and_then(|job_arc| copy_job_state(&Some(job_arc))),
        None => None,
    };
    let video = Video::Movie(Box::new(movie.clone()));
    app_state.save_current_video(Some(video.clone()));
    let template = MoviesShowTurbo {
        movies_show: &MoviesShow {
            movie,
            certification,
            ripped: &ripped,
            movies_cards: &MoviesCards {
                selected_disk: &selected_disk,
                job: &job,
                video: Some(&video),
            },
        },
    };
    super::render(template)
}

pub fn render_cards(app_handle: &tauri::AppHandle) -> Result<String, super::Error> {
    let app_state = app_handle.state::<AppState>();
    let background_process_state = app_handle.state::<BackgroundProcessState>();
    let selected_disk = match app_state.selected_disk() {
        Some(disk) => {
            let disk_lock = disk.read().unwrap();
            Some(disk_lock.clone())
        }
        None => None,
    };

    let video = match app_state.current_video.lock() {
        Ok(guard) => guard.clone(),
        Err(_) => return super::render_error("Failed to lock current video"),
    };

    let job = match &selected_disk {
        Some(disk) => background_process_state
            .find_job(Some(disk.id), &None, &[JobStatus::Processing])
            .and_then(|job_arc| copy_job_state(&Some(job_arc))),
        None => None,
    };

    let template = MoviesCardsTurbo {
        movies_cards: &MoviesCards {
            selected_disk: &selected_disk,
            job: &job,
            video: video.as_ref(),
        },
    };
    super::render(template)
}
