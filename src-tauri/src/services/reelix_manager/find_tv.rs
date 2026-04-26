//! Find a TV show by ID using the Reelix Manager API.
//!
//! GraphQL query: `tv(id: Int)`

use serde::Deserialize;

use super::error::Error;
use super::ReelixManager;
use crate::the_movie_db::models::GqlTvResponse;
use crate::the_movie_db::TvResponse;

/// Execute a TV show lookup by ID
pub fn execute(manager: &ReelixManager, id: u32) -> Result<TvResponse, Error> {
    let url = format!("{}/graphql", manager.host);

    const GQL_QUERY: &str = r#"{{ tv(id: $id) {{ adult backdropPath episodeRunTime firstAirDate genres {{ id name }} homepage id inProduction languages lastAirDate name numberOfEpisodes numberOfSeasons originCountry originalLanguage originalName overview popularity posterPath seasons {{ airDate episodeCount id name overview posterPath seasonNumber voteAverage }} showType status tagline voteAverage voteCount }} }}"#;

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
        data: GqlTvResponse,
    }

    let wrapper: Wrapper = resp
        .json()
        .map_err(|e| Error::new(format!("Failed to parse tv response: {e}")))?;

    Ok(TvResponse::from(wrapper.data.tv))
}
