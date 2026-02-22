use crate::models::title_info::TitleInfo;
use crate::standard_error::StandardError;
use crate::state::title_video::{TitleVideo, Video};
use crate::{
    models::optical_disk_info::OpticalDiskInfo,
    progress_tracker::{self, components::TimeComponent},
};
use log::debug;
use serde::Serialize;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use std::{
    fmt,
    sync::atomic::{AtomicU64, Ordering},
};
use tauri::{AppHandle, Emitter};
use tokio::time::Duration;

#[derive(Serialize, Clone)]
pub struct Job {
    pub id: JobId,
    pub status: JobStatus,
    pub job_type: JobType,
    pub message: Option<String>,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub progress: JobProgress,
    pub disk: Option<OpticalDiskInfo>,
    pub title_videos: Vec<Arc<RwLock<TitleVideo>>>,
    pub current_title_video_id: Option<crate::state::title_video::TitleVideoId>,
    pub last_emit: SystemTime,
}

impl Job {
    pub fn new(job_type: JobType, disk: Option<OpticalDiskInfo>, status: JobStatus) -> Self {
        Job {
            id: JobId::new(),
            status,
            job_type,
            message: None,
            title: None,
            subtitle: None,
            progress: JobProgress {
                eta: TimeComponent::NO_TIME_ELAPSED_TEXT.to_string(),
                percent: 0.0,
            },
            disk,
            title_videos: Vec::new(),
            current_title_video_id: None,
            last_emit: SystemTime::now(),
        }
    }

    /// Builder method to add title_videos to a Job (useful for testing)
    #[cfg(test)]
    pub fn with_title_videos(mut self, title_videos: Vec<Arc<RwLock<TitleVideo>>>) -> Self {
        self.title_videos = title_videos;
        self
    }

    /// Returns all TV `TitleVideo` entries that belong to the same show/season/episode.
    ///
    /// Matching rules:
    /// - only `Video::Tv` entries are considered
    /// - `tv.id`, `season.id`, and `episode.id` must all match the provided values
    /// - entries whose lock cannot be read are skipped
    ///
    /// This is primarily used for multipart episodes, where multiple files share the
    /// same episode identity but differ by `part` (e.g. `pt1`, `pt2`).
    fn select_tv_title_video_parts(
        &self,
        mvdb_id: u32,
        season_number: u32,
        episode_number: u32,
    ) -> Vec<Arc<RwLock<TitleVideo>>> {
        let parts = self.title_videos.iter().filter(|tv| {
            if let Ok(guard) = tv.read() {
                if let Video::Tv(tv_ep) = &guard.video {
                    return tv_ep.tv.id == mvdb_id
                        && tv_ep.season.id == season_number
                        && tv_ep.episode.id == episode_number;
                }
            }
            false
        });
        parts.cloned().collect()
    }

    /// Returns true when more than one title entry is assigned to the same
    /// TV show/season/episode in this job.
    ///
    /// This lets us keep `part` metadata (e.g. `part=1`) on the assigned video
    /// while deciding at rip time whether `-pt1` is actually needed in filenames.
    pub fn has_multiple_parts(&self, title_video: &TitleVideo) -> bool {
        let (mvdb_id, season_number, episode_number) = match &title_video.video {
            Video::Tv(tv_season_episode) => (
                tv_season_episode.tv.id,
                tv_season_episode.season.id,
                tv_season_episode.episode.id,
            ),
            _ => return false,
        };
        self.select_tv_title_video_parts(mvdb_id, season_number, episode_number)
            .len()
            > 1
    }

    /// Finds a TitleVideo in the job that matches the given TV episode, season, title, and part.
    ///
    /// How to use:
    /// ```text
    /// let maybe_title_video = job.find_tv_title_video(mvdb_id, season_number, episode_number, title_id, Some(part));
    /// if let Some(title_video) = maybe_title_video {
    ///     // Do something with the matching TitleVideo
    /// }
    /// ```
    ///
    /// Returns Some(Arc<RwLock<TitleVideo>>) if found, otherwise None.
    pub fn find_tv_title_video(
        &self,
        mvdb_id: u32,
        season_number: u32,
        episode_number: u32,
        title_id: u32,
        part: Option<u16>,
    ) -> Option<Arc<RwLock<TitleVideo>>> {
        self.title_videos
            .iter()
            .find(|title_video| {
                let title_video = title_video.read().unwrap();
                if let Video::Tv(tv_season_episode) = &title_video.video {
                    tv_season_episode.tv.id == mvdb_id
                        && tv_season_episode.season.id == season_number
                        && tv_season_episode.episode.id == episode_number
                        && title_video.title.as_ref().map(|t| t.id) == Some(title_id)
                        && tv_season_episode.part == part
                } else {
                    false
                }
            })
            .cloned()
    }

