use crate::models::optical_disk_info::OpticalDiskInfo;
use state::AppState;
use std::sync::{Arc, Mutex, RwLock};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{App, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_log::log::debug;
use tauri_plugin_log::{Target, TargetKind};
use tauri_plugin_store::StoreExt;
use tokio::sync::broadcast;

mod commands;
mod disk;
mod models;
mod progress_tracker;
mod services;
mod state;
mod templates;

// only on macOS:
#[cfg(target_os = "macos")]
use tauri::TitleBarStyle;

const ICON_BYTES: &[u8] = include_bytes!("../icons/menu-icon.png");

fn spawn_disk_listener(app: &mut App) {
    let (sender, receiver) = broadcast::channel::<Vec<diff::Result<OpticalDiskInfo>>>(16);
    tauri::async_runtime::spawn(async move {
        disk::watch_for_changes(sender).await;
    });

    let app_handle = app.handle().clone();
    tauri::async_runtime::spawn(async move {
        disk::handle_changes(receiver, app_handle).await;
    });
}

fn setup_store(app: &mut App) {
    let app_handle = app.handle();
    let state = app_handle.state::<AppState>();
    let store = app.store("store.json").unwrap();
    store.keys().iter().for_each(|key| {
        if let Some(value) = store.get(key) {
            if let Some(value_str) = value.as_str() {
                match state.update(key, Some(value_str.to_string())) {
                    Ok(_n) => debug!("set {key} to {value_str}"),
                    Err(e) => debug!("setup store failure: {e}"),
                };
            }
        }
    });
    store.close_resource();
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
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)
        .expect("failed to create quit item");
    let show_i = MenuItem::with_id(app, "show", "Show", true, None::<&str>)
        .expect("failed to create quit item");
    let menu =
        Menu::with_items(app, &[&show_i, &quit_i]).expect("Failed to define menu with items");
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
        .inner_size(1075.0, 800.0)
        .min_inner_size(500.0, 500.0);

    // set transparent title bar only when building for macOS
    #[cfg(target_os = "macos")]
    let win_builder = win_builder.title_bar_style(TitleBarStyle::Transparent);

    let window = win_builder.build().unwrap();

    // set background color only when building for macOS
    #[cfg(target_os = "macos")]
    {
        use objc2::rc::Retained;
        use objc2::runtime::AnyObject;
        use objc2_app_kit::{NSColor, NSWindow};
        let raw = window.ns_window().unwrap();
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
    let app_state: AppState = AppState {
        ftp_host: Arc::new(Mutex::new(None)),
        ftp_movie_upload_path: Arc::new(Mutex::new(None)),
        ftp_pass: Arc::new(Mutex::new(None)),
        ftp_user: Arc::new(Mutex::new(None)),
        optical_disks: Arc::new(RwLock::new(Vec::<Arc<RwLock<OpticalDiskInfo>>>::new())),
        query: Arc::new(Mutex::new(String::new())),
        selected_optical_disk_id: Arc::new(RwLock::new(None)),
        the_movie_db_key: Arc::new(Mutex::new(String::new())),
    };

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_http::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: None }),
                    Target::new(TargetKind::Webview),
                ])
                .build(),
        )
        .manage(app_state)
        .setup(|app| {
            setup_store(app);
            spawn_disk_listener(app);
            setup_tray_icon(app);
            setup_view_window(app);
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
