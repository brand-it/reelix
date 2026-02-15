use crate::models::optical_disk_info::OpticalDiskInfo;
use crate::services::auto_complete;
use crate::services::ftp_validator::spawn_ftp_validator;
use crate::services::version_checker::spawn_version_checker;
use crate::state::background_process_state::BackgroundProcessState;
use crate::state::uploaded_state::UploadedState;
use state::AppState;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{App, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_log::log::{debug, error, LevelFilter};
use tauri_plugin_log::{Target, TargetKind};
use tauri_plugin_opener::OpenerExt;
use tokio::sync::broadcast;

mod commands;
mod disk_listener;
mod models;
mod progress_tracker;
mod services;
mod standard_error;
mod state;
mod templates;
mod the_movie_db;

// only on macOS:
#[cfg(target_os = "macos")]
use tauri::TitleBarStyle;

const ICON_BYTES: &[u8] = include_bytes!("../icons/menu-icon.png");

fn spawn_disk_listener(app: &mut App) {
    let (sender, receiver) = broadcast::channel::<Vec<diff::Result<OpticalDiskInfo>>>(16);
    tauri::async_runtime::spawn(async move {
        disk_listener::watch_for_changes(sender).await;
    });

    let app_handle = app.handle().clone();
    tauri::async_runtime::spawn(async move {
        disk_listener::handle_changes(receiver, app_handle).await;
    });
}

fn setup_store(app: &mut App) {
    let app_handle = app.handle();
    let state = app_handle.state::<AppState>();
    if let Err(e) = state.load_from_store(app_handle) {
        error!("Failed to load state from store: {e}");
    }
}

fn setup_uploaded_state(app: &mut App) {
    let uploaded_state = match UploadedState::new(app.handle()) {
        Ok(state) => state,
        Err(e) => {
            eprintln!("Failed to initialize UploadedState: {e}");
            UploadedState::new(app.handle()).unwrap()
        }
    };
    app.manage(uploaded_state);
    let app_handle = app.handle().clone();
    tauri::async_runtime::spawn(async move {
        services::upload_recovery::resume_pending_uploads(app_handle).await;
    });
}
/// Custom filter that formats a datetime string into "YYYY"
// pub fn to_year(value: &Value, _args: &HashMap<String, Value>) -> TeraResult<Value> {
//     let date_str = value
//         .as_str()
//         .ok_or("format_date filter: expected a string")?;
//     // Try parsing the string as an RFC3339 datetime.
//     let formatted = if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
//         dt.format("%Y").to_string()
//     } else if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
//         // Fallback: if it's already just a date, use it.
//         date.format("%Y").to_string()
//     } else {
//         return Err(format!("format_date filter: failed to parse date: {}", date_str).into());
//     };
//     to_value(formatted).map_err(Into::into)
// }
fn setup_tray_icon(app: &mut App) {
    let version_label = format!("Version {}", app.package_info().version);
    let version_i = MenuItem::with_id(app, "version", version_label, true, None::<&str>)
        .expect("failed to create version item");
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)
        .expect("failed to create quit item");
    let show_i = MenuItem::with_id(app, "show", "Show", true, None::<&str>)
        .expect("failed to create show item");
    let menu = Menu::with_items(app, &[&show_i, &version_i, &quit_i])
        .expect("Failed to define menu with items");
    let tray_icon = tauri::image::Image::from_bytes(ICON_BYTES).expect("failure to load tray icon");
    TrayIconBuilder::new()
        .icon(tray_icon)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => {
                app.exit(0);
            }
            "show" => {
                let webview_window = app
                    .get_webview_window("main")
                    .expect("failed to find main window");
                match webview_window.show() {
                    Ok(_e) => {
                        let _ = webview_window.set_focus();
                    }
                    Err(_e) => {
                        debug!("Failed to show window");
                    }
                };
            }
            "version" => {
                let app_handle: &tauri::AppHandle = app.app_handle();
                app_handle
                    .opener()
                    .open_url("https://brand-it.github.io/reelix/", None::<&str>)
                    .map_err(|e| error!("Failed to open URL: {e}"))
                    .ok();
            }
            _ => {
                debug!("menu item {:?} not handled", event.id);
            }
        })
        .build(app)
        .expect("Failed to build tray icon");
}

//   {
//     "title": "Reelix",
//     "width": 1075,
//     "height": 800,
//     "minWidth": 500,
//     "minHeight": 500
//   }

fn setup_view_window(app: &mut App) {
    let win_builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
        .title("Reelix")
        .inner_size(1100.0, 900.0)
        .min_inner_size(500.0, 500.0);

    // set transparent title bar only when building for macOS
    #[cfg(target_os = "macos")]
    let win_builder = win_builder.title_bar_style(TitleBarStyle::Transparent);

    let _window = win_builder.build().unwrap();

    // set background color only when building for macOS
    #[cfg(target_os = "macos")]
    {
        use objc2::rc::Retained;
        use objc2::runtime::AnyObject;
        use objc2_app_kit::{NSColor, NSWindow};
        let raw = _window.ns_window().unwrap();
        // SAFETY: We know this pointer is really an NSWindow instance.
        let ns_window: Retained<NSWindow> = unsafe {
            let obj_ptr = raw as *mut AnyObject;
            Retained::from_raw(obj_ptr.cast()).unwrap()
        };

        let bg_color: Retained<NSColor> = NSColor::colorWithSRGBRed_green_blue_alpha(
            33.0 / 255.0,
            36.0 / 255.0,
            41.0 / 255.0,
            1.0,
        );
        ns_window.setBackgroundColor(Some(&bg_color));
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_http::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(LevelFilter::Trace)
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: None }),
                    Target::new(TargetKind::Webview),
                ])
                .build(),
        )
        .manage(AppState::new())
        .manage(BackgroundProcessState::new())
        .setup(|app| {
            setup_store(app);
            spawn_disk_listener(app);
            spawn_version_checker(app);
            spawn_ftp_validator(app.handle());
            setup_tray_icon(app);
            setup_view_window(app);
            setup_uploaded_state(app);
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                window.hide().unwrap();
                api.prevent_close();
            }
        })
        .invoke_handler(all_commands!())
        .build(tauri::generate_context!())
        .expect("error while building Tauri application");
    // Kick off background loading for autocomplete data; non-blocking
    auto_complete::init_background();
    // Run the application with a run event callback to shutdown sidecar process
    app.run(|app_handle, event| {
        if let tauri::RunEvent::Exit = event {
            let state = app_handle.state::<AppState>();
            let disks = state
                .optical_disks
                .read()
                .expect("Failed to get lock on optical_disks");

            // Iterate over the optical disks and kill the associated PID if it exists
            for disk in disks.iter() {
                let locked_disk = disk.read().expect("failed to get lock on disk");
                locked_disk.kill_process();
            }
        }
    });
}
