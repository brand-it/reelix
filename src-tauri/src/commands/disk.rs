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

    templates::disk_titles::render_options(&state, &background_process_state)
}

#[tauri::command]
pub fn eject_disk(
    state: State<'_, AppState>,
    background_process_state: State<'_, BackgroundProcessState>,
) -> Result<String, templates::Error> {
    match state.selected_disk() {
        Some(optical_disk) => {
            match optical_disk.read() {
                Ok(disk) => disk_manager::eject(&disk.mount_point),
                Err(_) => return render_error("Failed to get lock on memory for optical disk"),
            };
        }
        None => return render_error("No Disk is Selected can't eject"),
    };

    templates::disk_titles::render_options(&state, &background_process_state)
}
