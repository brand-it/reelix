use super::{render, ApiError};
use crate::{
    models::{
        optical_disk_info::{self, DiskId, OpticalDiskInfo},
        title_info::TitleInfo,
    },
    state::AppState,
};
use serde::Serialize;
use tauri::State;
use tera::Context;

#[derive(Serialize)]
pub struct DiskOption {
    optical_disks: Vec<OpticalDiskInfo>,
    selected_optical_disk_id: Option<DiskId>,
    selected_disk: Option<OpticalDiskInfo>,
    selected_disk_titles: Vec<TitleInfo>,
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
    let optical_disks: Vec<OpticalDiskInfo> = {
        let guard = app_state.optical_disks.read().unwrap();
        guard
            .iter()
            .map(|disk_arc| disk_arc.read().unwrap().clone())
            .collect()
    };

    let selected_optical_disk_id = app_state
        .selected_optical_disk_id
        .read()
        .unwrap()
        .to_owned();

    let (selected_disk, selected_disk_titles) = match app_state.selected_disk() {
        Some(disk_arc) => {
            let guard = disk_arc.read().unwrap();
            let disk_clone = guard.clone(); // clone the struct
            let titles = guard.titles.lock().unwrap().to_vec();
            (Some(disk_clone), titles)
        }
        None => (None, Vec::new()),
    };

    DiskOption {
        optical_disks,
        selected_optical_disk_id,
        selected_disk,
        selected_disk_titles,
    }
}
