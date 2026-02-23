use crate::models::optical_disk_info::OpticalDiskInfo;
use crate::services::ftp_uploader;
use crate::state::background_process_state::{copy_job_state, BackgroundProcessState};
use crate::state::job_state::{Job, JobStatus};
use crate::state::AppState;
use crate::templates::disks::DisksOptions;
use crate::templates::InlineTemplate;
use crate::the_movie_db::{SeasonEpisode, SeasonResponse, TvResponse};
use askama::Template;
use tauri::Manager;

#[derive(Template)]
#[template(path = "seasons/parts.html")]
pub struct SeasonsParts<'a> {
    pub selected_disk: &'a Option<OpticalDiskInfo>,
    pub job: &'a Option<Job>,
    pub episode_id: Option<u32>,
}

impl SeasonsParts<'_> {
    pub fn selector_class(&self) -> &'static str {
        super::SEASONS_PARTS_SELECTOR_CLASS
    }

    pub fn selectors_disabled(&self) -> bool {
        self.job.as_ref().is_some_and(Job::is_ripping)
    }

    /// Resolves the episode from the job's title_videos based on the episode ID.
    ///
    /// Purpose:
    /// - Extracts the SeasonEpisode data from the job's title_videos.
    /// - Searches for a matching TV episode by ID.
    /// - Returns the episode if one is already assigned in the job, otherwise None.
    /// - This allows the template to determine what episode data to use without
    ///   needing it to be pre-computed and passed in.
    #[cfg(test)]
    pub fn resolve_episode_from_job(&self) -> Option<SeasonEpisode> {
        let episode_id = self.episode_id?; // Return None if no episode_id is set
        match &self.job {
            Some(job) => job.title_videos.iter().find_map(|title_video| {
                let tv = title_video.read().unwrap();
                match &tv.video {
                    crate::state::title_video::Video::Tv(tv_episode) => {
                        // Only return the episode if its ID matches this SeasonsParts instance's episode_id
                        if tv_episode.episode.id == episode_id {
                            Some(tv_episode.episode.clone())
                        } else {
                            None
                        }
                    }
                    crate::state::title_video::Video::Movie(_) => None,
                }
            }),
            None => None,
        }
    }
}

#[derive(Template)]
#[template(path = "seasons/fab.html")]
pub struct SeasonsFab<'a> {
    pub job: &'a Option<Job>,
}

impl SeasonsFab<'_> {
    pub fn is_visible(&self) -> bool {
        !self.job.as_ref().is_some_and(Job::is_ripping)
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
    pub seasons_fab: &'a SeasonsFab<'a>,
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
    pub ripped: bool,
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
    let ripped_episode_numbers = ftp_uploader::tv_ripped_episode_numbers(tv, season, &app_state);
    let selected_disk = match app_state.selected_disk() {
        Some(disk) => {
            let disk_lock = disk.read().unwrap();
            Some(disk_lock.clone())
        }
        None => None,
    };
    let job = get_job(app_handle, &selected_disk);

    // Create individual SeasonsParts for each episode to ensure proper episode-specific resolution
    let episodes_with_parts: Vec<(SeasonsParts, &SeasonEpisode)> = season
        .episodes
        .iter()
        .map(|ep| {
            let parts = SeasonsParts {
                selected_disk: &selected_disk,
                job: &job,
                episode_id: Some(ep.id),
            };
            (parts, ep)
        })
        .collect();

    let episodes: Vec<SeasonsEpisode> = episodes_with_parts
        .iter()
        .map(|(parts, ep)| SeasonsEpisode {
            episode: ep,
            seasons_parts: parts,
            ripped: ripped_episode_numbers.contains(&ep.episode_number),
        })
        .collect();

    let seasons_show_turbo = SeasonsShowTurbo {
        seasons_show: &SeasonsShow {
            tv,
            season,
            seasons_episodes: &SeasonsEpisodes {
                episodes: &episodes,
            },
            seasons_fab: &SeasonsFab { job: &job },
        },
    };
    super::render(seasons_show_turbo)
}

