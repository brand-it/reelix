use super::{render, ApiError};
use crate::models::optical_disk_info::{self, DiskId, OpticalDiskInfoView};
use crate::state::AppState;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tera::Context;

#[derive(Serialize)]
pub struct DiskOption {
    optical_disks: Vec<OpticalDiskInfoView>,
    selected_optical_disk_id: Option<DiskId>,
    selected_disk: Option<OpticalDiskInfoView>,
}

pub fn emit_disk_change(app_handle: &AppHandle) {
    let state = app_handle.state::<AppState>();
    let result = render_options(&state).expect("Failed to render disks/options");
    app_handle
        .emit("disks-changed", result)
        .expect("Failed to emit disks-changed");
}

pub fn render_options(app_state: &State<'_, AppState>) -> Result<String, ApiError> {
    let disk_option = build_disk_option(app_state);
    let context = Context::from_serialize(&disk_option).unwrap();
    render(&app_state.tera, "disks/options.html.turbo", &context, None)
}

pub fn render_toast_progress(
    app_state: &State<'_, AppState>,
    title: &Option<String>,
    progress: &Option<&optical_disk_info::Progress>,
) -> Result<String, ApiError> {
    let mut context = Context::new();
    context.insert("progress", &progress);
    context.insert("title", &title);

    render(
        &app_state.tera,
        "disks/toast_progress.html.turbo",
        &context,
        None,
    )
}

pub fn build_disk_option(app_state: &State<'_, AppState>) -> DiskOption {
    let optical_disks: Vec<OpticalDiskInfoView> = {
        let guard = app_state.optical_disks.read().unwrap();
        guard
            .iter()
            .map(|disk_arc| OpticalDiskInfoView::from(&*disk_arc.read().unwrap()))
            .collect()
    };

    let selected_optical_disk_id = app_state
        .selected_optical_disk_id
        .read()
        .unwrap()
        .to_owned();

    let selected_disk = match app_state.selected_disk() {
        Some(disk_arc) => {
            let guard = disk_arc.read().unwrap();
            Some(OpticalDiskInfoView::from(&*guard))
        }
        None => None,
    };

    DiskOption {
        optical_disks,
        selected_optical_disk_id,
        selected_disk,
    }
}
