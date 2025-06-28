use super::movie_db::{MovieResponse, SeasonResponse, TvResponse};
use super::title_info::TitleInfo;
use serde::Serialize;
use std::fmt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use sysinfo::{Pid, System};
#[derive(Serialize, Clone)]
pub struct TvSeasonContent {
    pub season: SeasonResponse,
    pub tv: TvResponse,
}

#[derive(Serialize, Clone)]
pub enum DiskContent {
    Tv(TvSeasonContent),
    Movie(MovieResponse),
}

#[derive(Serialize)]
pub struct OpticalDiskInfo {
    pub id: DiskId,
    pub name: String,
    pub mount_point: PathBuf,
    pub available_space: u64,
    pub total_space: u64,
    pub file_system: String,
    pub is_removable: bool,
    pub is_read_only: bool,
    pub kind: String,
    pub dev: String, // AKA: Disk Name or Device Name
    pub titles: Mutex<Vec<TitleInfo>>,
    pub progress: Mutex<Option<Progress>>,
    pub pid: Mutex<Option<u32>>,
    pub content: Option<DiskContent>,
}

impl OpticalDiskInfo {
    pub fn set_pid(&self, pid: Option<u32>) {
        *self.pid.lock().expect("failed to unlock pid") = pid;
    }

    pub fn set_progress(&self, progress: Option<Progress>) {
        *self.progress.lock().expect("failed to unlock progress") = progress;
    }

    pub fn has_process(&self) -> bool {
        if let Some(pid) = *self.pid.lock().unwrap() {
            let mut system = System::new_all();
            system.refresh_all();
            let sys_pid = Pid::from_u32(pid);
            system.process(sys_pid).is_some()
        } else {
            false
        }
    }

    pub fn kill_process(&self) {
        match *self.pid.lock().unwrap() {
            Some(pid) => {
                println!("Killing process {:?}", pid);
                let mut system = System::new_all();
                system.refresh_all();
                let sys_pid = Pid::from_u32(pid.clone());
                if let Some(process) = system.process(sys_pid) {
                    if process.kill() {
                        println!("Killed {:?}", pid);
                    } else {
                        println!("Failed to kill process with PID {}", pid);
                    }
                } else {
                    println!("Process with PID {} not found", pid);
                }
            }
            None => println!("No PID defined for Disk {}", self.id),
        }
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
        OpticalDiskInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            available_space: self.available_space,
            total_space: self.total_space,
            file_system: self.file_system.clone(),
            is_removable: self.is_removable,
            is_read_only: self.is_read_only,
            kind: self.kind.clone(),
            dev: self.dev.clone(),
            mount_point: self.mount_point.clone(),
            titles: Mutex::new(cloned_titles),
            progress: Mutex::new(cloned_progress),
            pid: Mutex::new(None),
            content: self.content.clone(),
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
            && self.dev == other.dev
            && self.mount_point == other.mount_point
    }
}

static NEXT_DISK_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Serialize, Clone, PartialEq, Copy)]
pub struct DiskId(u64);

impl DiskId {
    pub fn new() -> Self {
        DiskId(NEXT_DISK_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl fmt::Display for DiskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DiskId({})", self.0)
    }
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

// Optical Disk View Struct, This takes things like functions and converts them into pub method defined values
#[derive(Serialize)]
pub struct OpticalDiskInfoView {
    pub available_space: u64,
    pub content: Option<DiskContent>,
    pub dev: String, // AKA: Disk Name or Device Name
    pub file_system: String,
    pub has_process: bool,
    pub id: DiskId,
    pub is_read_only: bool,
    pub is_removable: bool,
    pub kind: String,
    pub mount_point: PathBuf,
    pub name: String,
    pub pid: Option<u32>,
    pub progress: Option<Progress>,
    pub titles: Vec<TitleInfo>,
    pub total_space: u64,
}

impl From<&OpticalDiskInfo> for OpticalDiskInfoView {
    fn from(optical_disk: &OpticalDiskInfo) -> Self {
        let has_process = optical_disk.has_process();
        let pid = optical_disk.pid.lock().unwrap().clone();
        let progress = optical_disk.progress.lock().unwrap().clone();
        let titles = optical_disk.titles.lock().unwrap().clone();
        OpticalDiskInfoView {
            available_space: optical_disk.available_space.clone(),
            content: optical_disk.content.clone(),
            dev: optical_disk.dev.clone(),
            file_system: optical_disk.file_system.clone(),
            has_process: has_process,
            id: optical_disk.id.clone(),
            is_read_only: optical_disk.is_read_only.clone(),
            is_removable: optical_disk.is_removable.clone(),
            kind: optical_disk.kind.clone(),
            mount_point: optical_disk.mount_point.clone(),
            name: optical_disk.name.clone(),
            pid: pid,
            progress: progress.clone(),
            titles: titles.clone(),
            total_space: optical_disk.total_space.clone(),
        }
    }
}

// --- Optical Progress ---
#[derive(Serialize, Clone)]
pub struct Progress {
    pub percentage: String,
    pub eta: String,
    pub label: String,
    pub message: String,
    pub failed: bool,
    pub title_id: Option<u32>,
}
