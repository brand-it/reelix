mod commands;
mod disk;
mod models;
mod progress_tracker;
mod services;
mod state;

use crate::models::optical_disk_info::OpticalDiskInfo;
use include_dir::{include_dir, Dir};
use state::AppState;
use std::sync::{Arc, Mutex, RwLock};
use sysinfo::System;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{App, Manager};
use tauri_plugin_store::StoreExt;
use tera::Tera;
use tokio::sync::broadcast;

// Embed the `templates` directory into the binary
static TEMPLATES_DIR: Dir = include_dir!("templates");
const ICON_BYTES: &[u8] = include_bytes!("../icons/menu-icon.png");

fn add_templates_from_dir(tera: &mut Tera, dir: &Dir) {
    for file in dir.files() {
        if let Some(path) = file.path().to_str() {
            let content = file
                .contents_utf8()
                .expect("Failed to read file content as UTF-8");
            let name = path.replace("templates/", ""); // Strip the base path for Tera
            println!("Adding template: {}", name);
            tera.add_raw_template(&name, content)
                .expect("Failed to add template");
        }
    }

    for subdir in dir.dirs() {
        add_templates_from_dir(tera, subdir);
    }
}

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
    let state: tauri::State<'_, AppState> = app_handle.state::<AppState>();
    let store = app.store("store.json").unwrap();
    let value = store.get("the_movie_db_key");

    if let Some(key) = value {
        if let Some(key_str) = key.as_str() {
            let mut movie_db_key = state.the_movie_db_key.write().unwrap();
            *movie_db_key = key_str.to_string();
        }
    }
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

fn kill_process(pid: u32) {
    println!("Killing process {:?}", pid);
    let mut system = System::new_all();
    system.refresh_all();
    let sys_pid = sysinfo::Pid::from_u32(pid.clone());
    if let Some(process) = system.process(sys_pid) {
        if process.kill() {
            println!("Killed {:?}", pid);
        } else {
            println!("Failed to kill process with PID {}", pid);
        }
    } else {
        println!("Process with PID {} not found", pid);
    }
}

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
                        println!("Failed to show window");
                    }
                };
            }
            _ => {
                println!("menu item {:?} not handled", event.id);
            }
        })
        .build(app)
        .expect("Failed to build tray icon");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut tera = Tera::default();
    add_templates_from_dir(&mut tera, &TEMPLATES_DIR);
    let app_state: AppState = AppState {
        tera: Arc::new(tera),
        query: Arc::new(Mutex::new(String::new())),
        the_movie_db_key: Arc::new(RwLock::new(String::new())),
        optical_disks: Arc::new(Mutex::new(Vec::<Arc<Mutex<OpticalDiskInfo>>>::new())),
        selected_optical_disk_id: Arc::new(Mutex::new(None)),
    };

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_http::init())
        .manage(app_state)
        .setup(|app| {
            setup_store(app);
            spawn_disk_listener(app);
            setup_tray_icon(app);
            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                window.hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            commands::index,
            commands::movie,
            commands::open_browser,
            commands::rip_one,
            commands::search,
            commands::the_movie_db,
            commands::tv,
        ])
        .build(tauri::generate_context!())
        .expect("error while building Tauri application");

    // Run the application with a run event callback to shutdown sidecar process
    app.run(|app_handle, event| {
        if let tauri::RunEvent::Exit = event {
            let state = app_handle.state::<AppState>();
            let disks = state
                .optical_disks
                .lock()
                .expect("Failed to get lock on optical_disks");

            // Iterate over the optical disks and kill the associated PID if it exists
            for disk in disks.iter() {
                let locked_disk = disk.lock().expect("failed to get lock on disk");
                let pid = locked_disk.pid.lock().expect("failed to lock pid");
                if pid.is_some() {
                    kill_process(pid.expect("pid is not defined"));
                }
            }
        }
    });
}
