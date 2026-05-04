//! Shared request builder and response types for Reelix Manager API calls.
//!
//! Provides `ApiRequest` (sync) and `AsyncApiRequest` (async) builder types
//! that encapsulate common request construction patterns: URL building,
//! authorization headers, content-type handling, and error mapping.
//!
//! `send()` returns a wrapper with `status`, `headers`, and `body` already
//! read, eliminating per-module boilerplate.

use std::collections::HashMap;

use tauri_plugin_http::reqwest::header::HeaderMap;
use tauri_plugin_http::reqwest::Method;
use tauri_plugin_http::reqwest::blocking::Client as BlockingClient;
use tauri_plugin_http::reqwest::blocking::Response as BlockingResponse;
use tauri_plugin_http::reqwest::Client as AsyncClient;
use tauri_plugin_http::reqwest::Response as AsyncResponse;

use super::error::Error;

// ===========================
// Response Wrappers
// ===========================

/// Response from a sync API request.
///
/// The body is already read as text. Use `parse_json()` to deserialize it.
pub struct ApiResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl ApiResponse {
    /// Deserialize the body as JSON.
    pub fn parse_json<T: serde::de::DeserializeOwned>(&self) -> Result<T, Error> {
        serde_json::from_str(&self.body).map_err(|e| Error::new(format!("Failed to parse JSON response: {e}. Body: {}", self.body)))
    }
}

/// Response from an async API request.
///
/// The body is already read as text. Use `parse_json()` to deserialize it.
pub struct AsyncApiResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl AsyncApiResponse {
    /// Deserialize the body as JSON.
    pub fn parse_json<T: serde::de::DeserializeOwned>(&self) -> Result<T, Error> {
        serde_json::from_str(&self.body).map_err(|e| Error::new(format!("Failed to parse JSON response: {e}. Body: {}", self.body)))
    }
}

/// Extract headers from a reqwest HeaderMap into a HashMap for easy access.
fn extract_headers(headers: &HeaderMap) -> HashMap<String, String> {
    headers
        .iter()
        .map(|(name, value)| (name.to_string(), value.to_str().unwrap_or_default().to_string()))
        .collect()
}

// ===========================
// Sync Builder
// ===========================

/// Sync request builder for Reelix Manager API calls.
pub struct ApiRequest {
    client: BlockingClient,
    host: String,
    token: String,
    method: Method,
    path: String,
    json: Option<serde_json::Value>,
    body: Option<Vec<u8>>,
    extra_headers: HashMap<String, String>,
}

impl ApiRequest {
    pub fn new(client: BlockingClient, host: String, token: String) -> Self {
        Self {
            client,
            host,
            token,
            method: Method::POST,
            path: String::new(),
            json: None,
            body: None,
            extra_headers: HashMap::new(),
        }
    }

    /// Set the HTTP method explicitly.
    pub fn method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    /// Set method to POST and set the path.
    pub fn post(mut self, path: &str) -> Self {
        self.method = Method::POST;
        self.path = path.to_string();
        self
    }

    /// Set method to HEAD and set the path.
    pub fn head(mut self, path: &str) -> Self {
        self.method = Method::HEAD;
        self.path = path.to_string();
        self
    }

    /// Set method to PATCH and set the path.
    pub fn patch(mut self, path: &str) -> Self {
        self.method = Method::PATCH;
        self.path = path.to_string();
        self
    }

    /// Set the URL path without changing the method.
    pub fn path(mut self, path: &str) -> Self {
        self.path = path.to_string();
        self
    }

    /// Add an extra header. Overwrites any previously set header with the same key,
    /// including auto-added defaults like `Content-Type`.
    pub fn header(mut self, key: &str, value: String) -> Self {
        self.extra_headers.insert(key.to_string(), value);
        self
    }

    /// Set a JSON body. Auto-adds `Content-Type: application/json`.
    pub fn json(mut self, value: serde_json::Value) -> Self {
        self.json = Some(value);
        self
    }

    /// Set a raw byte body. Takes precedence over `.json()` if both are set.
    pub fn body(mut self, data: Vec<u8>) -> Self {
        self.body = Some(data);
        self
    }

