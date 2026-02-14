use crate::models::optical_disk_info;
use crate::models::optical_disk_info::OpticalDiskInfo;
use std::sync::Mutex;
use sysinfo::{Disk, Disks};

pub fn opticals() -> Vec<OpticalDiskInfo> {
    let disks = Disks::new_with_refreshed_list();
    let mut opticals = Vec::new();
    disks
        .iter()
        .filter(|disk| is_optical_disk(disk))
        .enumerate()
        .for_each(|(idx, disk)| {
            let mount_point =
                std::path::PathBuf::from(format!("{}", disk.mount_point().to_string_lossy()));

            // Extract disc name from mount point (e.g., /Volumes/THE_NAKED_GUN -> THE_NAKED_GUN)
            // Fall back to device name if mount point doesn't have a filename component
            let name = mount_point
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| disk.name().to_string_lossy().to_string());

            opticals.push(OpticalDiskInfo {
                id: optical_disk_info::DiskId::new(),
                name,
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

fn is_optical_disk(disk: &Disk) -> bool {
    let fs_bytes = disk.file_system();
    let fs_str = fs_bytes.to_str().unwrap_or("");

    disk.is_removable() && (fs_str.contains("udf") || fs_str.contains("iso9660"))
}
