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
pub fn eject(volume: &Path) {
    debug!("Can't eject on linux yet {}", volume.display())
}
