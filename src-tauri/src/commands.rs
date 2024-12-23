mod the_movie_db;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::fmt;
use tauri::ipc::InvokeError;
use tauri::State;
use tera::Context;
use the_movie_db::TheMovieDb;

// Define your error
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub code: u16,
    pub message: String,
}

// impl std::error::Error for ApiError {}

// // Implement `Display` and `Error` so we can convert it to a string
// impl fmt::Display for ApiError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "Error {}: {}", self.code, self.message)
//     }
// }

// // The critical part: convert our `ApiError` into Tauri's `InvokeError`
// impl From<ApiError> for InvokeError {
//     fn from(err: ApiError) -> Self {
//         // Tauri expects an `InvokeError::from(String)`, so just convert `err` to a string
//         InvokeError::from(err.to_string())
//     }
// }
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
pub fn search(search: &str, state: State<'_, AppState>) -> Result<String, ApiError> {
    let api_key = "token".to_string();
    let language = "en-US".to_string();
    let movie_db = TheMovieDb::new(api_key, language);
    let response: Result<the_movie_db::SearchResponse, String> = movie_db.search_multi(search, 1);
    let response = match response {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Error from TMDB: {e}");
            // Instead of panic, return an ApiError with code=500
            return Err(ApiError {
                code: 500,
                message: format!("Error from TMDB: {e}"),
            });
        }
    };

    let search = Search {
        query: search.to_string(),
        search: response,
    };

    let context = Context::from_serialize(&search).expect("Failed to retrieve the value");

    match state.tera.render("search/results.html", &context) {
        Ok(result) => Ok(result),
        Err(e) => {
            eprintln!("Template rendering error: {:#?}", e);
            // Instead of panic, return an ApiError with code=500
            return Err(ApiError {
                code: 500,
                message: format!("An error occurred: {e}"),
            });
        }
    }
}