    /// Execute the request and return a wrapper with status, headers, and body.
    pub fn send(self) -> Result<ApiResponse, Error> {
        let url = format!("{}{}", self.host, self.path);
        let mut builder = self.client.request(self.method, &url).header(
            "Authorization",
            format!("Bearer {}", self.token),
        );

        // Auto-add Content-Type only when .json() is used
        if self.json.is_some() {
            builder = builder.header("Content-Type", "application/json");
        }

        // Extra headers are applied after defaults, allowing them to override
        for (key, value) in self.extra_headers {
            builder = builder.header(&key, value);
        }

        let builder = if let Some(json) = self.json {
            builder.json(&json)
        } else if let Some(body) = self.body {
            builder.body(body)
        } else {
            builder
        };

        let resp: BlockingResponse = builder.send().map_err(|e| Error::new(format!("Request failed: {e}")))?;

        let status = resp.status().as_u16();
        let headers = extract_headers(resp.headers());
        let body = resp.text().unwrap_or_default();

        if status == 401 || status == 422 {
            return Err(Error::unauthorized());
        }

        if !(200..300).contains(&status) {
            return Err(Error::new(format!(
                "Request failed with status {status}: {body}"
            )));
        }

        Ok(ApiResponse { status, headers, body })
    }
}

// ===========================
// Async Builder
// ===========================

/// Async request builder for Reelix Manager API calls.
pub struct AsyncApiRequest {
    client: AsyncClient,
    host: String,
    token: String,
    method: Method,
    path: String,
    json: Option<serde_json::Value>,
    body: Option<Vec<u8>>,
    extra_headers: HashMap<String, String>,
}

impl AsyncApiRequest {
    pub fn new(client: AsyncClient, host: String, token: String) -> Self {
        Self {
            client,
            host,
            token,
            method: Method::POST,
            path: String::new(),
            json: None,
            body: None,
            extra_headers: HashMap::new(),
        }
    }

    /// Set the HTTP method explicitly.
    pub fn method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    /// Set method to POST and set the path.
    pub fn post(mut self, path: &str) -> Self {
        self.method = Method::POST;
        self.path = path.to_string();
        self
    }

    /// Set method to HEAD and set the path.
    pub fn head(mut self, path: &str) -> Self {
        self.method = Method::HEAD;
        self.path = path.to_string();
        self
    }

    /// Set method to PATCH and set the path.
    pub fn patch(mut self, path: &str) -> Self {
        self.method = Method::PATCH;
        self.path = path.to_string();
        self
    }

    /// Set the URL path without changing the method.
    pub fn path(mut self, path: &str) -> Self {
        self.path = path.to_string();
        self
    }

    /// Add an extra header. Overwrites any previously set header with the same key,
    /// including auto-added defaults like `Content-Type`.
    pub fn header(mut self, key: &str, value: String) -> Self {
        self.extra_headers.insert(key.to_string(), value);
        self
    }

    /// Set a JSON body. Auto-adds `Content-Type: application/json`.
    pub fn json(mut self, value: serde_json::Value) -> Self {
        self.json = Some(value);
        self
    }

    /// Set a raw byte body. Takes precedence over `.json()` if both are set.
    pub fn body(mut self, data: Vec<u8>) -> Self {
        self.body = Some(data);
        self
    }

    /// Execute the request and return a wrapper with status, headers, and body.
    pub async fn send(self) -> Result<AsyncApiResponse, Error> {
        let url = format!("{}{}", self.host, self.path);
        let mut builder = self.client.request(self.method, &url).header(
            "Authorization",
            format!("Bearer {}", self.token),
        );

        // Auto-add Content-Type only when .json() is used
        if self.json.is_some() {
            builder = builder.header("Content-Type", "application/json");
        }

        // Extra headers are applied after defaults, allowing them to override
        for (key, value) in self.extra_headers {
            builder = builder.header(&key, value);
        }

        let builder = if let Some(json) = self.json {
            builder.json(&json)
        } else if let Some(body) = self.body {
            builder.body(body)
        } else {
            builder
        };

        let resp: AsyncResponse = builder.send().await.map_err(|e| Error::new(format!("Request failed: {e}")))?;

        let status = resp.status().as_u16();
        let headers = extract_headers(resp.headers());
        let body = resp.text().await.unwrap_or_default();

        if status == 401 || status == 422 {
            return Err(Error::unauthorized());
        }

        if !(200..300).contains(&status) {
            return Err(Error::new(format!(
                "Request failed with status {status}: {body}"
            )));
        }

        Ok(AsyncApiResponse { status, headers, body })
    }
}
