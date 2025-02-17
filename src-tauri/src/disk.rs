use crate::services::makemkvcon;
use std::path::PathBuf;
use sysinfo::{Disk, DiskKind, Disks};
use tauri::AppHandle;
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};
#[derive(Debug)]
pub struct OpticalDiskInfo {
    pub name: String,
    pub mount_point: PathBuf,
    pub available_space: u64,
    pub total_space: u64,
    pub file_system: String,
    pub is_removable: bool,
    pub is_read_only: bool,
    pub kind: DiskKind,
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

    disks
        .iter()
        .filter(|disk| is_optical_disk(disk))
        .map(|disk| OpticalDiskInfo {
            name: disk.name().to_string_lossy().to_string(),
            mount_point: disk.mount_point().to_path_buf(),
            available_space: disk.available_space(),
            total_space: disk.total_space(),
            file_system: disk.file_system().to_string_lossy().to_string(),
            is_removable: disk.is_removable(),
            is_read_only: disk.is_removable(),
            kind: disk.kind(),
        })
        .collect()
}

fn is_optical_disk(disk: &Disk) -> bool {
    let fs_bytes = disk.file_system();
    let fs_str = fs_bytes.to_str().unwrap_or("");

    disk.is_removable() && (fs_str.contains("udf") || fs_str.contains("iso9660"))
}

pub async fn watch_for_changes(sender: broadcast::Sender<()>) {
    let mut previous_opticals = Vec::new();
    println!("Watching for changes on Disk");
    loop {
        let current_opticals = opticals()
            .iter()
            .map(|disk| disk.name.to_string())
            .collect();

        if current_opticals != previous_opticals {
            println!(
                "Change detected: old={:?}, new={:?}",
                previous_opticals, current_opticals
            );
            match sender.send(()) {
                Ok(num_receivers) => println!("Broadcast sent to {} receivers", num_receivers),
                Err(err) => eprintln!("Broadcast send failed: {:?}", err),
            }
            previous_opticals = current_opticals;
        }
        sleep(Duration::from_secs(1)).await;
    }
}

/// A separate async task that listens for changes and reacts to them.
pub async fn handle_changes(mut receiver: broadcast::Receiver<()>, app_handle: AppHandle) {
    loop {
        println!("Listing for changes on Disk");
        match receiver.recv().await {
            Ok(()) => {
                println!("Message received");
                for disk in opticals() {
                    println!("Name: {:?}", disk.name);
                    println!("Mount Point: {:?}", disk.mount_point);
                    println!("Available Space: {}", disk.available_space);
                    println!("Total Space: {}", disk.total_space);
                    println!("Kind: {}", disk.kind);
                    println!("File System: {:?}", disk.file_system);
                    println!("Is Removable: {}", disk.is_removable);
                    println!("Is Read Only: {}", disk.is_read_only);
                    makemkvcon::info(&app_handle, &disk.mount_point.to_string_lossy().to_string());
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
