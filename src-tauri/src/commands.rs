// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use crate::services::makemkvcon_parser;
use crate::services::the_movie_db;
use crate::services::the_movie_db::TheMovieDb;
use crate::state::AppState;
use serde::Serialize;
use serde_json::json;
use sysinfo::Disks;
use tauri::State;
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;
use tauri_plugin_store::StoreExt;
use tera::{Context, Tera};
#[derive(Serialize)]
pub struct ApiError {
    pub code: u16,
    pub message: String,
    pub api_key: Option<String>,
}

#[derive(Serialize)]
struct Search {
    query: String,
    search: the_movie_db::SearchResponse,
}

type ErrorHandler = fn(&tera::Error) -> ApiError;

// Usage and example code
// let result = render_template(
//     &state.tera,
//     "the_movie_db/index.html.turbo",
//     &Context::new(),
//     None, // No custom error handler
// );
//
// fn my_custom_error(e: &tera::Error) -> ApiError {
//     ApiError {
//         code: 404,
//         message: format!("Template not found or rendering failed: {e}"),
//     }
// }
//
// let result = render_template(
//     &state.tera,
//     "the_movie_db/index.html.turbo",
//     &Context::new(),
//     Some(my_custom_error),
// );
fn render_template(
    tera: &Tera,
    template_path: &str,
    context: &Context,
    on_error: Option<ErrorHandler>,
) -> Result<String, ApiError> {
    match tera.render(template_path, context) {
        Ok(result) => Ok(result),
        Err(e) => {
            eprintln!("Template rendering error: {:#?}", e);
            // Custom Error handler if provided
            if let Some(handler) = on_error {
                return Err(handler(&e));
            } else {
                return Err(ApiError {
                    code: 500,
                    message: format!("An error occurred: {e}"),
                    api_key: None,
                });
            }
        }
    }
}

#[tauri::command]
pub fn open_browser(url: &str, app_handle: tauri::AppHandle) -> String {
    let shell = app_handle.shell();

    #[cfg(target_os = "macos")]
    let browser_cmd = "open";

    #[cfg(target_os = "windows")]
    let browser_cmd = "cmd /C start";

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let browser_cmd = "xdg-open";

    tauri::async_runtime::block_on(async move {
        match shell.command(browser_cmd).args([url]).output().await {
            Ok(resp) => {
                return format!("Result: {:?}", String::from_utf8(resp.stdout));
            }
            Err(e) => {
                eprintln!("Open URL Error: {e}");
                return format!("Open URL Error: {}", e);
            }
        }
    })
}

#[tauri::command]
pub fn movie(id: u32, query: &str, state: State<'_, AppState>) -> Result<String, ApiError> {
    let api_key: String = {
        let locked_key = state.the_movie_db_key.lock().unwrap();
        locked_key.clone()
    };
    let language: String = "en-US".to_string();
    let movie_db: TheMovieDb = TheMovieDb::new(api_key, language);
    let response = movie_db.movie(id);
    let movie = match response {
        Ok(resp) => resp,
        Err(e) => {
            let api_key = {
                let locked_key = state.the_movie_db_key.lock().unwrap();
                locked_key.clone()
            };

            let mut context = Context::new();
            context.insert("code", "500");
            context.insert("message", &format!("Error from TMDB: {}", e.message));
            context.insert("api_key", &api_key);
            return render_template(&state.tera, "the_movie_db/index.html.turbo", &context, None);
        }
    };

    let mut context = Context::new();
    context.insert("movie", &movie);
    context.insert("query", query);
    render_template(&state.tera, "movies/index.html.turbo", &context, None)
}

// This is the entry point, basically it decide what to first show the user
#[tauri::command]
pub fn index(state: State<'_, AppState>) -> Result<String, ApiError> {
    let api_key: String = {
        let locked_key = state.the_movie_db_key.lock().unwrap();
        locked_key.clone()
    };
    let language = "en-US".to_string();
    let movie_db = TheMovieDb::new(api_key.clone(), language);
    let response = movie_db.search_multi("Martian", 1);

    match response {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Error from TMDB: {}", e.message);
            let mut context = Context::new();
            context.insert("api_key", &api_key);
            // let context = Context::from_serialize(&movie_db).expect("Failed to retrieve the value");
            return render_template(&state.tera, "the_movie_db/index.html.turbo", &context, None);
        }
    };

    render_template(
        &state.tera,
        "search/index.html.turbo",
        &Context::new(),
        None,
    )
}