    pub fn matching_title(&self, title: &TitleInfo) -> bool {
        self.title_videos.iter().any(|title_video| {
            let title_video = title_video.read().unwrap();
            title_video.title.as_ref().map(|t| t.id) == Some(title.id)
        })
    }

    pub fn has_incomplete_titles(&self) -> bool {
        self.title_videos.iter().any(|title_video| {
            let title_video = title_video.read().unwrap();
            title_video.title.is_none()
        })
    }

    pub fn add_title_video(&mut self, title: TitleInfo, video: Video) -> Result<(), StandardError> {
        self.validate_title_video_modifiable("add")?;
        let title_video = TitleVideo {
            id: crate::state::title_video::TitleVideoId::new(),
            title: Some(title),
            video,
        };
        self.title_videos.push(Arc::new(RwLock::new(title_video)));
        Ok(())
    }

    pub fn add_incomplete_video(&mut self, video: Video) -> Result<(), StandardError> {
        self.validate_title_video_modifiable("add")?;
        let title_video = TitleVideo {
            id: crate::state::title_video::TitleVideoId::new(),
            title: None,
            video,
        };
        self.title_videos.push(Arc::new(RwLock::new(title_video)));
        Ok(())
    }

    // pub fn remove_title_video(&mut self, title: &TitleInfo) -> Result<(), StandardError> {
    //     self.validate_title_video_modifiable("remove")?;
    //     self.title_videos
    //         .retain(|tv| tv.read().unwrap().title.id != title.id);
    //     if self.title_videos.is_empty() {
    //         self.status = JobStatus::Pending;
    //     }
    //     Ok(())
    // }

    // pub fn clear_title_videos(&mut self) -> Result<(), StandardError> {
    //     self.validate_title_video_modifiable("clear")?;
    //     self.title_videos.clear();
    //     self.status = JobStatus::Pending;
    //     Ok(())
    // }

    /// Update the job's progress and estimated time remaining using a progress tracker.
    ///
    /// Purpose:
    /// - Sets the job's progress percentage and ETA based on the current state of a `progress_tracker::Base`.
    /// - Used during long-running operations (ripping, uploading, etc.) to reflect real-time progress in the UI or logs.
    ///
    /// How to use:
    /// - Call this method whenever you want to update the job's progress, typically inside a loop or callback as work advances.
    /// - Pass a reference to a `progress_tracker::Base` that tracks the operation's progress and time.
    ///
    /// Example:
    /// ```text
    /// job.update_progress(&tracker);
    /// ```
    ///
    /// Notes:
    /// - This will overwrite the job's `progress` field with the latest values from the tracker.
    /// - The tracker should be updated externally as the operation proceeds.
    pub fn update_progress(&mut self, tracker: &progress_tracker::Base) {
        let percent = tracker.percentage_component.percentage();
        self.progress = JobProgress {
            eta: tracker.time_component.estimated(None),
            percent,
        };
    }

    /// Emits a progress change event for THIS JOB ONLY to the frontend UI.
    ///
    /// Purpose:
    /// - Renders only this job's progress using the `render_job_item` template.
    /// - Emits a `disks-changed` event to the frontend via Tauri for targeted update.
    /// - Used whenever THIS JOB'S progress changes to keep UI in sync without re-rendering all jobs.
    /// - Much more efficient than re-rendering all jobs every time one changes.
    ///
    /// How to use:
    /// - Call this method after updating this specific job's progress or status.
    /// - The frontend listens for the `disks-changed` event and updates only this job's progress display.
    pub fn emit_progress_change(&self, app_handle: &tauri::AppHandle) {
        debug!(
            "Emitting progress for {} (percentage={})",
            self.id,
            self.progress.formatted_percentage()
        );
        let result =
            crate::templates::jobs::render_job_item(self).expect("Failed to render job item");
        app_handle
            .emit("disks-changed", result)
            .expect("Failed to emit job-changed");
    }

    pub fn rate_limited_emit_progress_change(&mut self, app_handle: &tauri::AppHandle) {
        let now = SystemTime::now();
        if let Ok(duration) = now.duration_since(self.last_emit) {
            if duration >= Duration::from_secs(1) {
                self.emit_progress_change(app_handle);
                self.last_emit = now;
            }
        }
    }

