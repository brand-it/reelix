use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};
use reqwest::{blocking::Client, Error as ReqwestError, Response};
use serde_json::Value;
use url::Url;

// Configuration Structs
#[derive(Debug)]
pub struct Config {
    pub api_key: Option<String>,
    pub language: Option<String>,
}

impl Config {
    pub fn new(api_key: Option<String>, language: Option<String>) -> Self {
        Config { api_key, language }
    }

    pub fn settings() -> Config {
        Config {
            api_key: Some("your_api_key_here".to_string()), // Replace with real config
            language: Some("en-US".to_string()),           // Default language
        }
    }
}

// Cache Entry Struct
struct CacheEntry {
    value: Value,
    expires_at: SystemTime,
}

// In-Memory Cache Implementation
struct Cache {
    data: Mutex<HashMap<String, CacheEntry>>,
}

impl Cache {
    pub fn new() -> Self {
        Cache {
            data: Mutex::new(HashMap::new()),
        }
    }

    pub fn fetch<F>(&self, key: &str, ttl: Duration, fetch_fn: F) -> Value
    where
        F: FnOnce() -> Value,
    {
        let mut cache = self.data.lock().unwrap();
        if let Some(entry) = cache.get(key) {
            if SystemTime::now() < entry.expires_at {
                return entry.value.clone();
            }
        }
        let value = fetch_fn();
        cache.insert(
            key.to_string(),
            CacheEntry {
                value: value.clone(),
                expires_at: SystemTime::now() + ttl,
            },
        );
        value
    }
}

// Main API Client
pub struct TheMovieDb {
    config: Config,
    cache: Cache,
    client: Client,
}

impl TheMovieDb {
    const HOST: &'static str = "api.themoviedb.org";
    const VERSION: &'static str = "3";
    const CACHE_TTL: Duration = Duration::from_secs(7 * 24 * 60 * 60); // 7 days

    pub fn new(api_key: Option<String>, language: Option<String>) -> Self {
        TheMovieDb {
            config: Config::new(api_key, language),
            cache: Cache::new(),
            client: Client::new(),
        }
    }

    pub fn results(&self, use_cache: bool) -> Result<Value, ReqwestError> {
        if use_cache {
            let cache_key = format!("{:?}", self.query_params());
            Ok(self.cache.fetch(&cache_key, Self::CACHE_TTL, || {
                self.get().unwrap_or(Value::Null)
            }))
        } else {
            self.get()
        }
    }

    fn get(&self) -> Result<Value, ReqwestError> {
        let uri = self.build_uri();
        let query_params = self.query_params();
        let response = self
            .client
            .get(uri)
            .query(&query_params)
            .send()?;

        if response.status().is_success() {
            let body = response.json::<Value>()?;
            Ok(body)
        } else {
            self.error!(response);
            Err(ReqwestError::new())
        }
    }

    fn build_uri(&self) -> Url {
        let mut url = Url::parse(&format!(
            "https://{}/{}",
            Self::HOST,
            Self::VERSION
        ))
        .unwrap();
        url.set_path(&self.path());
        url
    }

    fn path(&self) -> String {
        "your_path_here".to_string() // Replace with dynamic logic
    }

    fn query_params(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        if let Some(api_key) = &self.config.api_key {
            params.insert("api_key".to_string(), api_key.clone());
        }
        if let Some(language) = &self.config.language {
            params.insert("language".to_string(), language.clone());
        }
        params
    }

    fn error!(&self, response: Response) {
        eprintln!(
            "Error: {}",
            response.text().unwrap_or_else(|_| "Unknown error".to_string())
        );
    }
}

fn main() {
    let api_key = Some("your_api_key_here".to_string());
    let movie_db = TheMovieDb::new(api_key, Some("en-US".to_string()));
    match movie_db.results(true) {
        Ok(results) => println!("Results: {:?}", results),
        Err(err) => eprintln!("Error: {:?}", err),
    }
}
