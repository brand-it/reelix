pub mod disk;
pub mod general;
pub mod rip;
pub mod setting;

#[macro_export]
macro_rules! all_commands {
    () => {
        tauri::generate_handler!(
            $crate::commands::general::index,
            $crate::commands::general::movie,
            $crate::commands::general::open_url,
            $crate::commands::general::search,
            $crate::commands::general::suggestion,
            $crate::commands::general::season,
            $crate::commands::disk::selected_disk,
            $crate::commands::disk::eject_disk,
            $crate::commands::general::tv,
            $crate::commands::rip::assign_episode_to_title,
            $crate::commands::rip::rip_movie,
            $crate::commands::rip::rip_season,
            $crate::commands::rip::reorder_tv_episodes_on_ftp,
            $crate::commands::rip::set_auto_rip,
            $crate::commands::rip::withdraw_episode_from_title,
            $crate::commands::setting::update_ftp_settings,
            $crate::commands::setting::ftp_settings,
            $crate::commands::setting::the_movie_db,
        )
    };
}
