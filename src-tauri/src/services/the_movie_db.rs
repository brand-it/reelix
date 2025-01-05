use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// use tauri::http::response;
use tauri_plugin_http::reqwest::blocking::Client;
// Struct for the TMDB Client
pub struct TheMovieDb {
    api_key: String,
    language: String,
    client: Client,
}
#[derive(Serialize, Deserialize)]
pub struct Error {
    pub code: u16,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct SearchError {
    status_code: u16,
    status_message: String,
    success: bool,
}

#[derive(Serialize, Deserialize)]
pub struct SearchResult {
    #[serde(default)]
    name: String,
    #[serde(default)]
    original_name: String,
    adult: bool,
    backdrop_path: Option<String>,
    #[serde(default)]
    genre_ids: Vec<u32>,
    id: u32,
    media_type: String,
    #[serde(default)]
    original_language: String,
    original_title: Option<String>,
    #[serde(default)]
    overview: String,
    popularity: f64,
    profile_path: Option<String>,
    poster_path: Option<String>,
    release_date: Option<String>,
    first_air_date: Option<String>,
    title: Option<String>,
    #[serde(default)]
    video: bool,
    #[serde(default)]
    vote_average: f64,
    #[serde(default)]
    vote_count: u32,
}

// Struct to represent the full response
#[derive(Serialize, Deserialize)]
pub struct SearchResponse {
    page: u32,
    results: Vec<SearchResult>,
    total_pages: u32,
    total_results: u32,
}

impl TheMovieDb {
    pub fn new(api_key: String, language: String) -> Self {
        TheMovieDb {
            api_key,
            language,
            client: Client::new(),
        }
    }

    // Method to perform a GET request to the "search/multi" endpoint
    // let api_key = "your_api_key_here".to_string();
    // let language = "en-US".to_string();
    // let movie_db = TheMovieDb::new(api_key, language);

    // // Example query to search for "Inception"
    // match movie_db.search_multi("Inception") {
    //     Ok(results) => println!("Results: {:#?}", results),
    //     Err(err) => eprintln!("Error: {:?}", err),
    // }
    pub fn search_multi(&self, query: &str, page: u32) -> Result<SearchResponse, Error> {
        let url = "https://api.themoviedb.org/3/search/multi";
        let page_string = page.to_string();

        // Build the query parameters
        let mut params = HashMap::new();
        params.insert("api_key", self.api_key.as_str());
        params.insert("language", self.language.as_str());
        params.insert("query", query);
        params.insert("page", &page_string);
        // dbg!(&params.clone());
        // Perform the GET request & Error handling
        let response = self
            .client
            .get(url)
            .query(&params)
            .send()
            .map_err(|e| Error {
                code: 500,
                message: format!("Request error: {:?}", e),
            })?;
        let status = response.status();
        let text_body = response.text().map_err(|e| Error {
            code: 500,
            message: format!("Request error reading text: {:?}", e),
        })?;

        if !status.is_success() {
            match self.parse_error(&text_body) {
                Ok(response) => {
                    return Err(Error {
                        code: response.status_code,
                        message: response.status_message,
                    });
                }
                Err(err) => return Err(err),
            };
        };
        self.parse_response(&text_body)
    }

    fn parse_response(&self, text_body: &str) -> Result<SearchResponse, Error> {
        serde_json::from_str(&text_body).map_err(|e| Error {
            code: 500,
            message: format!("Failed to parse response JSON: {:?}", e),
        })
    }

    fn parse_error(&self, text_body: &str) -> Result<SearchError, Error> {
        serde_json::from_str(&text_body).map_err(|e| Error {
            code: 500,
            message: format!("Failed to parse response JSON: {:?}", e),
        })
    }
}
