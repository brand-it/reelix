mod commands;
mod disk;
mod models;
mod services;
mod state;

use chrono::DateTime;
use chrono::NaiveDate;
use disk::OpticalDiskInfo;
use include_dir::{include_dir, Dir};
use state::AppState;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{App, Manager};
use tauri_plugin_store::StoreExt;
use tera::{to_value, Result as TeraResult, Tera, Value};
use tokio::sync::broadcast;

// Embed the `templates` directory into the binary
static TEMPLATES_DIR: Dir = include_dir!("templates");

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
    let (sender, receiver) = broadcast::channel::<Vec<diff::Result<disk::OpticalDiskInfo>>>(16);
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
            let mut movie_db_key = state.the_movie_db_key.lock().unwrap();
            *movie_db_key = key_str.to_string();
        }
    }
    store.close_resource();
}

/// Custom filter that formats a datetime string into "YYYY"
pub fn to_year(value: &Value, _args: &HashMap<String, Value>) -> TeraResult<Value> {
    let date_str = value
        .as_str()
        .ok_or("format_date filter: expected a string")?;

    // Try parsing the string as an RFC3339 datetime.
    let formatted = if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
        dt.format("%Y").to_string()
    } else if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        // Fallback: if it's already just a date, use it.
        date.format("%Y").to_string()
    } else {
        return Err(format!("format_date filter: failed to parse date: {}", date_str).into());
    };

    to_value(formatted).map_err(Into::into)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut tera = Tera::default();
    tera.register_filter("to_year", to_year);
    add_templates_from_dir(&mut tera, &TEMPLATES_DIR);
    let app_state: AppState = AppState {
        tera: Arc::new(tera),
        the_movie_db_key: Arc::new(Mutex::new(String::new())),
        optical_disks: Arc::new(Mutex::new(Vec::<Arc<Mutex<OpticalDiskInfo>>>::new())),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_http::init())
        .manage(app_state)
        .setup(|app| {
            setup_store(app);
            spawn_disk_listener(app);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::index,
            commands::movie,
            commands::open_browser,
            commands::search,
            commands::the_movie_db,
            commands::rip_one
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
