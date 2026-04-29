//! Search for movies and TV shows using the Reelix Manager API.
//!
//! GraphQL query: `searchMulti(query: String, page: Int)`

use serde::Deserialize;

use super::error::Error;
use super::ReelixManager;
use super::types::SearchResponse;

/// Execute a search query for movies and TV shows
pub fn execute(manager: &ReelixManager, query: &str, page: u32) -> Result<SearchResponse, Error> {
    let url = format!("{}/graphql", manager.host);
    if query.trim().is_empty() {
        return Ok(SearchResponse::default());
    }

    const GQL_QUERY: &str = r#"{ searchMulti(query: $query, page: $page) { page results { firstAirDate id mediaType name posterPath releaseDate title } totalPages totalResults } }"#;

    let body = serde_json::json!({
        "query": GQL_QUERY,
        "variables": {
            "query": query,
            "page": page,
        },
    });
    let resp = manager
        .client
        .post(&url)
        .header("Authorization", format!("Bearer {}", manager.token))
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
        data: Data,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Data {
        search_multi: SearchResponse,
    }

    let wrapper: Wrapper = resp
        .json()
        .map_err(|e| Error::new(format!("Failed to parse GraphQL response: {e}")))?;

    Ok(wrapper.data.search_multi)
}
