use crate::models::title_info;
use crate::services::makemkvcon;
use crate::state::AppState;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use sysinfo::{Disk, Disks};
use tauri::{AppHandle, Emitter, Manager};
use tera::{Context, Tera};
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};

type ErrorHandler = fn(&tera::Error) -> ApiError;
#[derive(Debug, Clone)]
pub struct ApiError {
    pub code: u16,
    pub message: String,
    pub api_key: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OpticalDiskInfo {
    pub name: String,
    pub mount_point: PathBuf,
    pub available_space: u64,
    pub total_space: u64,
    pub file_system: String,
    pub is_removable: bool,
    pub is_read_only: bool,
    pub kind: String,
    pub titles: Mutex<Vec<title_info::TitleInfo>>,
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
        OpticalDiskInfo {
            name: self.name.clone(),
            mount_point: self.mount_point.clone(),
            available_space: self.available_space,
            total_space: self.total_space,
            file_system: self.file_system.clone(),
            is_removable: self.is_removable,
            is_read_only: self.is_read_only,
            kind: self.kind.clone(),
            titles: Mutex::new(cloned_titles),
        }
    }
}

// Manually implement PartialEq
// I don't want to compair the titles because they can change state later on
// in the process
impl PartialEq for OpticalDiskInfo {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.mount_point == other.mount_point
            && self.available_space == other.available_space
            && self.total_space == other.total_space
            && self.file_system == other.file_system
            && self.is_removable == other.is_removable
            && self.is_read_only == other.is_read_only
            && self.kind == other.kind
    }
}

pub fn list() {
    let disks: Disks = Disks::new_with_refreshed_list();

    for disk in &disks {
        let fs_bytes = disk.file_system();
        let fs_str = fs_bytes.to_str().unwrap();
        println!("#-------------------DISK---------------------#");
        // Check if removable + known optical file system
        if disk.is_removable() && (fs_str.contains("udf") || fs_str.contains("iso9660")) {
            println!("Likely optical media:");
            println!("  Name: {:?}", disk.name());
            println!("  Mount Point: {:?}", disk.mount_point());
            println!("  Available Space: {}", disk.available_space());
            println!("  Total Space: {}", disk.total_space());
            println!("  Kind: {}", disk.kind());
            println!("  File System: {:?}", disk.file_system());
            println!("  Is Removable: {}", disk.is_removable());
            println!("  Is Read Only: {}", disk.is_read_only());
            println!("  Usage: {:?}", disk.usage());
        } else {
            println!("Non-optical or unrecognized: {:?}", disk.name());
        }
        println!("#-------------------END DISK-----------------#");
    }
}

pub fn opticals() -> Vec<OpticalDiskInfo> {
    let disks = Disks::new_with_refreshed_list();
    let mut opticals = Vec::new();
    disks
        .iter()
        .filter(|disk| is_optical_disk(disk))
        .for_each(|disk| {
            opticals.push(OpticalDiskInfo {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_path_buf(),
                available_space: disk.available_space(),
                total_space: disk.total_space(),
                file_system: disk.file_system().to_string_lossy().to_string(),
                is_removable: disk.is_removable(),
                is_read_only: disk.is_removable(),
                kind: format!("{:?}", disk.kind()),
                titles: Mutex::new(Vec::new()),
            })
        });
    opticals
}

fn is_optical_disk(disk: &Disk) -> bool {
    let fs_bytes = disk.file_system();
    let fs_str = fs_bytes.to_str().unwrap_or("");

    disk.is_removable() && (fs_str.contains("udf") || fs_str.contains("iso9660"))
}

fn changes(
    current_opticals: &Vec<OpticalDiskInfo>,
    previous_opticals: &Vec<OpticalDiskInfo>,
) -> Vec<diff::Result<OpticalDiskInfo>> {
    let mut optics = Vec::new();
    diff::slice(previous_opticals, current_opticals)
        .into_iter()
        .for_each(|result| match result {
            diff::Result::Left(info) => optics.push(diff::Result::Left(info.clone())),
            diff::Result::Both(info, _) => {
                optics.push(diff::Result::Both(info.clone(), info.clone()))
            }
            diff::Result::Right(info) => optics.push(diff::Result::Right(info.clone())),
        });
    optics
}

pub async fn watch_for_changes(sender: broadcast::Sender<Vec<diff::Result<OpticalDiskInfo>>>) {
    let mut previous_opticals = Vec::new();
    println!("Watching for changes on Disk");
    loop {
        let current_opticals = opticals();

        if current_opticals != previous_opticals {
            let diff_result = changes(&current_opticals, &previous_opticals);

            println!(
                "Change detected: old={:?}, new={:?}",
                previous_opticals, current_opticals
            );
            match sender.send(diff_result) {
                Ok(num_receivers) => println!("Broadcast sent to {} receivers", num_receivers),
                Err(err) => eprintln!("Broadcast send failed: {:?}", err),
            }
            previous_opticals = current_opticals;
        }
        sleep(Duration::from_secs(1)).await;
    }
}

fn render_template(
    tera: &Tera,
    template_path: &str,
    context: &Context,
    on_error: Option<ErrorHandler>,
) -> Result<String, ApiError> {
    match tera.render(template_path, context) {
        Ok(result) => Ok(result),
        Err(e) => {
            eprintln!("Template rendering error: {:#?}", e);
            // Custom Error handler if provided
            if let Some(handler) = on_error {
                return Err(handler(&e));
            } else {
                return Err(ApiError {
                    code: 500,
                    message: format!("An error occurred: {e}"),
                    api_key: None,
                });
            }
        }
    }
}

