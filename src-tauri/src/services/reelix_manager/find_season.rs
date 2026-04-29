//! Find a season by TV show ID and season number using the Reelix Manager API.
//!
//! GraphQL query: `season(tvId: Int, seasonNumber: Int)`

use serde::Deserialize;
use std::collections::HashSet;

use super::error::Error;
use super::ReelixManager;
use super::types::SeasonResponse;

/// Execute a season lookup by TV show ID and season number
///
/// Returns the season data and a set of episode numbers that have already
/// been ripped (have video blobs)
pub fn execute(
    manager: &ReelixManager,
    tv_id: u32,
    season_number: u32,
) -> Result<(SeasonResponse, HashSet<u32>), Error> {
    let url = format!("{}/graphql", manager.host);

    const GQL_QUERY: &str = r#"{ season(tvId: $tvId, seasonNumber: $seasonNumber) { episodes { airDate episodeNumber id name overview runtime seasonNumber showId stillPath voteAverage videoBlobs { id } } name posterPath seasonNumber } }"#;

    let body = serde_json::json!({
        "query": GQL_QUERY,
        "variables": {
            "tvId": tv_id,
            "seasonNumber": season_number,
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
        season: SeasonResponse,
    }

    let wrapper: Wrapper = resp
        .json()
        .map_err(|e| Error::new(format!("Failed to parse season response: {e}")))?;

    let season = wrapper.data.season;
    let ripped_episodes: HashSet<u32> = season
        .episodes
        .iter()
        .filter(|e| !e.video_blobs.is_empty())
        .map(|e| e.episode_number)
        .collect();
    Ok((season, ripped_episodes))
}
