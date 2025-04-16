use crate::models::title_info;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use super::movie_db::MovieResponse;

#[derive(Serialize)]
pub struct OpticalDiskInfo {
    pub id: DiskId,
    pub name: String,
    pub available_space: u64,
    pub total_space: u64,
    pub file_system: String,
    pub is_removable: bool,
    pub is_read_only: bool,
    pub kind: String,
    pub disc_name: String, // AKA: Disk Name or Device Name
    pub titles: Mutex<Vec<title_info::TitleInfo>>,
    pub progress: Mutex<Option<Progress>>,
    pub pid: Mutex<Option<u32>>,
    pub movie_details: Mutex<Option<MovieResponse>>,
}

impl OpticalDiskInfo {
    pub fn set_movie_details(&self, movie_details: Option<MovieResponse>) {
        *self
            .movie_details
            .lock()
            .expect("failed to unlock movie details") = movie_details;
    }

    pub fn set_pid(&self, pid: Option<u32>) {
        *self.pid.lock().expect("failed to unlock pid") = pid;
    }

    pub fn set_progress(&self, progress: Option<Progress>) {
        *self.progress.lock().expect("failed to unlock progress") = progress;
    }
}
// Can't clone a Mutex so I'm going to do it my self because I need to be
// able to clone this object to use in the state management.
impl Clone for OpticalDiskInfo {
    fn clone(&self) -> Self {
        // Clone the titles by locking the mutex and cloning the inner vector.
        // Note: This will panic if the mutex is poisoned.
        // Code will try to recover from the poisoned state assuming the data is still usable
        let cloned_titles = self
            .titles
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        let cloned_progress = self
            .progress
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        let cloned_movie_details = self
            .movie_details
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        OpticalDiskInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            available_space: self.available_space,
            total_space: self.total_space,
            file_system: self.file_system.clone(),
            is_removable: self.is_removable,
            is_read_only: self.is_read_only,
            kind: self.kind.clone(),
            disc_name: self.disc_name.clone(),
            titles: Mutex::new(cloned_titles),
            progress: Mutex::new(cloned_progress),
            pid: Mutex::new(None),
            movie_details: Mutex::new(cloned_movie_details),
        }
    }
}

// Manually implement PartialEq
// I don't want to compare the titles because they can change state later on
// in the process
impl PartialEq for OpticalDiskInfo {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.available_space == other.available_space
            && self.total_space == other.total_space
            && self.file_system == other.file_system
            && self.is_removable == other.is_removable
            && self.is_read_only == other.is_read_only
            && self.kind == other.kind
    }
}

static NEXT_DISK_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Serialize, Clone, PartialEq, Copy)]
pub struct DiskId(u64);

impl DiskId {
    pub fn new() -> Self {
        DiskId(NEXT_DISK_ID.fetch_add(1, Ordering::Relaxed))
    }

    // pub fn from_any<Trait: Into<DiskId>>(id: Trait) -> Self {
    //     id.into()
    // }
}

// From unsigned types
impl From<u8> for DiskId {
    fn from(id: u8) -> Self {
        DiskId(id as u64)
    }
}

impl From<u16> for DiskId {
    fn from(id: u16) -> Self {
        DiskId(id as u64)
    }
}

impl From<u32> for DiskId {
    fn from(id: u32) -> Self {
        DiskId(id as u64)
    }
}

impl From<u64> for DiskId {
    fn from(id: u64) -> Self {
        DiskId(id)
    }
}

impl From<u128> for DiskId {
    fn from(id: u128) -> Self {
        DiskId(id as u64)
    }
}

impl From<usize> for DiskId {
    fn from(id: usize) -> Self {
        DiskId(id as u64)
    }
}

// From signed types
impl From<i8> for DiskId {
    fn from(id: i8) -> Self {
        DiskId(id as u64)
    }
}

impl From<i16> for DiskId {
    fn from(id: i16) -> Self {
        DiskId(id as u64)
    }
}

impl From<i32> for DiskId {
    fn from(id: i32) -> Self {
        DiskId(id as u64)
    }
}

impl From<i64> for DiskId {
    fn from(id: i64) -> Self {
        DiskId(id as u64)
    }
}

impl From<i128> for DiskId {
    fn from(id: i128) -> Self {
        DiskId(id as u64)
    }
}

impl From<isize> for DiskId {
    fn from(id: isize) -> Self {
        DiskId(id as u64)
    }
}

impl TryFrom<&str> for DiskId {
    type Error = std::num::ParseIntError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let parsed = s.parse::<u64>()?;
        Ok(DiskId(parsed))
    }
}

// --- Optical Progress ---
#[derive(Debug, Serialize, Clone)]
pub struct Progress {
    pub percentage: String,
    pub eta: String,
    pub label: String,
    pub message: String,
}
