use log::debug;
#[cfg(target_os = "macos")]
use objc2_app_kit::NSWorkspace;
#[cfg(target_os = "macos")]
use objc2_foundation::{NSString, NSURL};
use std::path::Path;
#[cfg(target_os = "macos")]
use std::thread;

#[cfg(target_os = "macos")]
pub fn eject(volume: &Path) {
    let ws = NSWorkspace::sharedWorkspace();

    let path = NSString::from_str(&volume.to_string_lossy());
    let url = NSURL::fileURLWithPath(&path);
    const MAX_TRIES: usize = 5;
    for attempt in 1..=MAX_TRIES {
        match ws.unmountAndEjectDeviceAtURL_error(&url) {
            Ok(()) => {
                debug!("Ejected {}", volume.display());
                return;
            }
            Err(err) => {
                debug!(
                    "⚠️ Warning: eject of {} failed ({}).",
                    volume.display(),
                    err.localizedDescription(),
                );

                if attempt == MAX_TRIES {
                    debug!(
                        "❌ Failed to eject {} after {} attempts",
                        volume.display(),
                        MAX_TRIES
                    );
                    return;
                }
                thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    }
}

#[cfg(target_os = "windows")]
pub fn eject(volume: &Path) {
    debug!("Can't eject on windows yet {}", volume.display())
}

#[cfg(target_os = "linux")]
pub fn eject_by_device(device: &str) {
    use std::process::Command;

    if device.is_empty() {
        debug!("Device path is empty, cannot eject");
        return;
    }

    debug!("Attempting to eject device: {device}");

    // First try to unmount
    match Command::new("umount").arg(device).output() {
        Ok(output) => {
            if !output.status.success() {
                debug!(
                    "Warning: umount failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
        Err(e) => {
            debug!("Failed to run umount: {e}");
        }
    }

    // Then try to eject
    match Command::new("eject").arg(device).output() {
        Ok(output) => {
            if output.status.success() {
                debug!("Successfully ejected {device}");
            } else {
                debug!(
                    "Failed to eject {device}: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
        Err(e) => {
            debug!("Failed to run eject command: {e}");
        }
    }
}

#[cfg(target_os = "linux")]
pub fn eject(volume: &Path) {
    // Try to get the device path from the mount point (fallback method)
    let device = match get_device_from_mount(volume) {
        Some(dev) => dev,
        None => {
            debug!(
                "Could not find device for mount point: {}",
                volume.display()
            );
            return;
        }
    };

    eject_by_device(&device);
}

#[cfg(target_os = "linux")]
fn get_device_from_mount(mount_point: &Path) -> Option<String> {
    use std::fs;

    let mount_str = mount_point.to_string_lossy();

    if let Ok(mounts) = fs::read_to_string("/proc/mounts") {
        let mount_str_ref: &str = mount_str.as_ref();
        for line in mounts.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && parts[1] == mount_str_ref {
                return Some(parts[0].to_string());
            }
        }
    }

    None
}
