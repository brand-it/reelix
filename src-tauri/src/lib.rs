mod commands;
mod state;

use include_dir::{include_dir, Dir};
use state::AppState;
use std::sync::Arc;
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

    let app_state = AppState {
        tera: Arc::new(tera),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![commands::greet, commands::search])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
