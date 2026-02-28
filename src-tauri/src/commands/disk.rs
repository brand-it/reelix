// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use crate::models::optical_disk_info::DiskId;
use crate::services::disk_manager;
use crate::state::background_process_state::BackgroundProcessState;
use crate::state::AppState;
use crate::templates::{self, render_error};
use tauri::State;

#[tauri::command]
pub fn selected_disk(
    disk_id: u32,
    state: State<'_, AppState>,
    background_process_state: State<'_, BackgroundProcessState>,
) -> Result<String, templates::Error> {
    let id = DiskId::from(disk_id);

    let mut selected_optical_disk_id = state
        .selected_optical_disk_id
        .write()
        .expect("failed to lock selected disk ID");
    *selected_optical_disk_id = Some(id);

    // Also refresh the current season if one is being viewed
    let disk_selector_html =
        templates::disk_titles::render_options(&state, &background_process_state)?;

    Ok(disk_selector_html)
}

#[tauri::command]
pub fn eject_disk(
    state: State<'_, AppState>,
    background_process_state: State<'_, BackgroundProcessState>,
) -> Result<String, templates::Error> {
    match state.selected_disk() {
        Some(optical_disk) => {
            match optical_disk.read() {
                Ok(disk) => {
                    // On Linux, use the device path; on other platforms use mount point
                    #[cfg(target_os = "linux")]
                    disk_manager::eject_by_device(&disk.dev);

                    #[cfg(not(target_os = "linux"))]
                    disk_manager::eject(&disk.mount_point);
                }
                Err(_) => return render_error("Failed to get lock on memory for optical disk"),
            };
        }
        None => return render_error("No Disk is Selected can't eject"),
    };

    templates::disk_titles::render_options(&state, &background_process_state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_id_from_u32() {
        let _id1 = DiskId::from(1u32);
        let _id2 = DiskId::from(42u32);
        let _id3 = DiskId::from(999u32);

        // DiskId can be created from various u32 values
        // Test passes if no panic occurs
    }

    #[test]
    fn test_disk_id_equality() {
        let id1 = DiskId::from(1u32);
        let id2 = DiskId::from(1u32);
        let id3 = DiskId::from(2u32);

        // Same IDs should be equal
        assert!(id1 == id2);
        // Different IDs should not be equal
        assert!(id1 != id3);
    }

    #[test]
    fn test_disk_id_conversion_range() {
        // Test DiskId handles various u32 ranges
        let small_id = DiskId::from(1u32);
        let medium_id = DiskId::from(100u32);
        let large_id = DiskId::from(10000u32);

        assert!(small_id == DiskId::from(1));
        assert!(medium_id == DiskId::from(100));
        assert!(large_id == DiskId::from(10000));
    }
}
