//! Find a movie by ID using the Reelix Manager API.
//!
//! GraphQL query: `movie(id: Int)`

use serde::Deserialize;

use super::error::Error;
use super::ReelixManager;
use super::types::MovieResponse;

/// Execute a movie lookup by ID
///
/// Returns the movie data and a boolean indicating if it already exists
/// in the library (has video blobs)
pub fn execute(manager: &ReelixManager, id: u32) -> Result<(MovieResponse, bool), Error> {
    let url = format!("{}/graphql", manager.host);

    const GQL_QUERY: &str = r#"{ movie(id: $id) { genres { id name } id overview posterPath releaseDate runtime title videoBlobs { id } } }"#;

    let body = serde_json::json!({
        "query": GQL_QUERY,
        "variables": {
            "id": id,
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
    struct Data {
        movie: MovieResponse,
    }

    let wrapper: Wrapper = resp
        .json()
        .map_err(|e| Error::new(format!("Failed to parse movie response: {e}")))?;

    let movie = wrapper.data.movie;
    let exists = !movie.video_blobs.is_empty();
    Ok((movie, exists))
}
