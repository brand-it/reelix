#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
pub use linux::opticals;

#[cfg(target_os = "macos")]
pub use macos::opticals;

#[cfg(target_os = "windows")]
pub use windows::opticals;
