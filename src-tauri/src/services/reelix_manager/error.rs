//! Error types for Reelix Manager API operations.

use serde::{Deserialize, Serialize};

/// OAuth2 client ID for Reelix Manager authentication
pub const CLIENT_ID: &str = "reelix-client";

/// OAuth2 scopes requested for Reelix Manager authentication
pub const SCOPE: &str = "search upload";

/// Response from OAuth2 device authorization endpoint
#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: String,
    pub expires_in: u32,
    pub interval: u32,
}

/// Response from OAuth2 token endpoint
#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
}

/// Errors that can occur during OAuth2 token polling
#[derive(Debug)]
pub enum PollError {
    /// Authorization is still pending user action
    Pending,
    /// Server requested slower polling
    SlowDown,
    /// User denied authorization
    AccessDenied,
    /// Device code has expired
    ExpiredToken,
    /// HTTP or parsing error
    Http(String),
}

/// Error type for Reelix Manager API operations
#[derive(Debug, Serialize, Deserialize)]
pub struct Error {
    pub message: String,
}

impl Error {
    /// Create a new error with the given message
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }

    /// Create an unauthorized error
    pub fn unauthorized() -> Self {
        Self {
            message: "unauthorized".to_string(),
        }
    }
}
