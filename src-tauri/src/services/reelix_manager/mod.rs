//! Reelix Manager API client for GraphQL operations and tus uploads.
//!
//! This module provides a client for interacting with the Reelix Manager API,
//! including authentication (OAuth2 device flow), media search, metadata queries,
//! and resumable file uploads via the tus protocol.

mod active_uploads;
mod client;
mod error;
mod finalize_upload;
mod find_movie;
mod find_season;
mod find_tv;
pub mod oauth;
mod search;
mod session_by_id;
mod tus_create;
mod tus_offset;
mod tus_upload;
mod types;
pub use client::{ApiRequest, AsyncApiRequest};
pub use error::{DeviceCodeResponse, Error, PollError, TokenResponse};
pub use finalize_upload::FinalizeResponse;
pub use tus_types::UploadSession;
pub use types::*;

use crate::state::AppState;
use tauri_plugin_http::reqwest::blocking::Client;
use tauri_plugin_http::reqwest::Client as AsyncClient;

/// Reelix Manager API client that encapsulates connection state
#[derive(Clone)]
pub struct ReelixManager {
    pub(crate) host: String,
    pub(crate) token: String,
    pub(crate) client: Client,
    pub(crate) async_client: AsyncClient,
}

impl ReelixManager {
    /// Create a new ReelixManager from AppState
    /// Extracts host and token from the app state automatically
    pub fn new(state: &AppState) -> Self {
        let host = state.get_manager_host().unwrap_or_default();
        let token = state.get_manager_token().unwrap_or_default();
        Self {
            host,
            token,
            client: Client::new(),
            async_client: AsyncClient::new(),
        }
    }

    /// Create a new ReelixManager with explicit credentials
    /// Useful for testing or edge cases
    pub fn with_credentials(host: impl Into<String>, token: impl Into<String>) -> Self {
        let host = host.into();
        let token = token.into();
        Self {
            host,
            token,
            client: Client::new(),
            async_client: AsyncClient::new(),
        }
    }

    /// Verify that the current token is valid
    pub fn verify_token(&self) -> Result<bool, Error> {
        let body = serde_json::json!({ "query": "{ __typename }" });

        match self.sync_request().path("/graphql").json(body).send() {
            Ok(_) => Ok(true),
            Err(e) if e.message == "unauthorized" => Ok(false),
            Err(e) => Err(e),
        }
    }
    /// Create a sync request builder for authenticated API calls.
    pub fn sync_request(&self) -> ApiRequest {
        ApiRequest::new(self.client.clone(), self.host.clone(), self.token.clone())
    }

    /// Create an async request builder for authenticated API calls.
    pub fn async_request(&self) -> AsyncApiRequest {
        AsyncApiRequest::new(
            self.async_client.clone(),
            self.host.clone(),
            self.token.clone(),
        )
    }

    /// Search for movies and TV shows
    /// GraphQL query: `searchMulti(query: String, page: Int)`
    pub fn search(&self, query: &str, page: u32) -> Result<SearchResponse, Error> {
        search::execute(self, query, page)
    }

    /// Find a movie by ID
    /// GraphQL query: `movie(id: Int)`
    pub fn find_movie(&self, id: u32) -> Result<MovieResponse, Error> {
        find_movie::execute(self, id)
    }

    /// Find a TV show by ID
    /// GraphQL query: `tv(id: Int)`
    pub fn find_tv(&self, id: u32) -> Result<TvResponse, Error> {
        find_tv::execute(self, id)
    }

    /// Find a season by TV show ID and season number
    /// GraphQL query: `season(tvId: Int, seasonNumber: Int)`
    pub fn find_season(&self, tv_id: u32, season_number: u32) -> Result<SeasonResponse, Error> {
        find_season::execute(self, tv_id, season_number)
    }

    // ===========================
    // Tus Upload Methods
    // ===========================

    /// Create a new tus upload session
    /// Returns the upload ID which should be used for subsequent upload operations
    /// API: POST /files with tus headers
    pub async fn create_tus_upload(&self, file_size: u64, filename: &str) -> Result<String, Error> {
        tus_create::execute(self, file_size, filename).await
    }

    /// Get the current offset of an upload (how many bytes have been uploaded)
    /// API: HEAD /files/:id with tus headers
    pub async fn get_upload_offset(&self, upload_id: &str) -> Result<u64, Error> {
        tus_offset::execute(self, upload_id).await
    }

    /// Upload a chunk of data to an existing tus upload
    /// Returns the new offset after the upload
    /// API: PATCH /files/:id with tus headers
    pub async fn upload_chunk(
        &self,
        upload_id: &str,
        offset: u64,
        data: &[u8],
    ) -> Result<u64, Error> {
        tus_upload::execute(self, upload_id, offset, data).await
    }

    /// Finalize an upload by associating it with TMDB metadata
    /// GraphQL mutation: `finalizeUpload(input: FinalizeUploadInput!)`
    pub async fn finalize_upload(
        &self,
        upload_id: &str,
        tmdb_id: u32,
        media_type: &str,
        season_number: Option<u32>,
        episode_number: Option<u32>,
    ) -> Result<FinalizeResponse, Error> {
        finalize_upload::execute(
            self,
            upload_id,
            tmdb_id,
            media_type,
            season_number,
            episode_number,
        )
        .await
    }

    /// Get a specific upload session by ID
    /// GraphQL query: `uploadSession(id: ID!)`
    pub async fn get_upload_session_by_id(
        &self,
        upload_id: &str,
    ) -> Result<Option<UploadSession>, Error> {
        session_by_id::execute(self, upload_id).await
    }

    /// Get all active upload sessions
    /// GraphQL query: `uploadSessions`
    pub async fn get_active_uploads(&self) -> Result<Vec<UploadSession>, Error> {
        active_uploads::execute(self).await
    }
}

// ===========================
// Tus Upload Response Types
// ===========================

mod tus_types {
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct UploadSession {
        pub id: String,
        pub filename: String,
        #[serde(rename = "uploadLength")]
        pub upload_length: String,
        #[serde(rename = "uploadOffset")]
        pub upload_offset: String,
        pub status: String,
    }

    impl UploadSession {
        pub fn upload_length_u64(&self) -> u64 {
            self.upload_length.parse().unwrap_or(0)
        }

        pub fn upload_offset_u64(&self) -> u64 {
            self.upload_offset.parse().unwrap_or(0)
        }
    }
}