    pub fn update_message(&mut self, message: &str) {
        self.message = Some(message.to_string());
    }

    pub fn update_title(&mut self, title_video: &TitleVideo) {
        let title = match title_video.video {
            Video::Movie(ref movie) => Some(movie.movie.title_year()),
            Video::Tv(ref tv) => Some(tv.title()),
        };
        self.title = title;
    }

    // pub fn update_subtitle(&mut self, title_video: &TitleVideo) {
    //     self.subtitle = match title_video.video {
    //         Video::Movie(ref movie) => Some(movie.overview.clone()),
    //         Video::Tv(ref season) => Some(season.episode.overview.clone()),
    //     };
    // }

    pub fn update_status(&mut self, status: JobStatus) {
        self.status = status;
        if self.is_completed() {
            self.progress = JobProgress {
                eta: TimeComponent::NO_TIME_ELAPSED_TEXT.to_string(),
                percent: 100.0,
            };
        }
    }

    // pub fn reset(&mut self) {
    //     self.message = None;
    //     self.progress = JobProgress {
    //         eta: TimeComponent::NO_TIME_ELAPSED_TEXT.to_string(),
    //         percent: 0.0,
    //     };
    // }

    // pub fn update_job_type(&mut self, job_type: JobType) {
    //     self.reset();
    //     self.job_type = job_type;
    // }

    pub fn is_loading(&self) -> bool {
        self.is_processing() && self.job_type == JobType::Loading
    }

    pub fn is_ripping(&self) -> bool {
        self.is_processing() && self.job_type == JobType::Ripping
    }

    // pub fn is_uploading(&self) -> bool {
    //     self.is_processing() && self.job_type == JobType::Uploading
    // }

    pub fn is_modifiable(&self) -> bool {
        self.status == JobStatus::Pending
    }

    pub fn is_pending(&self) -> bool {
        self.status == JobStatus::Pending
    }

    pub fn is_processing(&self) -> bool {
        self.status == JobStatus::Processing
    }

    pub fn is_finished(&self) -> bool {
        self.status == JobStatus::Finished
    }

    pub fn is_error(&self) -> bool {
        self.status == JobStatus::Error
    }

    pub fn is_completed(&self) -> bool {
        self.status == JobStatus::Finished || self.status == JobStatus::Error
    }

    pub fn total_titles_count(&self) -> usize {
        self.title_videos.len()
    }

    pub fn current_title_position(&self) -> Option<usize> {
        let current_id = self.current_title_video_id?;
        self.title_videos
            .iter()
            .position(|title_video| title_video.read().map(|tv| tv.id == current_id).unwrap_or(false))
            .map(|index| index + 1)
    }

    pub fn completed_titles_count(&self) -> usize {
        let total = self.total_titles_count();
        if total == 0 {
            return 0;
        }

        if self.is_finished() {
            return total;
        }

        self.current_title_position().map_or(0, |position| position.saturating_sub(1))
    }

    pub fn remaining_titles_count(&self) -> usize {
        self.total_titles_count()
            .saturating_sub(self.completed_titles_count())
    }

    pub fn overall_progress_percent(&self) -> f64 {
        let total = self.total_titles_count();
        if total == 0 {
            return 0.0;
        }

        if self.is_finished() {
            return 100.0;
        }

        let completed = self.completed_titles_count() as f64;
        let current_fraction = if self.is_processing() {
            ((self.progress.percent as f64) / 100.0).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let overall = (((completed + current_fraction) / total as f64) * 100.0).clamp(0.0, 100.0);

        // When actively processing with no progress yet, show at least 1% to indicate work in progress
        if self.is_processing() && overall < 1.0 {
            1.0
        } else {
            overall
        }
    }

    pub fn overall_progress_formatted_percentage(&self) -> String {
        format!("{}%", self.overall_progress_percent().round() as u8)
    }

    // Human-friendly label for finished states, used by templates
    // pub fn finished_label(&self) -> Option<&'static str> {
    //     if self.is_finished() {
    //         Some("Finished")
    //     } else if self.is_error() {
    //         Some("Error")
    //     } else {
    //         None
    //     }
    // }

    fn validate_title_video_modifiable(&self, action: &str) -> Result<(), StandardError> {
        if !self.is_modifiable() {
            return Err(StandardError {
                title: format!("Cannot {action} title video"),
                message: format!("Cannot {} title video while job is {}", action, self.status),
            });
        }
        Ok(())
    }
}

#[derive(Serialize, Clone)]
pub struct JobProgress {
    pub percent: f32,
    pub eta: String,
}

impl JobProgress {
    // Formatted percentage with no decimal places
    pub fn formatted_percentage(&self) -> String {
        format!("{:.0}%", self.percent)
    }
}

// Progress state will track the current state of DVD ripping
// it will go from pending -> ripping -> uploading -> finished
// The state at any point can go to error if something goes wrong
// defaults to Pending
#[derive(Default, Clone, Serialize, PartialEq)]
pub enum JobStatus {
    #[default]
    Pending,
    Processing,
    Finished,
    Error,
}

impl fmt::Display for JobStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JobStatus::Pending => write!(f, "Pending"),
            JobStatus::Processing => write!(f, "Processing"),
            JobStatus::Finished => write!(f, "Finished"),
            JobStatus::Error => write!(f, "Error"),
        }
    }
}

