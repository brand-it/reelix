#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
use crate::models::optical_disk_info;
use crate::models::optical_disk_info::OpticalDiskInfo;
use std::sync::Mutex;

#[cfg(target_os = "windows")]
use {serde::Deserialize, wmi::WMIConnection};

#[cfg(not(target_os = "windows"))]
use sysinfo::{Disk, Disks};

// This struct maps to the WMI class Win32_CDROMDrive.
// https://crates.io/crates/wmi
#[derive(Deserialize)]
#[cfg(target_os = "windows")]
struct Win32_CDROMDrive {
    Drive: Option<String>,
    Name: String,
    VolumeName: String,
}

#[cfg(not(target_os = "windows"))]
pub fn opticals() -> Vec<OpticalDiskInfo> {
    // use std::path::PathBuf; (removed unused import)

    let disks = Disks::new_with_refreshed_list();
    let mut opticals = Vec::new();
    disks
        .iter()
        .filter(|disk| is_optical_disk(disk))
        .enumerate()
        .for_each(|(idx, disk)| {
            let mount_point =
                std::path::PathBuf::from(format!("{}", disk.mount_point().to_string_lossy()));
            opticals.push(OpticalDiskInfo {
                id: optical_disk_info::DiskId::new(),
                name: disk.name().to_string_lossy().to_string(),
                available_space: disk.available_space(),
                total_space: disk.total_space(),
                file_system: disk.file_system().to_string_lossy().to_string(),
                is_removable: disk.is_removable(),
                is_read_only: disk.is_removable(),
                kind: format!("{:?}", disk.kind()),
                dev: String::new(),
                mount_point,
                titles: Mutex::new(Vec::new()),
                pid: Mutex::new(None),
                index: idx as u32,
            })
        });
    opticals
}

#[cfg(not(target_os = "windows"))]
fn is_optical_disk(disk: &Disk) -> bool {
    let fs_bytes = disk.file_system();
    let fs_str = fs_bytes.to_str().unwrap_or("");

    disk.is_removable() && (fs_str.contains("udf") || fs_str.contains("iso9660"))
}

#[cfg(target_os = "windows")]
pub fn opticals() -> Vec<OpticalDiskInfo> {
    // use std::path::PathBuf; (removed unused import)

    let wmi_con = WMIConnection::new().expect("Failed to create WMI connection");

    let results: Vec<Win32_CDROMDrive> = wmi_con
        .query()
        .expect("WMI query for optical drives failed");

    let mut opticals = Vec::new();

    // Convert each drive returned by WMI into your OpticalDiskInfo.
    for (idx, drive) in results.into_iter().enumerate() {
        if let Some(dev) = drive.Drive {
            // Use the Caption if available, otherwise use the drive letter.
            let name = drive.VolumeName;
            opticals.push(OpticalDiskInfo {
                id: optical_disk_info::DiskId::new(),
                name,
                available_space: 0,
                total_space: 0,
                file_system: String::new(),
                is_removable: true,
                is_read_only: true,
                kind: "Optical Disk".to_string(),
                dev,
                mount_point: std::path::PathBuf::new(),
                titles: Mutex::new(Vec::new()),
                // progress: Mutex::new(None), // removed, not a field of OpticalDiskInfo
                pid: Mutex::new(None),
                // content: None, // removed, not a field of OpticalDiskInfo
                index: idx as u32,
            });
        }
    }
    opticals
}
