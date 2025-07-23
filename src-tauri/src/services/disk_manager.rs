#[cfg(target_os = "macos")]
use objc2_app_kit::NSWorkspace;
#[cfg(target_os = "macos")]
use objc2_foundation::{NSString, NSURL};
use std::path::Path;
use std::{thread, time::Duration};

#[cfg(target_os = "macos")]
pub fn eject(volume: &Path) {
    let ws = unsafe { NSWorkspace::sharedWorkspace() };

    let path = NSString::from_str(&volume.to_string_lossy());
    let url = unsafe { NSURL::fileURLWithPath(&path) };
    const MAX_TRIES: usize = 5;
    for attempt in 1..=MAX_TRIES {
        match unsafe { ws.unmountAndEjectDeviceAtURL_error(&url) } {
            Ok(()) => {
                println!("Ejected {}", volume.display());
                return;
            }
            Err(err) => {
                eprintln!(
                    "⚠️ Warning: eject of {} failed ({}).",
                    volume.display(),
                    err.localizedDescription(),
                );

                if attempt == MAX_TRIES {
                    eprintln!(
                        "❌ Failed to eject {} after {} attempts",
                        volume.display(),
                        MAX_TRIES
                    );
                    return;
                }
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
}

#[cfg(target_os = "windows")]
pub fn eject(volume: &Path) {
    println!("Can't eject on windows yet {}", volume.display())
}
