#[cfg(target_os = "macos")]
use objc2_app_kit::NSWorkspace;
#[cfg(target_os = "macos")]
use objc2_foundation::{NSString, NSURL};
use std::path::Path;

#[cfg(target_os = "macos")]
pub fn eject(volume: &Path) {
    let ws = unsafe { NSWorkspace::sharedWorkspace() };

    let path = NSString::from_str(&volume.to_string_lossy());
    let url = unsafe { NSURL::fileURLWithPath(&path) };
    match unsafe { ws.unmountAndEjectDeviceAtURL_error(&url) } {
        Ok(()) => println!("Ejected {}", volume.display()),
        Err(err) => eprintln!("Error ejecting {path}: {err:?}"),
    }
}

#[cfg(target_os = "windows")]
pub fn eject(volume: &Path) {
    println!("Can't eject on windows yet {}", volume.display())
}
