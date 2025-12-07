use crate::models::movie_db::{SeasonEpisode, SeasonResponse, TvEpisode, TvResponse};
use crate::models::optical_disk_info::OpticalDiskInfo;
use crate::state::background_process_state::{copy_job_state, BackgroundProcessState};
use crate::state::job_state::{Job, JobStatus};
use crate::state::AppState;
use crate::templates::disks::DisksOptions;
use crate::templates::InlineTemplate;
use askama::Template;
use tauri::Manager;

#[derive(Template)]
#[template(path = "seasons/parts.html")]
pub struct SeasonsParts<'a> {
    pub selected_disk: &'a Option<OpticalDiskInfo>,
    pub episode: &'a Option<TvEpisode>,
    pub job: &'a Option<Job>,
}

impl SeasonsParts<'_> {
    pub fn selector_class(&self) -> &'static str {
        super::SEASONS_PARTS_SELECTOR_CLASS
    }
}

#[derive(Template)]
#[template(path = "seasons/show.turbo.html")]
pub struct SeasonsShowTurbo<'a> {
    pub seasons_show: &'a SeasonsShow<'a>,
}

#[derive(Template)]
#[template(path = "seasons/show.html")]
pub struct SeasonsShow<'a> {
    pub tv: &'a TvResponse,
    pub season: &'a SeasonResponse,
    pub seasons_episodes: &'a SeasonsEpisodes<'a>,
}

impl SeasonsShow<'_> {
    pub fn dom_id(&self) -> &'static str {
        super::SEARCH_RESULTS_ID
    }
}

#[derive(Template)]
#[template(path = "seasons/title_selected.turbo.html")]
pub struct SeasonsTitleSelectedTurbo<'a> {
    pub season_episodes: &'a SeasonsEpisodes<'a>,
    pub disks_options: &'a DisksOptions<'a>,
}
#[derive(Template)]
#[template(path = "seasons/episodes.html")]
pub struct SeasonsEpisodes<'a> {
    pub episodes: &'a Vec<SeasonsEpisode<'a>>,
}

#[derive(Template)]
#[template(path = "seasons/episode.html")]
pub struct SeasonsEpisode<'a> {
    pub episode: &'a SeasonEpisode,
    pub seasons_parts: &'a SeasonsParts<'a>,
}

impl SeasonsEpisode<'_> {
    pub fn dom_id(&self) -> String {
        format!("episode-{}", self.episode.id)
    }
}

pub fn render_show(
    app_handle: &tauri::AppHandle,
    tv: &TvResponse,
    season: &SeasonResponse,
) -> Result<String, super::Error> {
    let app_state = app_handle.state::<AppState>();
    let selected_disk = match app_state.selected_disk() {
        Some(disk) => {
            let disk_lock = disk.read().unwrap();
            Some(disk_lock.clone())
        }
        None => None,
    };
    let job = get_job(app_handle, &selected_disk);

    let parts = SeasonsParts {
        selected_disk: &selected_disk,
        episode: &None,
        job: &job,
    };
    let seasons_show_turbo = SeasonsShowTurbo {
        seasons_show: &SeasonsShow {
            tv,
            season,
            seasons_episodes: &SeasonsEpisodes {
                episodes: &season
                    .episodes
                    .iter()
                    .map(|ep| SeasonsEpisode {
                        episode: ep,
                        seasons_parts: &parts,
                    })
                    .collect::<Vec<SeasonsEpisode>>(),
            },
        },
    };
    super::render(seasons_show_turbo)
}

pub fn render_title_selected(
    app_handle: &tauri::AppHandle,
    season: SeasonResponse,
) -> Result<String, super::Error> {
    let app_state = app_handle.state::<AppState>();

    let selected_disk = match app_state.selected_disk() {
        Some(disk) => {
            let disk_lock = disk.read().unwrap();
            Some(disk_lock.clone())
        }
        None => None,
    };
    let optical_disks = app_state.clone_optical_disks();
    let job = get_job(app_handle, &selected_disk);
    let parts = SeasonsParts {
        selected_disk: &selected_disk,
        episode: &None,
        job: &job,
    };
    let episodes = season
        .episodes
        .iter()
        .map(|ep| SeasonsEpisode {
            episode: ep,
            seasons_parts: &parts,
        })
        .collect::<Vec<SeasonsEpisode>>();

    let seasons_episodes = SeasonsEpisodes {
        episodes: &episodes,
    };
    let disks_options = DisksOptions {
        optical_disks: &optical_disks,
        selected_disk: &selected_disk,
        job: &job,
    };
    let template = SeasonsTitleSelectedTurbo {
        season_episodes: &seasons_episodes,
        disks_options: &disks_options,
    };
    super::render(template)
}

fn get_job(app_handle: &tauri::AppHandle, selected_disk: &Option<OpticalDiskInfo>) -> Option<Job> {
    let background_process_state = app_handle.state::<BackgroundProcessState>();
    match selected_disk {
        Some(ref disk) => {
            let disk_id = disk.id;
            background_process_state
                .find_job(
                    Some(disk_id),
                    &None,
                    &[JobStatus::Pending, JobStatus::Ready, JobStatus::Processing],
                )
                .and_then(|j| copy_job_state(&Some(j)))
        }
        None => None,
    }
}
