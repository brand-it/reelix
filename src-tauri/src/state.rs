use crate::models::optical_disk_info::{DiskId, OpticalDiskInfo};
use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard};
use tauri::State;
use tera::Tera;
// Structure to hold shared state, thread safe version
pub struct AppState {
    pub tera: Arc<Tera>,
    pub query: Arc<Mutex<String>>,
    pub the_movie_db_key: Arc<RwLock<String>>,
    pub optical_disks: Arc<RwLock<Vec<Arc<RwLock<OpticalDiskInfo>>>>>,
    pub selected_optical_disk_id: Arc<RwLock<Option<DiskId>>>,
}

impl AppState {
    pub fn selected_disk(&self) -> Option<Arc<RwLock<OpticalDiskInfo>>> {
        let disk_id = self
            .selected_optical_disk_id
            .read()
            .expect("failed to lock selected_optical_disk_id in find_selected_disk");
        match disk_id.as_ref() {
            Some(disk_id) => self.find_optical_disk_by_id(&disk_id),
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

pub fn get_api_key<'a>(state: &'a State<AppState>) -> RwLockReadGuard<'a, String> {
    state
        .the_movie_db_key
        .read()
        .expect("Failed to acquire read lock on get_api_key")
}
