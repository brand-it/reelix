use super::InlineTemplate;
use crate::models::optical_disk_info::OpticalDiskInfo;
use crate::state::background_process_state::copy_job_state;
use crate::state::job_state::{JobStatus, JobType};
use crate::state::{background_process_state, AppState};
use crate::templates::movies::MoviesCards;
use crate::templates::seasons::SeasonsParts;
use askama::Template;
use tauri::State;

#[derive(Template)]
#[template(path = "disk_titles/options.turbo.html")]
pub struct DiskTitlesOptionsTurbo<'a> {
    pub seasons_parts: &'a SeasonsParts<'a>,
    pub movies_cards: &'a MoviesCards<'a>,
}

pub fn render_options(
    app_state: &State<'_, AppState>,
    background_process_state: &State<'_, background_process_state::BackgroundProcessState>,
) -> Result<String, super::Error> {
    let selected_disk: Option<OpticalDiskInfo> = match app_state.selected_disk() {
        Some(disk) => {
            let read = disk.read().unwrap();
            Some(read.clone())
        }
        None => None,
    };
    let job = match selected_disk {
        Some(ref disk) => {
            let disk_id = disk.id;
            let job = background_process_state.find_job(
                Some(disk_id),
                &Some(JobType::Ripping),
                &[JobStatus::Pending, JobStatus::Ready],
            );
            copy_job_state(&job)
        }
        None => None,
    };

    let template = DiskTitlesOptionsTurbo {
        seasons_parts: &SeasonsParts {
            selected_disk: &selected_disk,
            episode: &None,
            job: &job,
        },
        movies_cards: &MoviesCards {
            selected_disk: &selected_disk,
            job: &job,
        },
    };
    super::render(template)
}
