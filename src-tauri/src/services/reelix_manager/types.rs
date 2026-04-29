use chrono::NaiveDate;
use humantime::format_duration;
use serde::{Deserialize, Serialize};
use std::time::Duration;

// -------------------------
// -------- Common ---------
// -------------------------

#[derive(Serialize, Deserialize, Clone)]
pub struct Genre {
    pub id: u32,
    pub name: String,
}

#[derive(Deserialize, Clone)]
pub struct GqlVideoBlob {}

// -------------------------
// -------- Movies ---------
// -------------------------

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MovieResponse {
    pub genres: Vec<Genre>,
    pub id: u32,
    pub overview: String,
    pub poster_path: Option<String>,
    pub release_date: Option<String>,
    #[serde(default)]
    pub runtime: u64,
    pub title: String,
    #[serde(default, skip_serializing)]
    pub video_blobs: Vec<GqlVideoBlob>,
}

impl MovieResponse {
    /// Margin for runtime matching, in seconds.
    const MOVIE_RUNTIME_MARGIN: u64 = 600; // seconds (10 minutes)

    pub fn year(&self) -> Option<u32> {
        self.release_date.as_ref().and_then(|date_str| {
            NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .ok()
                .and_then(|dt| dt.format("%Y").to_string().parse::<u32>().ok())
        })
    }

    pub fn title_year(&self) -> String {
        match self.year() {
            Some(v) => format!("{} ({})", self.title, v),
            None => self.title.to_string(),
        }
    }

    pub fn runtime_seconds(&self) -> u64 {
        self.runtime * 60
    }

    /// Returns a range of acceptable runtimes for this movie, centered on the movie's runtime.
    ///
    /// The range is calculated as (runtime - margin) to (runtime + margin), where margin is a constant (600 seconds = 10 minutes).
    /// The returned range is in **seconds**.
    /// This is useful for matching disk titles whose duration is close to the expected runtime of the movie.
    pub fn runtime_range(&self) -> std::ops::Range<u64> {
        self.runtime_seconds()
            .saturating_sub(Self::MOVIE_RUNTIME_MARGIN)
            ..self.runtime_seconds() + Self::MOVIE_RUNTIME_MARGIN
    }

    pub fn human_runtime(&self) -> String {
        let duration = Duration::from_secs(self.runtime_seconds());
        format!("{}", format_duration(duration))
    }
}

// -------------------------
// -------- Search ---------
// -------------------------

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub page: u32,
    pub results: Vec<SearchResult>,
    pub total_pages: u32,
    pub total_results: u32,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    #[serde(default)]
    pub first_air_date: Option<String>,
    pub id: u32,
    #[serde(default)]
    pub media_type: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub poster_path: Option<String>,
    #[serde(default)]
    pub release_date: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
}

impl SearchResult {
    pub fn get_title(&self) -> String {
        self.title
            .clone()
            .or_else(|| Some(self.name.clone()))
            .unwrap_or_else(|| "Unknown".to_string())
    }

    pub fn get_date(&self) -> String {
        self.release_date
            .clone()
            .or_else(|| self.first_air_date.clone())
            .map(|date| {
                if date.len() >= 4 {
                    date[..4].to_string()
                } else {
                    "N/A".to_string()
                }
            })
            .unwrap_or_else(|| "N/A".to_string())
    }
}

// -------------------------
// ---------- TV -----------
// -------------------------

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Copy, PartialOrd, Ord, Debug)]
pub struct TvId(u32);

impl std::fmt::Display for TvId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<TvId> for u32 {
    fn from(id: TvId) -> Self {
        id.0
    }
}

// From unsigned types
impl From<u8> for TvId {
    fn from(id: u8) -> Self {
        TvId(id as u32)
    }
}

impl From<u16> for TvId {
    fn from(id: u16) -> Self {
        TvId(id as u32)
    }
}

impl From<u32> for TvId {
    fn from(id: u32) -> Self {
        TvId(id)
    }
}

impl From<u64> for TvId {
    fn from(id: u64) -> Self {
        TvId(id as u32)
    }
}

impl From<u128> for TvId {
    fn from(id: u128) -> Self {
        TvId(id as u32)
    }
}

impl From<usize> for TvId {
    fn from(id: usize) -> Self {
        TvId(id as u32)
    }
}

// From signed types
impl From<i8> for TvId {
    fn from(id: i8) -> Self {
        TvId(id as u32)
    }
}

