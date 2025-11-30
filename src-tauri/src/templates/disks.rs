use crate::state::background_process_state::{copy_job_state, BackgroundProcessState};
use crate::state::job_state::JobStatus;
use crate::state::AppState;
use crate::templates::movies::MoviesCards;
use crate::templates::seasons::SeasonsParts;
use crate::templates::InlineTemplate;
use crate::{models::optical_disk_info, state::job_state::Job};
use askama::Template;
use std::sync::{Arc, RwLock};
use tauri::{AppHandle, Emitter, Manager, State};

#[derive(Template)]
#[template(path = "disks/options.html")]
pub struct DisksOptions<'a> {
    pub optical_disks: &'a Vec<optical_disk_info::OpticalDiskInfo>,
    pub selected_disk: &'a Option<optical_disk_info::OpticalDiskInfo>,
    pub job: &'a Option<Job>,
}

impl DisksOptions<'_> {
    pub fn dom_id(&self) -> &'static str {
        super::DISK_SELECTOR_DOM_ID
    }
}

#[derive(Template)]
#[template(path = "disks/options.turbo.html")]
pub struct DisksOptionsTurbo<'a> {
    pub disks_options: &'a DisksOptions<'a>,
    pub seasons_parts: &'a SeasonsParts<'a>,
    pub movies_cards: &'a MoviesCards<'a>,
}

#[derive(Template)]
#[template(path = "disks/toast_progress.html")]
pub struct DisksToastProgress<'a> {
    pub disks_toast_progress_summary: &'a DisksToastProgressSummary<'a>,
    pub disks_toast_progress_details: &'a DisksToastProgressDetails<'a>,
}

impl DisksToastProgress<'_> {
    pub fn dom_id(&self) -> &'static str {
        super::DISK_TOAST_PROGRESS_DOM_ID
    }
}
#[derive(Template)]
#[template(path = "disks/toast_progress.turbo.html")]
pub struct DisksToastProgressTurbo<'a> {
    pub disks_toast_progress_summary: &'a DisksToastProgressSummary<'a>,
    pub disks_toast_progress_details: &'a DisksToastProgressDetails<'a>,
    pub movie_cards: &'a Option<MoviesCards<'a>>,
}

#[derive(Template)]
#[template(path = "disks/toast_progress_summary.html")]
pub struct DisksToastProgressSummary<'a> {
    pub job: &'a Option<Job>,
}

impl DisksToastProgressSummary<'_> {
    pub fn dom_id(&self) -> &'static str {
        super::DISK_TOAST_PROGRESS_SUMMARY_DOM_ID
    }
}

#[derive(Template)]
#[template(path = "disks/toast_progress_details.turbo.html")]
pub struct DisksToastProgressDetailsTurbo<'a> {
    pub disks_toast_progress_details: &'a DisksToastProgressDetails<'a>,
}

#[derive(Template)]
#[template(path = "disks/toast_progress_details.html")]
pub struct DisksToastProgressDetails<'a> {
    pub job: &'a Option<Job>,
}

impl DisksToastProgressDetails<'_> {
    pub fn dom_id(&self) -> &'static str {
        super::DISK_TOAST_PROGRESS_DETAILS_DOM_ID
    }
}

pub fn emit_disk_change(app_handle: &AppHandle) {
    let state = app_handle.state::<AppState>();
    let background_process_state = app_handle.state::<BackgroundProcessState>();

    let result =
        render_options(&state, &background_process_state).expect("Failed to render disks/options");
    app_handle
        .emit("disks-changed", result)
        .expect("Failed to emit disks-changed");
}

pub fn render_options(
    app_state: &State<'_, AppState>,
    background_process_state: &State<'_, BackgroundProcessState>,
) -> Result<String, super::Error> {
    let optical_disks = app_state.clone_optical_disks();

    let selected_disk: Option<optical_disk_info::OpticalDiskInfo> = match app_state.selected_disk()
    {
        Some(disk_arc) => {
            let guard = disk_arc.read().unwrap();
            Some(guard.to_owned())
        }
        None => None,
    };
    let job = &background_process_state
        .find_job(
            selected_disk.as_ref().map(|d| d.id),
            &None,
            &[JobStatus::Processing],
        )
        .and_then(|job_arc| copy_job_state(&Some(job_arc)));

    let disks_options = DisksOptions {
        optical_disks: &optical_disks,
        selected_disk: &selected_disk,
        job,
    };
    let seasons_parts = SeasonsParts {
        selected_disk: &selected_disk,
        episode: &None,
        job,
    };
    let movies_cards = MoviesCards {
        selected_disk: &selected_disk,
        job,
    };
    let disks_options_turbo = DisksOptionsTurbo {
        disks_options: &disks_options,
        seasons_parts: &seasons_parts,
        movies_cards: &movies_cards,
    };

    super::render(disks_options_turbo)
}

pub fn render_toast_progress(
    app_handle: &AppHandle,
    job: &Arc<RwLock<Job>>,
) -> Result<String, super::Error> {
    let app_state = app_handle.state::<AppState>();
    let selected_disk = match app_state.selected_disk() {
        Some(disk) => {
            let disk_lock = disk.read().unwrap();
            Some(disk_lock.clone())
        }
        None => None,
    };
    let job = copy_job_state(&Some(job.clone()));
    let movie_cards = if job.as_ref().and_then(|j| j.disk.as_ref()).is_none()
        || selected_disk.is_none()
        || job.as_ref().and_then(|j| j.disk.as_ref()).map(|d| d.id)
            != selected_disk.as_ref().map(|d| d.id)
    {
        None
    } else {
        Some(MoviesCards {
            selected_disk: &selected_disk,
            job: &job,
        })
    };

    let template = DisksToastProgressTurbo {
        disks_toast_progress_summary: &DisksToastProgressSummary { job: &job },
        disks_toast_progress_details: &DisksToastProgressDetails { job: &job },
        movie_cards: &movie_cards,
    };
    super::render(template)
}
