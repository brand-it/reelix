mod the_movie_db;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use crate::state::AppState;
use serde::Serialize;
use tauri::State;
use tera::Context;
use the_movie_db::TheMovieDb;
#[derive(Serialize)]
struct Greeting {
    name: String,
}

#[derive(Serialize)]
struct Search {
    query: String,
    search: the_movie_db::SearchResponse,
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
    let api_key = "token".to_string();
    let language = "en-US".to_string();
    let movie_db = TheMovieDb::new(api_key, language);
    let response: Result<the_movie_db::SearchResponse, String> = movie_db.search_multi(search, 1);
    let search = Search {
        query: search.to_string(),
        search: response.expect("Failed"),
    };

    let context = Context::from_serialize(&search).expect("Failed to retrieve the value");

    match state.tera.render("search/results.html", &context) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Template rendering error: {:#?}", e);
            format!("An error occurred: {e}")
        }
    }
}