impl From<i16> for TvId {
    fn from(id: i16) -> Self {
        TvId(id as u32)
    }
}

impl From<i32> for TvId {
    fn from(id: i32) -> Self {
        TvId(id as u32)
    }
}

impl From<i64> for TvId {
    fn from(id: i64) -> Self {
        TvId(id as u32)
    }
}

impl From<i128> for TvId {
    fn from(id: i128) -> Self {
        TvId(id as u32)
    }
}

impl From<isize> for TvId {
    fn from(id: isize) -> Self {
        TvId(id as u32)
    }
}

impl TryFrom<&str> for TvId {
    type Error = std::num::ParseIntError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let parsed = s.parse::<u32>()?;
        Ok(TvId(parsed))
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TvResponse {
    pub episode_run_time: Vec<u32>,
    pub first_air_date: Option<String>,
    pub genres: Vec<Genre>,
    pub id: TvId,
    pub name: String,
    pub overview: String,
    pub poster_path: Option<String>,
    pub seasons: Vec<TvSeason>,
    #[serde(rename = "showType")]
    pub show_type: String,
}

impl TvResponse {
    pub fn year(&self) -> Option<u32> {
        self.first_air_date.as_ref().and_then(|date_str| {
            NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .ok()
                .and_then(|dt| dt.format("%Y").to_string().parse::<u32>().ok())
        })
    }

    pub fn title_year(&self) -> String {
        match self.year() {
            Some(v) => format!("{} ({})", self.name, v),
            None => self.name.to_string(),
        }
    }

    pub fn average_episode_run_time(&self) -> String {
        if self.episode_run_time.is_empty() {
            return "N/A".to_string();
        }
        let total: u32 = self.episode_run_time.iter().sum();
        let average = total as f64 / self.episode_run_time.len() as f64;
        let duration = Duration::from_secs((average * 60.0) as u64);
        format!("{}", format_duration(duration))
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TvSeason {
    pub name: String,
    pub poster_path: Option<String>,
    pub season_number: u32,
}

// ------------------------------------
// ------- TV Season Response ---------
// ------------------------------------

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SeasonResponse {
    pub episodes: Vec<SeasonEpisode>,
    pub name: String,
    pub poster_path: Option<String>,
    pub season_number: u32,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SeasonEpisode {
    #[serde(default)]
    pub air_date: Option<String>,
    pub episode_number: u32,
    pub id: u32,
    pub name: String,
    pub overview: String,
    #[serde(default)]
    pub runtime: Option<u32>,
    pub season_number: u32,
    pub show_id: u32,
    #[serde(default)]
    pub still_path: Option<String>,
    pub vote_average: f64,
    #[serde(default, skip_serializing)]
    pub video_blobs: Vec<GqlVideoBlob>,
}

impl SeasonEpisode {
    /// Margin for runtime matching, in seconds.
    const EPISODE_RUNTIME_MARGIN: u64 = 600; // seconds (10 minutes)

    /// Returns a range of acceptable runtimes for this episode, centered on the episode's runtime.
    ///
    /// The range is calculated as (runtime - margin) to (runtime + margin), where margin is a constant (600 seconds = 10 minutes).
    /// The returned range is in **seconds**.
    /// This is useful for matching disk titles whose duration is close to the expected runtime of the episode.
    pub fn runtime_range(&self) -> std::ops::Range<u64> {
        let runtime = self.runtime_seconds();
        runtime.saturating_sub(Self::EPISODE_RUNTIME_MARGIN)..runtime + Self::EPISODE_RUNTIME_MARGIN
    }

    pub fn runtime_seconds(&self) -> u64 {
        self.runtime.map(|r| r as u64 * 60).unwrap_or(0)
    }

    pub fn formatted_vote_average(&self) -> String {
        let average = (self.vote_average * 10.0).round();
        format!("{average}")
    }

    pub fn formatted_air_date(&self) -> String {
        self.air_date
            .as_ref()
            .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
            .map(|date| date.format("%B %-d, %Y").to_string())
            .unwrap_or_default()
    }

    pub fn formatted_runtime(&self) -> String {
        let minutes = self.runtime.unwrap_or(0);
        let hours = minutes / 60;
        if hours > 0 {
            format!("{hours}h {}m", minutes % 60)
        } else {
            format!("{minutes}m")
        }
    }
}
