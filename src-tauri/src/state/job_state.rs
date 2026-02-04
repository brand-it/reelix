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
    pub last_emit: SystemTime,
}

impl Job {
    pub fn new(job_type: JobType, disk: Option<OpticalDiskInfo>) -> Self {
        Job {
            id: JobId::new(),
            status: JobStatus::Pending,
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
            last_emit: SystemTime::now(),
        }
    }

    /// Finds a TitleVideo in the job that matches the given TV episode, season, title, and part.
    ///
    /// How to use:
    /// ```rust,ignore
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
                        && title_video.title.id == title_id
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
            title_video.title.id == title.id
        })
    }

    pub fn add_title_video(&mut self, title: TitleInfo, video: Video) -> Result<(), StandardError> {
        self.validate_title_video_modifiable("add")?;
        let title_video = TitleVideo { title, video };
        self.title_videos.push(Arc::new(RwLock::new(title_video)));
        self.status = JobStatus::Ready;
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
    /// ```rust,ignore
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
        self.status == JobStatus::Pending || self.status == JobStatus::Ready
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
// it will go from ready -> ripping -> uploading -> finished
// The state at any point can go to error if something goes wrong
// defaults to Ready
#[derive(Default, Clone, Serialize, PartialEq)]
pub enum JobStatus {
    #[default]
    Pending,
    Ready,
    Processing,
    Finished,
    Error,
}

impl fmt::Display for JobStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JobStatus::Pending => write!(f, "Pending"),
            JobStatus::Ready => write!(f, "Ready"),
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