#[tauri::command]
pub fn the_movie_db(
    key: &str,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, ApiError> {
    let mut movie_db_key: std::sync::MutexGuard<'_, String> =
        state.the_movie_db_key.lock().unwrap();
    *movie_db_key = key.to_string();
    let api_key = key.to_string();
    let language = "en-US".to_string();
    let movie_db = TheMovieDb::new(api_key, language);
    let response = movie_db.search_multi("Avengers", 1);
    match response {
        Ok(resp) => resp,
        Err(e) => {
            let api_error = ApiError {
                code: 500,
                message: e.message,
                api_key: None,
            };

            let context =
                Context::from_serialize(&api_error).expect("Failed to serialize api error");
            return render_template(&state.tera, "error.html.turbo", &context, None);
        }
    };
    let store = app_handle
        .store("store.json")
        .expect("Failed to load store.json");
    store.set("the_movie_db_key", json!(key));
    store.save().expect("Failed to save");

    render_template(
        &state.tera,
        "search/index.html.turbo",
        &Context::new(),
        None,
    )
}

#[tauri::command]
pub fn search(search: &str, state: State<'_, AppState>) -> Result<String, ApiError> {
    let api_key: String = {
        let locked_key = state.the_movie_db_key.lock().unwrap();
        locked_key.clone()
    };
    let language: String = "en-US".to_string();
    let movie_db: TheMovieDb = TheMovieDb::new(api_key, language);
    let response = movie_db.search_multi(search, 1);
    let response = match response {
        Ok(resp) => resp,
        Err(e) => {
            let api_key = {
                let locked_key = state.the_movie_db_key.lock().unwrap();
                locked_key.clone()
            };

            let mut context = Context::new();
            context.insert("code", "500");
            context.insert("message", &format!("Error from TMDB: {}", e.message));
            context.insert("api_key", &api_key);
            return render_template(&state.tera, "the_movie_db/index.html.turbo", &context, None);
        }
    };

    let search = Search {
        query: search.to_string(),
        search: response,
    };

    let context = Context::from_serialize(&search).expect("Failed to retrieve the value");

    render_template(&state.tera, "search/results.html.turbo", &context, None)
}

#[tauri::command]
pub fn mkvcommand(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, ApiError> {
    let sidecar_command = app_handle.shell().sidecar("makemkvcon").unwrap();
    let (mut rx, mut _child) = sidecar_command
        .args(["-r", "--cache=1", "info", "disc:9999"])
        .spawn()
        .expect("Failed to spawn sidecar");
    println!("mkvcommand");

    tauri::async_runtime::spawn(async move {
        // read events such as stdout
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(line_bytes) => {
                    let line = String::from_utf8_lossy(&line_bytes);
                    let parsed_stdout = makemkvcon_parser::parse_mkv_string(&line);
                    eprintln!("Stdout: {:?}", parsed_stdout);
                }
                CommandEvent::Stderr(line_bytes) => {
                    let line = String::from_utf8_lossy(&line_bytes);
                    eprintln!("Stderr: {}", line);
                }
                other => {
                    eprintln!("Other command event: {:?}", other);
                }
            }
            // if let CommandEvent::Stdout(line_bytes) = event {
            //     let line = String::from_utf8_lossy(&line_bytes);
            //     eprintln!("commandEvent line {:#?}", line);
            //     // write to stdin
            //     // child.write("message from Rust\n".as_bytes()).unwrap();
            // }
        }
    });

    let disks = Disks::new_with_refreshed_list();
    for disk in disks.list() {
        let fs_bytes = disk.file_system();
        let fs_str = fs_bytes.to_str().unwrap();

        // Check if removable + known optical file system
        if disk.is_removable() && (fs_str.contains("udf") || fs_str.contains("iso9660")) {
            println!("Likely optical media:");
            println!("  Name:        {:?}", disk.name());
            println!("  Mount point: {:?}", disk.mount_point());
            println!("  File system: {}", fs_str);
        } else {
            println!("Non-optical or unrecognized: {:?}", disk);
        }
    }
    render_template(
        &state.tera,
        "search/index.html.turbo",
        &Context::new(),
        None,
    )
}
