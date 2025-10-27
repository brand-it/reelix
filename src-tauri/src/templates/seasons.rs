use crate::models::movie_db::{SeasonEpisode, SeasonResponse, TvEpisode, TvResponse};
use crate::models::optical_disk_info::OpticalDiskInfo;
use crate::state::AppState;
use crate::templates::disks::DisksOptions;
use crate::templates::InlineTemplate;
use askama::Template;
use tauri::State;

#[derive(Template)]
#[template(path = "seasons/parts.html")]
pub struct SeasonsParts<'a> {
    pub selected_disk: &'a Option<OpticalDiskInfo>,
    pub episode: &'a Option<TvEpisode>,
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

pub fn render_show(
    app_state: &State<'_, AppState>,
    tv: &TvResponse,
    season: &SeasonResponse,
) -> Result<String, super::Error> {
    let selected_disk = match app_state.selected_disk() {
        Some(disk) => {
            let disk_lock = disk.read().unwrap();
            Some(disk_lock.clone())
        }
        None => None,
    };
    let parts = SeasonsParts {
        selected_disk: &selected_disk,
        episode: &None,
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
    app_state: &State<'_, AppState>,
    season: SeasonResponse,
) -> Result<String, super::Error> {
    let selected_disk = match app_state.selected_disk() {
        Some(disk) => {
            let disk_lock = disk.read().unwrap();
            Some(disk_lock.clone())
        }
        None => None,
    };
    let optical_disks = app_state.clone_optical_disks();

    let parts = SeasonsParts {
        selected_disk: &selected_disk,
        episode: &None,
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
    };
    let template = SeasonsTitleSelectedTurbo {
        season_episodes: &seasons_episodes,
        disks_options: &disks_options,
    };
    super::render(template)
}
