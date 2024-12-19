mod commands;
mod state;

use state::AppState;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tera::Tera;

fn add_templates_from_dir(tera: &mut tera::Tera, dir: &Path) {
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir).expect("Failed to read directory") {
            let entry = entry.expect("Failed to read entry");
            let path = entry.path();

            if path.is_dir() {
                // Recurse into subdirectory
                add_templates_from_dir(tera, &path);
            } else if path.is_file() {
                // Process the file
                if let Some(path_str) = path.to_str() {
                    let content = fs::read_to_string(&path).expect("Failed to read file content");
                    let name: String = path_str.replace("templates/", "");
                    println!("adding templates: {}", name);
                    tera.add_raw_template(&name, &content)
                        .expect("Failed to add template");
                }
            }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut tera = Tera::default();

    add_templates_from_dir(&mut tera, Path::new("templates"));

    let app_state = AppState {
        tera: Arc::new(tera),
    };

    tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![commands::greet, commands::search])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
