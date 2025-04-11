use crate::models::optical_disk_info::{DiskId, OpticalDiskInfo};
use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard};
use tauri::State;
use tera::Tera;
// Structure to hold shared state, thread safe version
pub struct AppState {
    pub tera: Arc<Tera>,
    pub query: Arc<Mutex<String>>,
    pub the_movie_db_key: Arc<RwLock<String>>,
    pub optical_disks: Arc<Mutex<Vec<Arc<Mutex<OpticalDiskInfo>>>>>,
    pub selected_optical_disk_id: Arc<Mutex<Option<DiskId>>>,
}

impl AppState {
    pub fn find_optical_disk_by_id(&self, disk_id: &DiskId) -> Option<Arc<Mutex<OpticalDiskInfo>>> {
        let disks = self
            .optical_disks
            .lock()
            .expect("Failed to acquire lock on optical_disks in find_optical_disk_by_id command");
        for disk in disks.iter() {
            let disk_guard = disk
                .lock()
                .expect("Failed to acquire lock on disk in find_optical_disk_by_id command");
            if &disk_guard.id == disk_id {
                return Some(Arc::clone(disk));
            }
        }
        None
    }
}

pub fn get_api_key<'a>(state: &'a State<AppState>) -> RwLockReadGuard<'a, String> {
    state
        .the_movie_db_key
        .read()
        .expect("Failed to acquire read lock on get_api_key")
}
