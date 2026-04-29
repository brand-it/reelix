//! Find a TV show by ID using the Reelix Manager API.
//!
//! GraphQL query: `tv(id: Int)`

use serde::Deserialize;

use super::error::Error;
use super::ReelixManager;
use super::types::TvResponse;

/// Execute a TV show lookup by ID
pub fn execute(manager: &ReelixManager, id: u32) -> Result<TvResponse, Error> {
    let url = format!("{}/graphql", manager.host);

    const GQL_QUERY: &str = r#"{ tv(id: $id) { episodeRunTime firstAirDate genres { id name } id name overview posterPath seasons { name posterPath seasonNumber } showType } }"#;

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
        tv: TvResponse,
    }

    let wrapper: Wrapper = resp
        .json()
        .map_err(|e| Error::new(format!("Failed to parse tv response: {e}")))?;

    Ok(wrapper.data.tv)
}
