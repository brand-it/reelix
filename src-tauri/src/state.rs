use crate::models::optical_disk_info::{DiskId, OpticalDiskInfo};
use log::debug;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, RwLock};

pub mod background_process_state;
pub mod job_state;
pub mod title_video;
// Structure to hold shared state, thread safe version
pub struct AppState {
    pub ftp_host: Arc<Mutex<Option<String>>>,
    pub ftp_movie_upload_path: Arc<Mutex<Option<PathBuf>>>,
    pub ftp_tv_upload_path: Arc<Mutex<Option<PathBuf>>>,
    pub ftp_pass: Arc<Mutex<Option<String>>>,
    pub ftp_user: Arc<Mutex<Option<String>>>,
    pub optical_disks: Arc<RwLock<Vec<Arc<RwLock<OpticalDiskInfo>>>>>,
    pub query: Arc<Mutex<String>>,
    pub selected_optical_disk_id: Arc<RwLock<Option<DiskId>>>,
    pub the_movie_db_key: Arc<Mutex<String>>,
    pub movies_dir: Arc<RwLock<PathBuf>>,
    pub tv_shows_dir: Arc<RwLock<PathBuf>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            ftp_host: Arc::new(Mutex::new(None)),
            ftp_movie_upload_path: Arc::new(Mutex::new(None)),
            ftp_pass: Arc::new(Mutex::new(None)),
            ftp_user: Arc::new(Mutex::new(None)),
            optical_disks: Arc::new(RwLock::new(Vec::<Arc<RwLock<OpticalDiskInfo>>>::new())),
            query: Arc::new(Mutex::new(String::new())),
            selected_optical_disk_id: Arc::new(RwLock::new(None)),
            the_movie_db_key: Arc::new(Mutex::new(String::new())),
            movies_dir: Arc::new(RwLock::new(Self::default_movies_dir())),
            tv_shows_dir: Arc::new(RwLock::new(Self::default_tv_shows_dir())),
            ftp_tv_upload_path: Arc::new(Mutex::new(None)),
        }
    }

    pub fn save_query(&self, search: &str) {
        let mut query = self.query.lock().unwrap();
        *query = search.to_string();
    }

    fn default_movies_dir() -> PathBuf {
        dirs::home_dir()
            .expect("failed to find home dir")
            .join("Movies")
    }

    fn default_tv_shows_dir() -> PathBuf {
        dirs::home_dir()
            .expect("failed to find home dir")
            .join("TV Shows")
    }

    pub fn lock_the_movie_db_key(&self) -> MutexGuard<'_, String> {
        self.the_movie_db_key
            .lock()
            .expect("failed to lock the_movie_db_key")
    }
    pub fn lock_ftp_host(&self) -> MutexGuard<'_, Option<String>> {
        self.ftp_host.lock().expect("failed to lock ftp_host")
    }

    pub fn lock_ftp_user(&self) -> MutexGuard<'_, Option<String>> {
        self.ftp_user.lock().expect("failed to lock ftp_user")
    }

    pub fn lock_ftp_pass(&self) -> MutexGuard<'_, Option<String>> {
        self.ftp_pass.lock().expect("failed to lock ftp_pass")
    }

    pub fn lock_ftp_movie_upload_path(&self) -> MutexGuard<'_, Option<PathBuf>> {
        self.ftp_movie_upload_path
            .lock()
            .expect("failed to lock ftp_movie_upload_path")
    }

    pub fn update(&self, key: &str, value: Option<String>) -> Result<(), String> {
        let cleaned: Option<String> = value.and_then(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });
        debug!("Updating State {key} {cleaned:?}");
        match key {
            "ftp_host" => {
                let mut ftp_host = self.lock_ftp_host();
                *ftp_host = cleaned;
            }
            "ftp_user" => {
                let mut ftp_user = self.lock_ftp_user();
                *ftp_user = cleaned;
            }
            "ftp_pass" => {
                let mut ftp_pass = self.lock_ftp_pass();
                *ftp_pass = cleaned;
            }
            "ftp_movie_upload_path" => {
                let mut ftp_movie_upload_path = self.lock_ftp_movie_upload_path();
                *ftp_movie_upload_path = cleaned.map(PathBuf::from);
            }
            "the_movie_db_key" => {
                if let Some(val) = cleaned {
                    let mut the_movie_db_key = self.lock_the_movie_db_key();
                    *the_movie_db_key = val;
                };
            }
            "movies_dir" => {
                if let Some(val) = cleaned {
                    let mut movies_dir = self
                        .movies_dir
                        .write()
                        .expect("failed to lock movies_dir for write");
                    // validate path exists
                    if !movies_dir.exists() {
                        return Err(format!("movies_dir path does not exist: {val}"));
                    }
                    *movies_dir = PathBuf::from(val);
                };
            }
            "tv_shows_dir" => {
                if let Some(val) = cleaned {
                    let mut tv_shows_dir = self
                        .tv_shows_dir
                        .write()
                        .expect("failed to lock tv_shows_dir for write");
                    // validate path exists
                    if !tv_shows_dir.exists() {
                        return Err(format!("tv_shows_dir path does not exist: {val}"));
                    }
                    *tv_shows_dir = PathBuf::from(val);
                };
            }
            _ => return Err(format!("can't update {key}")),
        }
        Ok(())
    }

    pub fn clone_optical_disks(&self) -> Vec<OpticalDiskInfo> {
        let guard = self.optical_disks.read().unwrap();
        guard
            .iter()
            .map(|disk_arc| disk_arc.read().unwrap().to_owned())
            .collect()
    }

    pub fn selected_disk(&self) -> Option<Arc<RwLock<OpticalDiskInfo>>> {
        let disk_id = self
            .selected_optical_disk_id
            .read()
            .expect("failed to lock selected_optical_disk_id in find_selected_disk");
        match disk_id.as_ref() {
            Some(disk_id) => self.find_optical_disk_by_id(disk_id),
            None => None,
        }
    }

    pub fn find_optical_disk_by_id(
        &self,
        disk_id: &DiskId,
    ) -> Option<Arc<RwLock<OpticalDiskInfo>>> {
        let disks = self
            .optical_disks
            .read()
            .expect("Failed to acquire lock on optical_disks in find_optical_disk_by_id command");
        for disk in disks.iter() {
            let disk_guard = disk
                .read()
                .expect("Failed to acquire lock on disk in find_optical_disk_by_id command");
            if &disk_guard.id == disk_id {
                return Some(Arc::clone(disk));
            }
        }
        None
    }
}