fn emit_disk_change(app_handle: &AppHandle) {
    let state = app_handle.state::<AppState>();
    let mut context = Context::new();
    let optical_disks = &state.optical_disks.lock().unwrap().to_vec();
    context.insert("optical_disks", &unwrap_disks(optical_disks));
    let result = render_template(&state.tera, "disks/options.html.turbo", &context, None)
        .expect("Failed to render disks/options.html.turbo");
    app_handle
        .emit("disks-changed", result)
        .expect("Failed to emit disks-changed");
}

fn emit_disk_titles_change(app_handle: &AppHandle) {
    let state = app_handle.state::<AppState>();
    let mut context = Context::new();
    let optical_disks = &state.optical_disks.lock().unwrap().to_vec();
    context.insert("optical_disks", &unwrap_disks(optical_disks));
    let result = render_template(
        &state.tera,
        "disk_titles/options.html.turbo",
        &context,
        None,
    )
    .expect("Failed to render disk_titles/options.html.turbo");
    app_handle
        .emit("disks-changed", result)
        .expect("Failed to emit emit_disk_titles_change");
}

fn unwrap_disk(disk: &Arc<Mutex<OpticalDiskInfo>>) -> OpticalDiskInfo {
    disk.lock().expect("Failed to lock").clone()
}

fn unwrap_disks(disks: &Vec<Arc<Mutex<OpticalDiskInfo>>>) -> Vec<OpticalDiskInfo> {
    disks.iter().map(|disk| unwrap_disk(disk)).collect()
}

fn contains(
    optical_disks: &Vec<Arc<Mutex<OpticalDiskInfo>>>,
    disk: &Arc<Mutex<OpticalDiskInfo>>,
) -> bool {
    optical_disks
        .iter()
        .any(|optical_disk| unwrap_disk(optical_disk) == unwrap_disk(disk))
}

async fn load_titles(app_handle: &AppHandle, disk: &OpticalDiskInfo) {
    let path = &disk.mount_point.to_string_lossy().to_string();
    let state: tauri::State<'_, AppState> = app_handle.state::<AppState>();

    match makemkvcon::title_info(app_handle, path).await {
        Ok(results) => {
            let optical_disks = state.optical_disks.lock().unwrap();
            if let Some(disk_arc) = optical_disks.iter().find(|disk_arc| {
                let d = disk_arc.lock().unwrap();
                d.mount_point == disk.mount_point
            }) {
                disk_arc
                    .lock()
                    .unwrap()
                    .titles
                    .lock()
                    .unwrap()
                    .extend(results.title_infos);
                println!("Updated Titles: {:?}", disk_arc.lock().unwrap().titles);
            } else {
                println!("Disk not found in state.");
            }
        }
        Err(e) => {
            println!("Loading title info error {}", e)
        }
    }
}

fn add_optical_disk(app_handle: &AppHandle, disk: &OpticalDiskInfo) {
    let state: tauri::State<'_, AppState> = app_handle.state::<AppState>();
    let optical_disk = Arc::new(Mutex::new(disk.clone()));
    let mut optical_disks = state.optical_disks.lock().unwrap();
    if !contains(&optical_disks, &optical_disk) {
        optical_disks.push(optical_disk);
    }
    println!("optical_disks: {:?}", optical_disks);
}

fn remove_optical_disks(app_handle: &AppHandle, disk: &OpticalDiskInfo) {
    let state: tauri::State<'_, AppState> = app_handle.state::<AppState>();
    let mut optical_disks = state.optical_disks.lock().unwrap();
    optical_disks.retain(|x| *x.lock().unwrap() != *disk);
    println!("optical_disks: {:?}", optical_disks);
}

/// A separate async task that listens for changes and reacts to them.
pub async fn handle_changes(
    mut receiver: broadcast::Receiver<Vec<diff::Result<OpticalDiskInfo>>>,
    app_handle: AppHandle,
) {
    loop {
        println!("Waiting for changes on Disk");
        match receiver.recv().await {
            Ok(event) => {
                println!("Message received");
                for result in event {
                    match result {
                        diff::Result::Left(disk) => {
                            println!("- {:?}", disk);
                            remove_optical_disks(&app_handle, &disk);
                            emit_disk_change(&app_handle);
                        }
                        diff::Result::Both(disk, _) => {
                            println!("  {:?}", disk);
                        }
                        diff::Result::Right(disk) => {
                            println!("+ {:?}", disk);
                            println!("Name: {:?}", disk.name);
                            println!("Mount Point: {:?}", disk.mount_point);
                            println!("Available Space: {}", disk.available_space);
                            println!("Total Space: {}", disk.total_space);
                            println!("Kind: {}", disk.kind);
                            println!("File System: {:?}", disk.file_system);
                            println!("Is Removable: {}", disk.is_removable);
                            println!("Is Read Only: {}", disk.is_read_only);
                            add_optical_disk(&app_handle, &disk);
                            emit_disk_change(&app_handle);
                            load_titles(&app_handle, &disk).await;
                            emit_disk_titles_change(&app_handle);
                        }
                    }
                }
            }
            Err(broadcast::error::RecvError::Lagged(count)) => {
                println!("Dropped {} messages due to lag.", count);
            }
            Err(broadcast::error::RecvError::Closed) => {
                println!("Channel has closed.");
            }
        }
    }
}