#[derive(Serialize, Clone, PartialEq)]
pub enum JobType {
    Loading,
    Ripping,
    Uploading,
}

impl fmt::Display for JobType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JobType::Loading => write!(f, "Loading"),
            JobType::Ripping => write!(f, "Ripping"),
            JobType::Uploading => write!(f, "Uploading"),
        }
    }
}

static NEXT_JOB_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Serialize, Clone, PartialEq, Eq, Copy, PartialOrd, Ord)]
pub struct JobId(u64);

impl JobId {
    pub fn new() -> Self {
        JobId(NEXT_JOB_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl fmt::Display for JobId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn emit_progress(app_handle: &AppHandle, job: &Arc<RwLock<Job>>, now: bool) {
    if now {
        job.write()
            .expect("failed to lock job for write")
            .emit_progress_change(app_handle);
    } else {
        job.write()
            .expect("failed to lock job for write")
            .rate_limited_emit_progress_change(app_handle);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::title_video::{MoviePartEdition, TitleVideoId, TvSeasonEpisode};
    use crate::the_movie_db::{MovieResponse, SeasonEpisode, SeasonResponse, TvResponse};

    fn create_mock_tv(show_id: u32, name: &str) -> TvResponse {
        TvResponse {
            adult: false,
            backdrop_path: None,
            created_by: vec![],
            episode_run_time: vec![45],
            first_air_date: Some("2020-01-01".to_string()),
            genres: vec![],
            homepage: None,
            id: show_id,
            in_production: false,
            languages: vec!["en".to_string()],
            last_air_date: None,
            last_episode_to_air: None,
            name: name.to_string(),
            networks: vec![],
            next_episode_to_air: None,
            number_of_episodes: 10,
            number_of_seasons: 1,
            origin_country: vec!["US".to_string()],
            original_language: "en".to_string(),
            original_name: name.to_string(),
            overview: "Test show".to_string(),
            popularity: 1.0,
            poster_path: None,
            production_companies: vec![],
            production_countries: vec![],
            seasons: vec![],
            spoken_languages: vec![],
            status: "Ended".to_string(),
            tagline: "".to_string(),
            type_: "Scripted".to_string(),
            vote_average: 8.0,
            vote_count: 100,
        }
    }

    fn create_mock_episode(show_id: u32, season_number: u32, episode_number: u32) -> SeasonEpisode {
        SeasonEpisode {
            air_date: Some("2020-01-01".to_string()),
            episode_number,
            episode_type: "standard".to_string(),
            id: episode_number,
            name: format!("Episode {episode_number}"),
            overview: "Test episode".to_string(),
            production_code: None,
            runtime: Some(45),
            season_number,
            show_id,
            still_path: None,
            vote_average: 7.0,
            vote_count: 10,
            crew: vec![],
            guest_stars: vec![],
        }
    }

    fn create_mock_season(
        season_id: u32,
        season_number: u32,
        episodes: Vec<SeasonEpisode>,
    ) -> SeasonResponse {
        SeasonResponse {
            _id: format!("season-{season_id}"),
            air_date: Some("2020-01-01".to_string()),
            episodes,
            name: format!("Season {season_number}"),
            overview: "Test season".to_string(),
            id: season_id,
            poster_path: None,
            season_number,
            vote_average: 8.0,
        }
    }

    fn create_tv_title_video(
        show_id: u32,
        season_id: u32,
        season_number: u32,
        episode_number: u32,
        part: Option<u16>,
    ) -> Arc<RwLock<TitleVideo>> {
        let episode = create_mock_episode(show_id, season_number, episode_number);
        let season = create_mock_season(season_id, season_number, vec![episode.clone()]);
        let tv = create_mock_tv(show_id, "Test Show");

        Arc::new(RwLock::new(TitleVideo {
            id: TitleVideoId::new(),
            title: None,
            video: Video::Tv(Box::new(TvSeasonEpisode {
                episode,
                season,
                tv,
                part,
            })),
        }))
    }

    fn create_movie_title_video(movie_id: u32) -> Arc<RwLock<TitleVideo>> {
        Arc::new(RwLock::new(TitleVideo {
            id: TitleVideoId::new(),
            title: None,
            video: Video::Movie(Box::new(MoviePartEdition {
                movie: MovieResponse {
                    adult: false,
                    backdrop_path: None,
                    genres: vec![],
                    homepage: String::new(),
                    id: movie_id,
                    imdb_id: String::new(),
                    origin_country: vec![],
                    original_language: String::new(),
                    original_title: "Test Movie".to_string(),
                    overview: String::new(),
                    popularity: 0.0,
                    poster_path: None,
                    release_date: Some("2020-01-01".to_string()),
                    revenue: 0,
                    runtime: 90,
                    title: "Test Movie".to_string(),
                },
                part: None,
                edition: None,
            })),
        }))
    }

    #[test]
    fn select_tv_title_video_parts_returns_all_parts_for_matching_episode() {
        let match_part_1 = create_tv_title_video(100, 1, 1, 1, Some(1));
        let match_part_2 = create_tv_title_video(100, 1, 1, 1, Some(2));
        let different_episode = create_tv_title_video(100, 1, 1, 2, None);

        let job = Job::new(JobType::Ripping, None, JobStatus::Pending).with_title_videos(vec![
            match_part_1.clone(),
            match_part_2.clone(),
            different_episode,
        ]);

        let parts = job.select_tv_title_video_parts(100, 1, 1);

        assert_eq!(parts.len(), 2);
        assert!(parts.iter().any(|p| Arc::ptr_eq(p, &match_part_1)));
        assert!(parts.iter().any(|p| Arc::ptr_eq(p, &match_part_2)));
    }

    #[test]
    fn select_tv_title_video_parts_ignores_non_matching_and_movie_entries() {
        let matching_tv = create_tv_title_video(100, 1, 1, 1, None);
        let different_show = create_tv_title_video(101, 1, 1, 1, None);
        let different_season = create_tv_title_video(100, 2, 2, 1, None);
        let different_episode = create_tv_title_video(100, 1, 1, 2, None);
        let movie = create_movie_title_video(999);

        let job = Job::new(JobType::Ripping, None, JobStatus::Pending).with_title_videos(vec![
            matching_tv.clone(),
            different_show,
            different_season,
            different_episode,
            movie,
        ]);

        let parts = job.select_tv_title_video_parts(100, 1, 1);

        assert_eq!(parts.len(), 1);
        assert!(Arc::ptr_eq(&parts[0], &matching_tv));
    }

    #[test]
    fn select_tv_title_video_parts_returns_empty_when_no_matches_exist() {
        let job = Job::new(JobType::Ripping, None, JobStatus::Pending).with_title_videos(vec![]);

        let parts = job.select_tv_title_video_parts(100, 1, 1);

        assert!(parts.is_empty());
    }

    #[test]
    fn has_multiple_parts_returns_true_when_episode_has_multiple_parts() {
        let part1 = create_tv_title_video(100, 1, 1, 1, Some(1));
        let part2 = create_tv_title_video(100, 1, 1, 1, Some(2));

        let job = Job::new(JobType::Ripping, None, JobStatus::Pending)
            .with_title_videos(vec![part1.clone(), part2.clone()]);

        assert!(job.has_multiple_parts(&part1.read().unwrap()));
    }

    #[test]
    fn has_multiple_parts_returns_false_for_single_or_missing_matches() {
        let single = create_tv_title_video(100, 1, 1, 1, Some(1));
        let different_episode = create_tv_title_video(100, 9, 9, 9, Some(1));

        let job = Job::new(JobType::Ripping, None, JobStatus::Pending)
            .with_title_videos(vec![single.clone()]);

        assert!(!job.has_multiple_parts(&single.read().unwrap()));
        assert!(!job.has_multiple_parts(&different_episode.read().unwrap()));
    }
}
