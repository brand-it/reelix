//! Search for movies and TV shows using the Reelix Manager API.
//!
//! GraphQL query: `searchMulti(query: String, page: Int)`

use serde::Deserialize;

use super::error::Error;
use super::ReelixManager;
use crate::the_movie_db::models::{GqlSearchResponse, SearchResponse};

/// Execute a search query for movies and TV shows
pub fn execute(manager: &ReelixManager, query: &str, page: u32) -> Result<SearchResponse, Error> {
    let url = format!("{}/graphql", manager.host);

    const GQL_QUERY: &str = r#"{{ searchMulti(query: $query, page: $page) {{ page totalPages totalResults results {{ id mediaType displayTitle title name posterPath backdropPath releaseDate firstAirDate overview voteAverage popularity adult voteCount originalLanguage originalTitle originalName genreIds }} }} }}"#;

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
    struct GqlResponseWrapper {
        data: GqlSearchResponse,
    }

    let wrapper: GqlResponseWrapper = resp
        .json()
        .map_err(|e| Error::new(format!("Failed to parse GraphQL response: {e}")))?;

    Ok(SearchResponse::from(wrapper.data.search_multi))
}
