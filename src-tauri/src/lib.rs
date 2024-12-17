mod commands;
mod state;

use include_dir::{include_dir, Dir};
use state::AppState;
use std::sync::Arc;
use tera::Tera;

static TEMPLATES_DIR: Dir = include_dir!("templates");

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut tera = Tera::default();
    // Takes all the template files and compiles them with the binary
    for file in TEMPLATES_DIR.files() {
        if let Some(path) = file.path().to_str() {
            let content = file.contents_utf8().unwrap();
            tera.add_raw_template(path, content)
                .expect("Failed to add template");
        }
    }

    let app_state = AppState {
        tera: Arc::new(tera),
    };

    tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::search
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
