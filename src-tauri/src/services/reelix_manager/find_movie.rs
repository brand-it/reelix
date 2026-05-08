//! Find a movie by ID using the Reelix Manager API.
//!
//! GraphQL query: `movie(id: Int)`

use serde::Deserialize;

use super::error::Error;
use super::ReelixManager;
use super::types::MovieResponse;

/// Execute a movie lookup by ID
pub fn execute(manager: &ReelixManager, id: u32) -> Result<MovieResponse, Error> {
    const GQL_QUERY: &str = r#"query($id: Int!) { movie(id: $id) { genres { id name } id overview posterPath releaseDate runtime title videoBlobs { id } } }"#;

    let body = serde_json::json!({
        "query": GQL_QUERY,
        "variables": {
            "id": id,
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
    struct Data {
        movie: MovieResponse,
    }

    let wrapper: Wrapper = resp.parse_json()?;
    Ok(wrapper.data.movie)
}
