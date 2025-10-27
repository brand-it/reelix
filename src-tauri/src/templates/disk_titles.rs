use crate::models::optical_disk_info::OpticalDiskInfo;
use crate::state::AppState;
use tauri::State;

// #[derive(Template)]
// #[template(path = "disk_titles/options.turbo.html")]
// pub struct DiskTitlesOptions {
//     pub selected_disk: Option<OpticalDiskInfo>,
//     pub episode: Option<TvEpisode>,
// }

pub fn render_options(app_state: &State<'_, AppState>) -> Result<String, super::Error> {
    let _selected_disk: Option<OpticalDiskInfo> = match app_state.selected_disk() {
        Some(disk) => {
            let read = disk.read().unwrap();
            Some(read.clone())
        }
        None => None,
    };

    // let template = DiskTitlesOptions {
    //     selected_disk,
    //     episode: None,
    // };
    // render(template)
    Ok("".to_string())
}
