mod commands;
mod disk;
mod services;
mod state;

use include_dir::{include_dir, Dir};
use state::AppState;
use std::sync::Mutex;
use std::sync::{atomic::AtomicBool, Arc};
use tauri::Manager;
use tauri_plugin_store::StoreExt;
use tera::Tera;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut tera = Tera::default();

    add_templates_from_dir(&mut tera, &TEMPLATES_DIR);

    let app_state: AppState = AppState {
        tera: Arc::new(tera),
        the_movie_db_key: Arc::new(Mutex::new(String::new())),
    };
    let runtime: tokio::runtime::Runtime =
        tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    let change_flag = Arc::new(AtomicBool::new(false));
    runtime.spawn(async {
        disk::watch_for_changes(change_flag).await;
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_http::init())
        .manage(app_state)
        .setup(|app| {
            let app_handle = app.handle();
            let state = app_handle.state::<AppState>();
            let store = app.store("store.json")?;
            let value = store.get("the_movie_db_key");

            if let Some(key) = value {
                if let Some(key_str) = key.as_str() {
                    let mut movie_db_key = state.the_movie_db_key.lock().unwrap();
                    *movie_db_key = key_str.to_string();
                }
            }
            store.close_resource();

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::index,
            commands::movie,
            commands::open_browser,
            commands::search,
            commands::the_movie_db,
            commands::mkvcommand
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
