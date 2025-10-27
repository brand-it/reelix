use crate::models::optical_disk_info;
use crate::state::AppState;
use crate::templates::movies::MoviesCards;
use crate::templates::seasons::SeasonsParts;
use crate::templates::InlineTemplate;
use askama::Template;
use tauri::{AppHandle, Emitter, Manager, State};

#[derive(Template)]
#[template(path = "disks/options.html")]
pub struct DisksOptions<'a> {
    pub optical_disks: &'a Vec<optical_disk_info::OpticalDiskInfo>,
    pub selected_disk: &'a Option<optical_disk_info::OpticalDiskInfo>,
}
#[derive(Template)]
#[template(path = "disks/options.turbo.html")]
pub struct DisksOptionsTurbo<'a> {
    pub disks_options: &'a DisksOptions<'a>,
    pub seasons_parts: &'a SeasonsParts<'a>,
    pub movies_cards: &'a MoviesCards<'a>,
}

#[derive(Template)]
#[template(path = "disks/toast_progress.html")]
pub struct DiskToastProgress<'a> {
    pub title: &'a Option<String>,
    pub progress: &'a Option<&'a optical_disk_info::Progress>,
}

// impl DiskOptions {
//     pub fn any_ripped(&self) -> bool {
//         self.selected_disk
//             .as_ref()
//             .map(|disk| disk.titles.iter().any(|t| t.rip))
//             .unwrap_or(false)
//     }

//     pub fn has_disks(&self) -> bool {
//         !self.optical_disks.is_empty()
//     }

//     pub fn selected_disk_id(&self) -> Option<DiskId> {
//         self.selected_disk.as_ref().map(|d| d.id)
//     }

//     pub fn selected_disk_name(&self) -> &str {
//         self.selected_disk
//             .as_ref()
//             .map(|d| d.name.as_str())
//             .unwrap_or("No Optical Drive Detected")
//     }
// }

pub fn emit_disk_change(app_handle: &AppHandle) {
    let state = app_handle.state::<AppState>();
    let result = render_options(&state).expect("Failed to render disks/options");
    app_handle
        .emit("disks-changed", result)
        .expect("Failed to emit disks-changed");
}

pub fn render_options(app_state: &State<'_, AppState>) -> Result<String, super::Error> {
    let optical_disks = app_state.clone_optical_disks();

    let selected_disk: Option<optical_disk_info::OpticalDiskInfo> = match app_state.selected_disk()
    {
        Some(disk_arc) => {
            let guard = disk_arc.read().unwrap();
            Some(guard.to_owned())
        }
        None => None,
    };

    let disks_options = DisksOptions {
        optical_disks: &optical_disks,
        selected_disk: &selected_disk,
    };
    let seasons_parts = SeasonsParts {
        selected_disk: &selected_disk,
        episode: &None,
    };
    let movies_cards = MoviesCards {
        selected_disk: &selected_disk,
    };
    let disks_options_turbo = DisksOptionsTurbo {
        disks_options: &disks_options,
        seasons_parts: &seasons_parts,
        movies_cards: &movies_cards,
    };

    super::render(disks_options_turbo)
}

// pub fn build_disk_option(app_state: &State<'_, AppState>) -> DiskOption {
//     let optical_disks: Vec<OpticalDiskInfoView> = {
//         let guard = app_state.optical_disks.read().unwrap();
//         guard
//             .iter()
//             .map(|disk_arc| OpticalDiskInfoView::from(&*disk_arc.read().unwrap()))
//             .collect()
//     };

//     let selected_optical_disk_id = app_state
//         .selected_optical_disk_id
//         .read()
//         .unwrap()
//         .to_owned();

//     let selected_disk = match app_state.selected_disk() {
//         Some(disk_arc) => {
//             let guard = disk_arc.read().unwrap();
//             Some(OpticalDiskInfoView::from(&*guard))
//         }
//         None => None,
//     };

//     DiskOption {
//         optical_disks,
//         selected_optical_disk_id,
//         selected_disk,
//     }
// }

pub fn render_toast_progress(
    _app_state: &State<'_, AppState>,
    title: &Option<String>,
    progress: &Option<&optical_disk_info::Progress>,
) -> Result<String, super::Error> {
    let template = DiskToastProgress { title, progress };
    super::render(template)
}
