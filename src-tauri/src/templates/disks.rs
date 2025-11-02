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

impl DisksOptions<'_> {
    pub fn dom_id(&self) -> &'static str {
        super::DISK_SELECTOR_DOM_ID
    }
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
pub struct DisksToastProgress<'a> {
    pub title: &'a Option<String>,
    pub progress: &'a Option<&'a optical_disk_info::Progress>,
}

impl DisksToastProgress<'_> {
    pub fn dom_id(&self) -> &'static str {
        super::DISK_TOAST_PROGRESS_DOM_ID
    }
}

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

pub fn render_toast_progress(
    _app_state: &State<'_, AppState>,
    title: &Option<String>,
    progress: &Option<&optical_disk_info::Progress>,
) -> Result<String, super::Error> {
    let template = DisksToastProgress { title, progress };
    super::render(template)
}
