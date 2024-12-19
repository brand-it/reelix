// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use crate::state::AppState;
use serde::Serialize;
use tauri::State;
use tera::Context;

#[derive(Serialize)]
struct Greeting {
    name: String,
}

#[derive(Serialize)]
struct Search {
    search: String,
}

#[tauri::command]
pub fn greet(name: &str, state: State<'_, AppState>) -> String {
    let greeting = Greeting {
        name: name.to_string(),
    };
    let context = Context::from_serialize(&greeting).expect("Failed to retrieve the value");

    match state.tera.render("greet.html", &context) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Template rendering error: {e}");
            format!("An error occurred: {e}")
        }
    }
}

#[tauri::command]
pub fn search(search: &str, state: State<'_, AppState>) -> String {
    let search = Search {
        search: search.to_string(),
    };
    let context = Context::from_serialize(&search).expect("Failed to retrieve the value");

    match state.tera.render("search/results.html", &context) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Template rendering error: {e}");
            format!("An error occurred: {e}")
        }
    }
}

// #[tauri::command]
// pub fn about(state: State<'_, AppState>) -> String {
//     match state.tera.render("about.html", &Context::new()) {
//         Ok(result) => result,
//         Err(e) => {
//             eprintln!("Template rendering error: {e}");
//             format!("An error occurred: {e}")
//         }
//     }
// }
