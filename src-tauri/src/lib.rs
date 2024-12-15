mod commands;
mod state;

use std::sync::Arc;
use tera::Tera;
use state::AppState;


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let tera = Tera::new("templates/**/*.html").unwrap_or_else(|e| {
        eprintln!("Parsing error(s): {e}");
        ::std::process::exit(1);
    });

    let app_state = AppState {
        tera: Arc::new(tera),
    };
    tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![commands::greet, commands::about])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
