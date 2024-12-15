mod commands;
mod state;

use state::AppState;
use std::sync::Arc;
use tauri::path::BaseDirectory;
use tauri::Manager;
use tera::Tera;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // let tera = Tera::new("resources/templates/**/*.html").unwrap_or_else(|e| {
    //     eprintln!("Parsing error(s): {e}");
    //     ::std::process::exit(1);
    // });

    // let app_state = AppState {
    //     tera: Arc::new(tera),
    // };
    tauri::Builder::default()
        .setup(|app| {
            // The path specified must follow the same syntax as defined in
            // `tauri.conf.json > bundle > resources`
            let template_path = app.path().resolve("templates", BaseDirectory::Resource)?;
            // Convert the path to a string
            let template_path_str = template_path
                .to_str()
                .ok_or("Failed to convert template path to string")?;
            let tera = Tera::new(&format!("{template_path_str}/**/*.html")).unwrap_or_else(|e| {
                eprintln!("Parsing error(s): {e}");
                ::std::process::exit(1);
            });

            let app_state = AppState {
                tera: Arc::new(tera),
            };
            app.manage(app_state);
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![commands::greet, commands::about])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
