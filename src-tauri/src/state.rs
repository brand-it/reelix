use crate::models::optical_disk_info::{DiskId, OpticalDiskInfo};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tera::Tera;
// Structure to hold shared state, thread safe version
pub struct AppState {
    pub tera: Arc<Tera>,
    pub the_movie_db_key: Arc<Mutex<String>>,
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

    pub fn find_optical_disk_by_mount_point(
        &self,
        mount_point: &str,
    ) -> Option<Arc<Mutex<OpticalDiskInfo>>> {
        let disks = self
            .optical_disks
            .lock()
            .expect("Failed to acquire lock on disks in find_optical_disk_by_mount_point command");
        for disk in disks.iter() {
            let disk_guard = disk.lock().expect(
            "Failed to acquire lock on a disk while iterating optical_disks in find_optical_disk_by_mount_point command",
        );
            if disk_guard.mount_point == PathBuf::from(mount_point) {
                return Some(Arc::clone(disk));
            }
        }
        None
    }
}
