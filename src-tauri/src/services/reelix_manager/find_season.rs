//! Find a season by TV show ID and season number using the Reelix Manager API.
//!
//! GraphQL query: `season(tvId: Int, seasonNumber: Int)`

use serde::Deserialize;

use super::error::Error;
use super::ReelixManager;
use super::types::SeasonResponse;

/// Execute a season lookup by TV show ID and season number
pub fn execute(
    manager: &ReelixManager,
    tv_id: u32,
    season_number: u32,
) -> Result<SeasonResponse, Error> {
    const GQL_QUERY: &str = r#"query($tvId: Int!, $seasonNumber: Int!) { season(tvId: $tvId, seasonNumber: $seasonNumber) { episodes { airDate episodeNumber id name overview runtime seasonNumber showId stillPath voteAverage videoBlobs { id } } name posterPath seasonNumber } }"#;

    let body = serde_json::json!({
        "query": GQL_QUERY,
        "variables": {
            "tvId": tv_id,
            "seasonNumber": season_number,
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
        season: SeasonResponse,
    }

    let wrapper: Wrapper = resp.parse_json()?;
    Ok(wrapper.data.season)
}
