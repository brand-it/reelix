use crate::the_movie_db::models::{
    GqlMovieResponse, GqlSeasonResponse, GqlSearchResponse, GqlTvResponse, SearchResponse,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tauri_plugin_http::reqwest::blocking::Client;

use crate::the_movie_db::{MovieResponse, SeasonResponse, TvResponse};

const CLIENT_ID: &str = "reelix-client";
const SCOPE: &str = "search upload";

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: String,
    pub expires_in: u32,
    pub interval: u32,
}

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
}

#[derive(Debug)]
pub enum PollError {
    Pending,
    SlowDown,
    AccessDenied,
    ExpiredToken,
    Http(String),
}

#[derive(Debug, Deserialize)]
struct OAuthError {
    error: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Error {
    pub message: String,
}

impl Error {
    fn new(msg: impl Into<String>) -> Self {
        Self { message: msg.into() }
    }

    pub fn unauthorized() -> Self {
        Self { message: "unauthorized".to_string() }
    }

    pub fn is_unauthorized(&self) -> bool {
        self.message == "unauthorized"
    }
}

pub fn authorize_device(host: &str) -> Result<DeviceCodeResponse, Error> {
    let url = format!("{host}/oauth/authorize_device");
    let body = serde_json::json!({
        "client_id": CLIENT_ID,
        "scope": SCOPE,
    });

    let client = Client::new();
    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .map_err(|e| Error::new(format!("Request failed: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        return Err(Error::new(format!("Server error: {status}")));
    }

    resp.json::<DeviceCodeResponse>()
        .map_err(|e| Error::new(format!("Failed to parse device code response: {e}")))
}

pub fn poll_token(host: &str, device_code: &str) -> Result<TokenResponse, PollError> {
    let url = format!("{host}/oauth/token");
    let body = serde_json::json!({
        "client_id": CLIENT_ID,
        "grant_type": "urn:ietf:params:oauth:grant-type:device_code",
        "device_code": device_code,
    });

    let client = Client::new();
    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .map_err(|e| PollError::Http(format!("Request failed: {e}")))?;

    let status = resp.status();

    if status.is_success() {
        return resp
            .json::<TokenResponse>()
            .map_err(|e| PollError::Http(format!("Failed to parse token: {e}")));
    }

    let oauth_error: OAuthError = resp
        .json()
        .map_err(|e| PollError::Http(format!("Failed to parse error response: {e}")))?;

    match oauth_error.error.as_str() {
        "authorization_pending" => Err(PollError::Pending),
        "slow_down" => Err(PollError::SlowDown),
        "access_denied" => Err(PollError::AccessDenied),
        "expired_token" => Err(PollError::ExpiredToken),
        other => Err(PollError::Http(format!("Unexpected error: {other}"))),
    }
}

pub fn check_health(host: &str) -> bool {
    let url = format!("{host}/up");
    let client = Client::new();
    client
        .get(&url)
        .send()
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

pub fn verify_token(host: &str, token: &str) -> Result<bool, Error> {
    let url = format!("{host}/graphql");
    let body = serde_json::json!({ "query": "{ __typename }" });

    let client = Client::new();
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .map_err(|e| Error::new(format!("Network error: {e}")))?;

    if resp.status() == 401 || resp.status() == 422 {
        return Ok(false);
    }

    Ok(true)
}

pub fn search(host: &str, token: &str, query: &str, page: u32) -> Result<SearchResponse, Error> {
    let url = format!("{host}/graphql");
    let gql_query = format!(
        r#"{{ searchMulti(query: "{query}", page: {page}) {{ page totalPages totalResults results {{ id mediaType displayTitle title name posterPath backdropPath releaseDate firstAirDate overview voteAverage popularity adult voteCount originalLanguage originalTitle originalName genreIds }} }} }}"#,
        query = query.replace('"', "\\\""),
        page = page,
    );

    let body = serde_json::json!({ "query": gql_query });

    let client = Client::new();
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .map_err(|e| Error::new(format!("GraphQL request failed: {e}")))?;

    if resp.status() == 401 || resp.status() == 422 {
        return Err(Error::unauthorized());
    }

    if !resp.status().is_success() {
        let status = resp.status();
        return Err(Error::new(format!("GraphQL server error: {status}")));
    }

    #[derive(Deserialize)]
    struct GqlResponseWrapper {
        data: GqlSearchResponse,
    }

    let wrapper: GqlResponseWrapper = resp
        .json()
        .map_err(|e| Error::new(format!("Failed to parse GraphQL response: {e}")))?;

    Ok(SearchResponse::from(wrapper.data.search_multi))
}

pub fn find_movie(
    host: &str,
    token: &str,
    id: u32,
) -> Result<(MovieResponse, bool), Error> {
    let url = format!("{host}/graphql");
    let gql_query = format!(
        r#"{{ movie(id: {id}) {{ adult backdropPath genres {{ id name }} homepage id imdbId originCountry originalLanguage originalTitle overview popularity posterPath releaseDate revenue runtime title videoBlobs {{ id }} }} }}"#,
    );

    let body = serde_json::json!({ "query": gql_query });
    let client = Client::new();
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .map_err(|e| Error::new(format!("GraphQL request failed: {e}")))?;

    if resp.status() == 401 || resp.status() == 422 {
        return Err(Error::unauthorized());
    }
    if !resp.status().is_success() {
        let status = resp.status();
        return Err(Error::new(format!("GraphQL server error: {status}")));
    }

    #[derive(Deserialize)]
    struct Wrapper {
        data: GqlMovieResponse,
    }

    let wrapper: Wrapper = resp
        .json()
        .map_err(|e| Error::new(format!("Failed to parse movie response: {e}")))?;

    let gql_movie = wrapper.data.movie;
    let exists = !gql_movie.video_blobs.is_empty();
    Ok((MovieResponse::from(gql_movie), exists))
}

pub fn find_tv(host: &str, token: &str, id: u32) -> Result<TvResponse, Error> {
    let url = format!("{host}/graphql");
    let gql_query = format!(
        r#"{{ tv(id: {id}) {{ adult backdropPath episodeRunTime firstAirDate genres {{ id name }} homepage id inProduction languages lastAirDate name numberOfEpisodes numberOfSeasons originCountry originalLanguage originalName overview popularity posterPath seasons {{ airDate episodeCount id name overview posterPath seasonNumber voteAverage }} showType status tagline voteAverage voteCount }} }}"#,
    );

    let body = serde_json::json!({ "query": gql_query });
    let client = Client::new();
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .map_err(|e| Error::new(format!("GraphQL request failed: {e}")))?;

    if resp.status() == 401 || resp.status() == 422 {
        return Err(Error::unauthorized());
    }
    if !resp.status().is_success() {
        let status = resp.status();
        return Err(Error::new(format!("GraphQL server error: {status}")));
    }

    #[derive(Deserialize)]
    struct Wrapper {
        data: GqlTvResponse,
    }

    let wrapper: Wrapper = resp
        .json()
        .map_err(|e| Error::new(format!("Failed to parse tv response: {e}")))?;

    Ok(TvResponse::from(wrapper.data.tv))
}

pub fn find_season(
    host: &str,
    token: &str,
    tv_id: u32,
    season_number: u32,
) -> Result<(SeasonResponse, HashSet<u32>), Error> {
    let url = format!("{host}/graphql");
    let gql_query = format!(
        r#"{{ season(tvId: {tv_id}, seasonNumber: {season_number}) {{ airDate episodes {{ airDate episodeNumber episodeType id name overview productionCode runtime seasonNumber showId stillPath videoBlobs {{ id }} voteAverage voteCount }} id name overview posterPath seasonNumber voteAverage }} }}"#,
    );

    let body = serde_json::json!({ "query": gql_query });
    let client = Client::new();
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .map_err(|e| Error::new(format!("GraphQL request failed: {e}")))?;

    if resp.status() == 401 || resp.status() == 422 {
        return Err(Error::unauthorized());
    }
    if !resp.status().is_success() {
        let status = resp.status();
        return Err(Error::new(format!("GraphQL server error: {status}")));
    }

    #[derive(Deserialize)]
    struct Wrapper {
        data: GqlSeasonResponse,
    }

    let wrapper: Wrapper = resp
        .json()
        .map_err(|e| Error::new(format!("Failed to parse season response: {e}")))?;

    let gql_season = wrapper.data.season;
    let ripped_episodes: HashSet<u32> = gql_season
        .episodes
        .iter()
        .filter(|e| !e.video_blobs.is_empty())
        .map(|e| e.episode_number)
        .collect();

    Ok((SeasonResponse::from(gql_season), ripped_episodes))
}
