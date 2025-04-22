use crate::models::optical_disk_info::{DiskId, OpticalDiskInfo};
use crate::services::drive_info::opticals;
use crate::services::makemkvcon;
use crate::state::AppState;
use crate::templates;
use std::sync::{Arc, RwLock};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};

// pub fn list() {
//     let disks: Disks = Disks::new_with_refreshed_list();

//     for disk in &disks {
//         let fs_bytes = disk.file_system();
//         let fs_str = fs_bytes.to_str().expect("Failed to load fs_bytes");
//         println!("#-------------------DISK---------------------#");
//         // Check if removable + known optical file system
//         if disk.is_removable() && (fs_str.contains("udf") || fs_str.contains("iso9660")) {
//             println!("Likely optical media:");
//             println!("  Name: {:?}", disk.name());
//             println!("  Mount Point: {:?}", disk.mount_point());
//             println!("  Available Space: {}", disk.available_space());
//             println!("  Total Space: {}", disk.total_space());
//             println!("  Kind: {}", disk.kind());
//             println!("  File System: {:?}", disk.file_system());
//             println!("  Is Removable: {}", disk.is_removable());
//             println!("  Is Read Only: {}", disk.is_read_only());
//             println!("  Usage: {:?}", disk.usage());
//         } else {
//             println!("Non-optical or unrecognized: {:?}", disk.name());
//         }
//         println!("#-------------------END DISK-----------------#");
//     }
// }

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
    println!("Stared watching for changes to optical Disks....");
    loop {
        let current_opticals = opticals();

        if current_opticals != previous_opticals {
            let diff_result = changes(&current_opticals, &previous_opticals);

            match sender.send(diff_result) {
                Ok(num_receivers) => println!("Broadcast sent to {} receivers", num_receivers),
                Err(_err) => eprintln!("Broadcast send failed"),
            }
            previous_opticals = current_opticals;
        }
        // Failure to sleep ever second means we use 100% of our CPU DUH
        // Hey future "human" improve this scanner system...or don't if it works why change it
        sleep(Duration::from_secs(1)).await;
    }
}

fn emit_disk_change(app_handle: &AppHandle) {
    let state = app_handle.state::<AppState>();
    let result = templates::disks::render_options(&state).expect("Failed to render disks/options");
    app_handle
        .emit("disks-changed", result)
        .expect("Failed to emit disks-changed");
}

fn emit_disk_titles_change(app_handle: &AppHandle) {
    let app_state = app_handle.state::<AppState>();
    let result = templates::disk_titles::render_options(&app_state)
        .expect("Failed to render disk_titles/options");
    app_handle
        .emit("disks-changed", result)
        .expect("Failed to emit emit_disk_titles_change");
}

fn unwrap_disk(disk: &Arc<RwLock<OpticalDiskInfo>>) -> OpticalDiskInfo {
    disk.read().expect("Failed to lock").clone()
}

fn contains(
    optical_disks: &Vec<Arc<RwLock<OpticalDiskInfo>>>,
    disk: &Arc<RwLock<OpticalDiskInfo>>,
) -> bool {
    optical_disks
        .iter()
        .any(|optical_disk| unwrap_disk(optical_disk) == unwrap_disk(disk))
}

async fn load_titles(app_handle: &AppHandle, disk_id: DiskId) {
    let state: tauri::State<'_, AppState> = app_handle.state::<AppState>();
    let results = makemkvcon::title_info(disk_id, app_handle).await;

    match state.find_optical_disk_by_id(&disk_id) {
        Some(disk) => {
            let locked_disk = disk.write().expect("Failed to grab disk");
            locked_disk
                .titles
                .lock()
                .expect("failed to get titles")
                .extend(results.title_infos);
        }
        None => println!("Disk not found in state."),
    }
}

fn add_optical_disk(app_handle: &AppHandle, disk: &OpticalDiskInfo) {
    let state: tauri::State<'_, AppState> = app_handle.state::<AppState>();
    let optical_disk = Arc::new(RwLock::new(disk.clone()));
    let mut optical_disks = state
        .optical_disks
        .write()
        .expect("Failed to grab optical disks");
    if !contains(&optical_disks, &optical_disk) {
        optical_disks.push(optical_disk);
    }
}

fn remove_optical_disks(app_handle: &AppHandle, disk: &OpticalDiskInfo) {
    let state: tauri::State<'_, AppState> = app_handle.state::<AppState>();
    let mut optical_disks = state
        .optical_disks
        .write()
        .expect("Failed to grab optical disks");
    optical_disks.retain(|x| *x.read().expect("Failed to grab optical disk info") != *disk);
}

pub fn set_default_selected_disk(app_handle: &AppHandle, disk_id: DiskId) {
    let state = app_handle.state::<AppState>();
    let mut selected_optical_disk_id = state
        .selected_optical_disk_id
        .write()
        .expect("failed to lock selected disk ID");
    if selected_optical_disk_id.is_none() {
        println!("changed default selected optical disk to {}", disk_id);
        *selected_optical_disk_id = Some(disk_id.clone());
    }
}

pub fn clear_selected_disk(app_handle: &AppHandle, disk_id: DiskId) {
    let state = app_handle.state::<AppState>();
    let mut selected_optical_disk_id = state
        .selected_optical_disk_id
        .write()
        .expect("failed to lock selected disk ID");

    if selected_optical_disk_id.as_ref() == Some(&disk_id) {
        *selected_optical_disk_id = None;
    }
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
                            println!("- {:?}", disk.name);
                            clear_selected_disk(&app_handle, disk.id);
                            remove_optical_disks(&app_handle, &disk);
                            emit_disk_change(&app_handle);
                            emit_disk_titles_change(&app_handle);
                        }
                        diff::Result::Both(disk, _) => {
                            println!("? {:?}", disk.name);
                        }
                        diff::Result::Right(disk) => {
                            println!("+ {:?}", disk.name);
                            add_optical_disk(&app_handle, &disk);
                            set_default_selected_disk(&app_handle, disk.id);
                            emit_disk_change(&app_handle);
                            let app_handle_clone = app_handle.clone();
                            tokio::spawn(async move {
                                load_titles(&app_handle_clone, disk.id).await;
                                emit_disk_titles_change(&app_handle_clone);
                                emit_disk_change(&app_handle_clone);
                            });
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
