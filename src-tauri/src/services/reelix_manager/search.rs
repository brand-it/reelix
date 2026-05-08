//! Search for movies and TV shows using the Reelix Manager API.
//!
//! GraphQL query: `searchMulti(query: String, page: Int)`

use serde::Deserialize;

use super::error::Error;
use super::ReelixManager;
use super::types::SearchResponse;

/// Execute a search query for movies and TV shows
pub fn execute(manager: &ReelixManager, query: &str, page: u32) -> Result<SearchResponse, Error> {
    if query.trim().is_empty() {
        return Ok(SearchResponse::default());
    }

    const GQL_QUERY: &str = r#"query($query: String!, $page: Int!) { searchMulti(query: $query, page: $page) { page results { firstAirDate id mediaType name posterPath releaseDate title } totalPages totalResults } }"#;

    let body = serde_json::json!({
        "query": GQL_QUERY,
        "variables": {
            "query": query,
            "page": page,
        },
    });

    let resp = manager
        .sync_request()
        .path("/graphql")
        .json(body)
        .send()?;

    #[derive(Deserialize)]
    struct Wrapper {
        data: Data,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Data {
        search_multi: SearchResponse,
    }

    let wrapper: Wrapper = resp.parse_json()?;
    Ok(wrapper.data.search_multi)
}
