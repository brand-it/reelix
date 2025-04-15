pub mod general;
mod helpers;
pub mod rip;

#[macro_export]
macro_rules! all_commands {
    () => {
        tauri::generate_handler!(
            crate::commands::general::index,
            crate::commands::general::movie,
            crate::commands::general::open_url,
            crate::commands::rip::rip_one,
            crate::commands::general::search,
            crate::commands::general::season,
            crate::commands::general::the_movie_db,
            crate::commands::general::tv,
            crate::commands::rip::rip_season,
            crate::commands::general::selected_disk,
        )
    };
}
