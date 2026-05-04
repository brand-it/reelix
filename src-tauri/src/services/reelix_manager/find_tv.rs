//! Find a TV show by ID using the Reelix Manager API.
//!
//! GraphQL query: `tv(id: Int)`

use serde::Deserialize;

use super::error::Error;
use super::ReelixManager;
use super::types::TvResponse;

/// Execute a TV show lookup by ID
pub fn execute(manager: &ReelixManager, id: u32) -> Result<TvResponse, Error> {
    const GQL_QUERY: &str = r#"query($id: Int!) { tv(id: $id) { episodeRunTime firstAirDate genres { id name } id name overview posterPath seasons { name posterPath seasonNumber } showType } }"#;

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
        tv: TvResponse,
    }

    let wrapper: Wrapper = resp.parse_json()?;
    Ok(wrapper.data.tv)
}
