use crate::models::movie_db::{
    MovieReleaseDatesResponse, MovieResponse, SearchResponse, TvResponse,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri_plugin_http::reqwest::blocking::{Client, RequestBuilder};

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
static URL_ENDPOINT: &str = "https://api.themoviedb.org/3";

impl TheMovieDb {
    pub fn new(api_key: &String, language: &str) -> Self {
        TheMovieDb {
            api_key: api_key.to_owned(),
            language: language.to_owned(),
            client: Client::new(),
        }
    }

    // Method to perform a GET request to the "search/multi" endpoint
    // let api_key = "your_api_key_here".to_string();
    // let language = "en-US".to_string();
    // let movie_db = TheMovieDb::new(api_key, language);

    // Example query to search for "Inception"
    // match movie_db.search_multi("Inception") {
    //     Ok(results) => println!("Results: {:#?}", results),
    //     Err(err) => eprintln!("Error: {:?}", err),
    // }
    pub fn search_multi(&self, query: &str, page: u32) -> Result<SearchResponse, Error> {
        let url = format!("{}/{}", URL_ENDPOINT, "search/multi");
        let page = &page.to_string();

        let mut params = HashMap::new();
        params.insert("api_key", self.api_key.as_str());
        params.insert("language", self.language.as_str());
        params.insert("query", query);
        params.insert("page", page);

        let request = self.client.get(url).query(&params);
        self.send_request(request)
    }

    pub fn movie(&self, id: u32) -> Result<MovieResponse, Error> {
        let url = format!("{}/movie/{}", URL_ENDPOINT, id);

        let mut params: HashMap<&str, &str> = HashMap::new();
        params.insert("api_key", self.api_key.as_str());

        let request = self.client.get(url).query(&params);
        self.send_request(request)
    }

    pub fn tv(&self, id: u32) -> Result<TvResponse, Error> {
        let url = format!("{}/tv/{}", URL_ENDPOINT, id);

        let mut params: HashMap<&str, &str> = HashMap::new();
        params.insert("api_key", self.api_key.as_str());

        let request = self.client.get(url).query(&params);
        self.send_request(request)
    }

    pub fn movie_release_dates(&self, id: u32) -> Result<MovieReleaseDatesResponse, Error> {
        let url = format!("https://api.themoviedb.org/3/movie/{}/release_dates", id);

        // Build the query parameters
        let mut params: HashMap<&str, &str> = HashMap::new();
        params.insert("api_key", self.api_key.as_str());
        let request = self.client.get(url).query(&params);

        self.send_request(request)
    }

    fn send_request<T: DeserializeOwned>(&self, request: RequestBuilder) -> Result<T, Error> {
        let response = request.send().map_err(|e| Error {
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
        }

        serde_json::from_str::<T>(&text_body).map_err(|e| Error {
            code: 500,
            message: format!("Failed to parse response JSON: {:?}, {:?}", e, text_body),
        })
    }

    fn parse_error(&self, text_body: &str) -> Result<SearchError, Error> {
        serde_json::from_str(&text_body).map_err(|e| Error {
            code: 500,
            message: format!("Failed to parse response JSON: {:?}, {:?}", e, text_body),
        })
    }
}
