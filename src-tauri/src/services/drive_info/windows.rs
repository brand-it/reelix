use crate::models::optical_disk_info;
use crate::models::optical_disk_info::OpticalDiskInfo;
use serde::Deserialize;
use std::sync::Mutex;
use wmi::WMIConnection;

// This struct maps to the WMI class Win32_CDROMDrive.
// https://crates.io/crates/wmi
#[derive(Deserialize)]
#[allow(non_snake_case)]
struct Win32_CDROMDrive {
    Drive: Option<String>,
    Name: String,
    VolumeName: String,
}

pub fn opticals() -> Vec<OpticalDiskInfo> {
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
                pid: Mutex::new(None),
                index: idx as u32,
            });
        }
    }
    opticals
}
