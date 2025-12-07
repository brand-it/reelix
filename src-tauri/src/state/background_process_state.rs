use crate::models::optical_disk_info::{DiskId, OpticalDiskInfo};
use crate::state::job_state::{Job, JobStatus, JobType};
use std::sync::{Arc, RwLock};

pub struct BackgroundProcessState {
    // A list of all background jobs currently running
    // Each job is wrapped in an Arc<RwLock> to allow for concurrent access
    // Data can be read or modified from multiple threads safely
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

    pub fn new_job(&self, job_type: JobType, disk: Option<OpticalDiskInfo>) -> Arc<RwLock<Job>> {
        self.add_job(Job::new(job_type, disk))
    }

    pub fn find_job(
        &self,
        disk_id: Option<DiskId>,
        job_type: &Option<JobType>,
        job_states: &[JobStatus],
    ) -> Option<Arc<RwLock<Job>>> {
        let jobs = self.jobs.read().expect("lock jobs for read");
        jobs.iter().find_map(|job| {
            let job_guard = job.read().expect("lock job for read");
            let job_type_matches = match job_type {
                Some(ref jt) => job_guard.job_type == *jt,
                None => true,
            };
            match disk_id {
                None => {
                    if job_guard.disk.is_none()
                        && job_type_matches
                        && job_states.contains(&job_guard.status)
                    {
                        return Some(job.clone());
                    }
                }
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
        job_states: &[JobStatus],
    ) -> Arc<RwLock<Job>> {
        if let Some(job) = self.find_job(disk_id, &Some(job_type.clone()), job_states) {
            job
        } else {
            match optical_disk {
                None => self.new_job(job_type.clone(), None),
                Some(optical_disk) => {
                    let optical_disk_info = optical_disk.read().unwrap().clone();
                    self.new_job(job_type.clone(), Some(optical_disk_info))
                }
            }
        }
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
