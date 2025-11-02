use super::InlineTemplate;
use crate::models::optical_disk_info::OpticalDiskInfo;
use crate::state::AppState;
use crate::templates::movies::MoviesCards;
use crate::templates::seasons::SeasonsParts;
use askama::Template;
use tauri::State;

#[derive(Template)]
#[template(path = "disk_titles/options.turbo.html")]
pub struct DiskTitlesOptionsTurbo<'a> {
    pub seasons_parts: &'a SeasonsParts<'a>,
    pub movies_cards: &'a MoviesCards<'a>,
}

pub fn render_options(app_state: &State<'_, AppState>) -> Result<String, super::Error> {
    let selected_disk: Option<OpticalDiskInfo> = match app_state.selected_disk() {
        Some(disk) => {
            let read = disk.read().unwrap();
            Some(read.clone())
        }
        None => None,
    };

    let template = DiskTitlesOptionsTurbo {
        seasons_parts: &SeasonsParts {
            selected_disk: &selected_disk,
            episode: &None,
        },
        movies_cards: &MoviesCards {
            selected_disk: &selected_disk,
        },
    };
    super::render(template)
}
