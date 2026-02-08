use crate::models::optical_disk_info::{DiskId, OpticalDiskInfo};
use crate::state::job_state::{Job, JobStatus, JobType};
use std::sync::{Arc, RwLock};
use tauri::Emitter;

pub struct BackgroundProcessState {
    pub jobs: RwLock<Vec<Arc<RwLock<Job>>>>,
}

impl BackgroundProcessState {
    pub fn new() -> Self {
        Self {
            jobs: RwLock::new(Vec::new()),
        }
    }

    pub fn add_job(&self, job: Job) -> Arc<RwLock<Job>> {
        let job = Arc::new(RwLock::new(job));
        self.jobs
            .write()
            .expect("lock jobs for write")
            .push(job.clone());
        job
    }

    pub fn new_job(
        &self,
        job_type: JobType,
        job_state: JobStatus,
        disk: Option<OpticalDiskInfo>,
    ) -> Arc<RwLock<Job>> {
        self.add_job(Job::new(job_type, disk, job_state))
    }

    pub fn clone_all_jobs(&self) -> Vec<Job> {
        self.jobs
            .read()
            .expect("lock jobs for read")
            .iter()
            .map(|job| job.read().expect("lock job for read").clone())
            .collect()
    }

    pub fn emit_jobs_changed(&self, app_handle: &tauri::AppHandle) {
        let jobs = self.clone_all_jobs();
        let result = crate::templates::jobs::render_container(&jobs)
            .expect("Failed to render jobs container");
        app_handle
            .emit("disks-changed", result)
            .expect("Failed to emit jobs-changed");
    }

    /// Finds the first job matching the specified criteria.
    ///
    /// # Parameters
    /// - `disk_id`: Optional disk identifier. If `Some(id)`, matches jobs for that disk.
    ///   If `None`, matches jobs with no associated disk.
    /// - `job_type`: Optional job type filter. If `Some(type)`, only matches that type.
    ///   If `None`, matches any job type.
    /// - `job_states`: Array of acceptable job statuses. Job must be in one of these states.
    ///
    /// # Matching Rules
    /// A job matches if ALL of the following conditions are met:
    /// 1. Job type matches (or no type filter specified)
    /// 2. Disk ID matches (or is None for both search and job)
    /// 3. Job status is in the provided states array
    ///
    /// # Examples
    /// ```ignore
    /// // Find a ripping job for disk 1 that's currently processing
    /// let job = state.find_job(
    ///     Some(DiskId::from(1)),
    ///     &Some(JobType::Ripping),
    ///     &[JobStatus::Processing]
    /// );
    ///
    /// // Find any pending job for disk 2 (any type)
    /// let job = state.find_job(
    ///     Some(DiskId::from(2)),
    ///     &None,
    ///     &[JobStatus::Pending]
    /// );
    ///
    /// // Find an upload job with no disk association
    /// let job = state.find_job(
    ///     None,
    ///     &Some(JobType::Uploading),
    ///     &[JobStatus::Processing]
    /// );
    /// ```
    ///
    /// # Returns
    /// - `Some(Arc<RwLock<Job>>)` if a matching job is found
    /// - `None` if no job matches all criteria
    pub fn find_job(
        &self,
        disk_id: Option<DiskId>,
        job_type: &Option<JobType>,
        job_states: &[JobStatus],
    ) -> Option<Arc<RwLock<Job>>> {
        let jobs = self.jobs.read().expect("lock jobs for read");
        jobs.iter().find_map(|job| {
            let job_guard = job.read().expect("lock job for read");

            // Check if job type matches (None means match any type)
            let job_type_matches = match job_type {
                Some(ref jt) => job_guard.job_type == *jt,
                None => true,
            };

            match disk_id {
                // When searching for jobs with no disk association
                None => {
                    if job_guard.disk.is_none()
                        && job_type_matches
                        && job_states.contains(&job_guard.status)
                    {
                        return Some(job.clone());
                    }
                }
                // When searching for jobs with a specific disk
                Some(disk_id) => {
                    if let Some(ref disk) = job_guard.disk {
                        if disk.id == disk_id
                            && job_type_matches
                            && job_states.contains(&job_guard.status)
                        {
                            return Some(job.clone());
                        }
                    }
                }
            }
            None
        })
    }

    pub fn find_or_create_job(
        &self,
        disk_id: Option<DiskId>,
        optical_disk: &Option<Arc<RwLock<OpticalDiskInfo>>>,
        job_type: &JobType,
        job_state: &JobStatus,
    ) -> (Arc<RwLock<Job>>, bool) {
        if let Some(job) = self.find_job(disk_id, &Some(job_type.clone()), &[job_state.clone()]) {
            (job, false)
        } else {
            let job = match optical_disk {
                None => self.new_job(job_type.clone(), job_state.clone(), None),
                Some(optical_disk) => {
                    let optical_disk_info = optical_disk.read().unwrap().clone();
                    self.new_job(job_type.clone(), job_state.clone(), Some(optical_disk_info))
                }
            };
            (job, true)
        }
    }

    pub fn delete_job(&self, job_id: crate::state::job_state::JobId) {
        let mut jobs = self.jobs.write().expect("lock jobs for write");
        jobs.retain(|job| {
            let job_guard = job.read().expect("lock job for read");
            job_guard.id != job_id
        });
    }
}

pub fn copy_job_state(job: &Option<Arc<RwLock<Job>>>) -> Option<Job> {
    match job {
        Some(j) => {
            let job_guard = j.read().expect("failed to lock job for read");
            Some(job_guard.clone())
        }
        None => None,
    }
}