pub fn render_title_selected(
    app_handle: &tauri::AppHandle,
    tv: &TvResponse,
    season: SeasonResponse,
) -> Result<String, super::Error> {
    let app_state = app_handle.state::<AppState>();
    let ripped_episode_numbers = ftp_uploader::tv_ripped_episode_numbers(tv, &season, &app_state);

    let selected_disk = match app_state.selected_disk() {
        Some(disk) => {
            let disk_lock = disk.read().unwrap();
            Some(disk_lock.clone())
        }
        None => None,
    };
    let optical_disks = app_state.clone_optical_disks();
    let job = get_job(app_handle, &selected_disk);

    // Create individual SeasonsParts for each episode to ensure proper episode-specific resolution
    let episodes_with_parts: Vec<(SeasonsParts, &SeasonEpisode)> = season
        .episodes
        .iter()
        .map(|ep| {
            let parts = SeasonsParts {
                selected_disk: &selected_disk,
                job: &job,
                episode_id: Some(ep.id),
            };
            (parts, ep)
        })
        .collect();

    let episodes: Vec<SeasonsEpisode> = episodes_with_parts
        .iter()
        .map(|(parts, ep)| SeasonsEpisode {
            episode: ep,
            seasons_parts: parts,
            ripped: ripped_episode_numbers.contains(&ep.episode_number),
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
                    &[JobStatus::Pending, JobStatus::Processing],
                )
                .and_then(|j| copy_job_state(&Some(j)))
        }
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::title_video::{TitleVideo, TvSeasonEpisode, Video};
    use crate::the_movie_db::{SeasonEpisode, SeasonResponse, TvResponse};
    use std::sync::{Arc, RwLock};

    /// Helper function to create a minimal mock SeasonEpisode for testing
    fn create_mock_episode(id: u32, episode_number: u32, name: &str) -> SeasonEpisode {
        SeasonEpisode {
            id,
            episode_number,
            episode_type: "standard".to_string(),
            name: name.to_string(),
            overview: "Test episode".to_string(),
            air_date: Some("2020-01-01".to_string()),
            production_code: None,
            runtime: Some(45),
            season_number: 1,
            show_id: 1,
            still_path: None,
            vote_average: 8.0,
            vote_count: 100,
            crew: vec![],
            guest_stars: vec![],
        }
    }

    /// Helper function to create a minimal mock SeasonResponse for testing
    fn create_mock_season_response() -> SeasonResponse {
        SeasonResponse {
            _id: "test_id".to_string(),
            id: 100,
            season_number: 1,
            name: "Season 1".to_string(),
            overview: "Test season".to_string(),
            poster_path: None,
            air_date: Some("2020-01-01".to_string()),
            episodes: vec![],
            vote_average: 8.5,
        }
    }

    /// Helper function to create a minimal mock TvResponse for testing
    fn create_mock_tv_response() -> TvResponse {
        TvResponse {
            adult: false,
            backdrop_path: None,
            created_by: vec![],
            episode_run_time: vec![45],
            first_air_date: Some("2020-01-01".to_string()),
            genres: vec![],
            homepage: None,
            id: 1,
            in_production: false,
            languages: vec!["en".to_string()],
            last_air_date: None,
            last_episode_to_air: None,
            name: "Test Show".to_string(),
            networks: vec![],
            next_episode_to_air: None,
            number_of_episodes: 20,
            number_of_seasons: 2,
            origin_country: vec!["US".to_string()],
            original_language: "en".to_string(),
            original_name: "Test Show".to_string(),
            overview: "Test overview".to_string(),
            popularity: 100.0,
            poster_path: None,
            production_companies: vec![],
            production_countries: vec![],
            seasons: vec![],
            spoken_languages: vec![],
            status: "Returning Series".to_string(),
            tagline: "Test tagline".to_string(),
            type_: "Scripted".to_string(),
            vote_average: 8.5,
            vote_count: 1000,
        }
    }

    /// Helper function to create a mock Job with multiple TV episodes
    fn create_mock_job_with_episodes(episode_ids: Vec<u32>) -> Job {
        let title_videos: Vec<Arc<RwLock<TitleVideo>>> = episode_ids
            .iter()
            .map(|&id| {
                let episode = create_mock_episode(id, id, &format!("Episode {id}"));
                let tv_season_episode = TvSeasonEpisode {
                    episode,
                    season: create_mock_season_response(),
                    tv: create_mock_tv_response(),
                    part: None,
                };
                Arc::new(RwLock::new(TitleVideo {
                    id: crate::state::title_video::TitleVideoId::new(),
                    title: None,
                    video: Video::Tv(Box::new(tv_season_episode)),
                }))
            })
            .collect();

        Job::new(
            crate::state::job_state::JobType::Ripping,
            None,
            crate::state::job_state::JobStatus::Pending,
        )
        .with_title_videos(title_videos)
    }

    #[test]
    fn test_resolve_episode_from_job_with_matching_id() {
        // Create a job with episodes 1, 2, and 3
        let job = create_mock_job_with_episodes(vec![1, 2, 3]);

        // Create SeasonsParts for episode 2
        let parts = SeasonsParts {
            selected_disk: &None,
            job: &Some(job),
            episode_id: Some(2),
        };

        // Should resolve to episode 2 only
        let resolved = parts.resolve_episode_from_job();
        assert!(resolved.is_some());
        let episode = resolved.unwrap();
        assert_eq!(episode.id, 2);
        assert_eq!(episode.name, "Episode 2");
    }

    #[test]
    fn test_resolve_episode_from_job_with_non_matching_id() {
        // Create a job with episodes 1, 2, and 3
        let job = create_mock_job_with_episodes(vec![1, 2, 3]);

        // Create SeasonsParts for episode 99 (not in job)
        let parts = SeasonsParts {
            selected_disk: &None,
            job: &Some(job),
            episode_id: Some(99),
        };

        // Should not resolve any episode
        let resolved = parts.resolve_episode_from_job();
        assert!(resolved.is_none());
    }

    #[test]
    fn test_resolve_episode_from_job_with_no_job() {
        // Create SeasonsParts without a job
        let parts = SeasonsParts {
            selected_disk: &None,
            job: &None,
            episode_id: Some(1),
        };

        // Should not resolve any episode
        let resolved = parts.resolve_episode_from_job();
        assert!(resolved.is_none());
    }

    #[test]
    fn test_resolve_episode_from_job_with_first_episode() {
        // Create a job with episodes 1, 2, and 3
        let job = create_mock_job_with_episodes(vec![1, 2, 3]);

        // Create SeasonsParts for episode 1
        let parts = SeasonsParts {
            selected_disk: &None,
            job: &Some(job),
            episode_id: Some(1),
        };

        // Should resolve to episode 1
        let resolved = parts.resolve_episode_from_job();
        assert!(resolved.is_some());
        let episode = resolved.unwrap();
        assert_eq!(episode.id, 1);
        assert_eq!(episode.name, "Episode 1");
    }

    #[test]
    fn test_resolve_episode_from_job_with_last_episode() {
        // Create a job with episodes 1, 2, and 3
        let job = create_mock_job_with_episodes(vec![1, 2, 3]);

        // Create SeasonsParts for episode 3
        let parts = SeasonsParts {
            selected_disk: &None,
            job: &Some(job),
            episode_id: Some(3),
        };

        // Should resolve to episode 3
        let resolved = parts.resolve_episode_from_job();
        assert!(resolved.is_some());
        let episode = resolved.unwrap();
        assert_eq!(episode.id, 3);
        assert_eq!(episode.name, "Episode 3");
    }

    #[test]
    fn test_resolve_episode_from_job_each_episode_independent() {
        // Create a job with episodes 1, 2, and 3
        let job = create_mock_job_with_episodes(vec![1, 2, 3]);

        // Test that each episode_id returns its own episode
        for episode_id in 1..=3 {
            let parts = SeasonsParts {
                selected_disk: &None,
                job: &Some(job.clone()),
                episode_id: Some(episode_id),
            };

            let resolved = parts.resolve_episode_from_job();
            assert!(resolved.is_some());
            let episode = resolved.unwrap();
            assert_eq!(episode.id, episode_id);
            assert_eq!(episode.name, format!("Episode {episode_id}"));
        }
    }

    #[test]
    fn test_multiple_seasons_parts_instances_are_independent() {
        // Create a job with episodes 1, 2, and 3
        let job = create_mock_job_with_episodes(vec![1, 2, 3]);

        // Create multiple SeasonsParts instances with different episode_ids
        let parts1 = SeasonsParts {
            selected_disk: &None,
            job: &Some(job.clone()),
            episode_id: Some(1),
        };

        let parts2 = SeasonsParts {
            selected_disk: &None,
            job: &Some(job.clone()),
            episode_id: Some(2),
        };

        let parts3 = SeasonsParts {
            selected_disk: &None,
            job: &Some(job),
            episode_id: Some(3),
        };

        // Each should resolve to its own episode
        let resolved1 = parts1.resolve_episode_from_job().unwrap();
        let resolved2 = parts2.resolve_episode_from_job().unwrap();
        let resolved3 = parts3.resolve_episode_from_job().unwrap();

        assert_eq!(resolved1.id, 1);
        assert_eq!(resolved2.id, 2);
        assert_eq!(resolved3.id, 3);

        // Verify they are truly independent
        assert_ne!(resolved1.id, resolved2.id);
        assert_ne!(resolved2.id, resolved3.id);
        assert_ne!(resolved1.id, resolved3.id);
    }

    #[test]
    fn test_resolve_episode_from_job_with_none_episode_id() {
        // Create a job with episodes 1, 2, and 3
        let job = create_mock_job_with_episodes(vec![1, 2, 3]);

        // Create SeasonsParts without an episode_id
        let parts = SeasonsParts {
            selected_disk: &None,
            job: &Some(job),
            episode_id: None,
        };

        // Should not resolve any episode when episode_id is None
        let resolved = parts.resolve_episode_from_job();
        assert!(resolved.is_none());
    }
}
