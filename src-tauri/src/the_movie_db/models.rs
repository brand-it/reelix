use chrono::NaiveDate;
use humantime::format_duration;
use serde::{Deserialize, Serialize};
use std::time::Duration;

// -------------------------
// -------- Movies ---------
// -------------------------

#[derive(Serialize, Deserialize, Clone)]
pub struct MovieResponse {
    pub adult: bool,
    pub backdrop_path: Option<String>,
    pub genres: Vec<MovieGenre>,
    pub homepage: String,
    pub id: u32,
    pub imdb_id: String,
    pub origin_country: Vec<String>,
    pub original_language: String,
    pub original_title: String,
    pub overview: String,
    pub popularity: f32,
    pub poster_path: Option<String>,
    pub release_date: Option<String>,
    pub revenue: u64,
    pub runtime: u64,
    pub title: String,
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

    // returns a basic file path for example Alien (1979)/Alien (1979).mkv
    pub fn to_file_path(&self) -> String {
        format!("{}/{}.mkv", self.title_year(), self.title_year())
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MovieGenre {
    pub id: u32,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct MovieReleaseDatesResponse {
    pub id: u32,
    pub results: Vec<CountryReleaseDates>,
}

#[derive(Serialize, Deserialize)]
pub struct CountryReleaseDates {
    pub iso_3166_1: String,
    pub release_dates: Vec<ReleaseDate>,
}

#[derive(Serialize, Deserialize)]
pub struct ReleaseDate {
    pub certification: String,
    pub descriptors: Vec<String>,
    pub iso_639_1: String,
    pub note: String,
    pub release_date: String,
    #[serde(rename = "type")]
    pub release_type: u32,
}

// -------------------------
// -------- Search ---------
// -------------------------

// Struct to represent the full response
#[derive(Serialize, Deserialize)]
pub struct SearchResponse {
    page: u32,
    pub results: Vec<SearchResult>,
    total_pages: u32,
    total_results: u32,
}

#[derive(Serialize, Deserialize)]
pub struct SearchResult {
    #[serde(default)]
    name: String,
    #[serde(default)]
    original_name: String,
    adult: bool,
    backdrop_path: Option<String>,
    #[serde(default)]
    genre_ids: Vec<u32>,
    pub id: u32,
    #[serde(default)]
    pub media_type: String,
    #[serde(default)]
    original_language: String,
    original_title: Option<String>,
    #[serde(default)]
    overview: String,
    #[serde(default)]
    popularity: Option<f64>,
    profile_path: Option<String>,
    pub poster_path: Option<String>,
    release_date: Option<String>,
    first_air_date: Option<String>,
    title: Option<String>,
    #[serde(default)]
    video: bool,
    #[serde(default)]
    vote_average: f64,
    #[serde(default)]
    vote_count: u32,
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

#[derive(Serialize, Deserialize, Clone)]
pub struct TvResponse {
    pub adult: bool,
    pub backdrop_path: Option<String>,
    pub created_by: Vec<TvCreatedBy>,
    pub episode_run_time: Vec<u32>,
    pub first_air_date: Option<String>,
    pub genres: Vec<TvGenre>,
    pub homepage: Option<String>,
    pub id: u32,
    pub in_production: bool,
    pub languages: Vec<String>,
    pub last_air_date: Option<String>,
    pub last_episode_to_air: Option<TvEpisode>,
    pub name: String,
    pub networks: Vec<TvNetwork>,
    pub next_episode_to_air: Option<TvEpisode>,
    pub number_of_episodes: u32,
    pub number_of_seasons: u32,
    pub origin_country: Vec<String>,
    pub original_language: String,
    pub original_name: String,
    pub overview: String,
    pub popularity: f64,
    pub poster_path: Option<String>,
    pub production_companies: Vec<TvProductionCompany>,
    pub production_countries: Vec<TvProductionCountry>,
    pub seasons: Vec<TvSeason>,
    pub spoken_languages: Vec<TvSpokenLanguage>,
    pub status: String,
    pub tagline: String,
    // Since "type" is a reserved word in Rust, we rename the field to "type_"
    #[serde(rename = "type")]
    pub type_: String,
    pub vote_average: f64,
    pub vote_count: u32,
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
pub struct TvCreatedBy {
    pub id: u32,
    pub credit_id: String,
    pub name: String,
    pub original_name: String,
    pub gender: u8,
    pub profile_path: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TvGenre {
    pub id: u32,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TvEpisode {
    pub id: u32,
    pub name: String,
    pub overview: String,
    pub vote_average: f64,
    pub vote_count: u32,
    pub air_date: String,
    pub episode_number: u32,
    pub episode_type: String,
    pub production_code: String,
    pub runtime: u32,
    pub season_number: u32,
    pub show_id: u32,
    pub still_path: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TvNetwork {
    pub id: u32,
    pub logo_path: Option<String>,
    pub name: String,
    pub origin_country: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TvProductionCompany {
    pub id: u32,
    pub logo_path: Option<String>,
    pub name: String,
    pub origin_country: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TvProductionCountry {
    pub iso_3166_1: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TvSeason {
    pub air_date: Option<String>,
    pub episode_count: u32,
    pub id: u32,
    pub name: String,
    pub overview: String,
    pub poster_path: Option<String>,
    pub season_number: u32,
    pub vote_average: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TvSpokenLanguage {
    pub english_name: String,
    pub iso_639_1: String,
    pub name: String,
}

// ------------------------------------
// ------- TV Season Response ---------
// ------------------------------------

#[derive(Serialize, Deserialize, Clone)]
pub struct SeasonResponse {
    pub _id: String,
    pub air_date: Option<String>,
    pub episodes: Vec<SeasonEpisode>,
    pub name: String,
    pub overview: String,
    pub id: u32,
    pub poster_path: Option<String>,
    pub season_number: u32,
    pub vote_average: f32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SeasonEpisode {
    pub air_date: Option<String>,
    pub episode_number: u32,
    pub episode_type: String,
    pub id: u32,
    pub name: String,
    pub overview: String,
    pub production_code: Option<String>,
    pub runtime: Option<u32>,
    pub season_number: u32,
    pub show_id: u32,
    pub still_path: Option<String>,
    pub vote_average: f32,
    pub vote_count: u32,
    pub crew: Vec<SeasonCrewMember>,
    pub guest_stars: Vec<SeasonGuestStar>,
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

#[derive(Serialize, Deserialize, Clone)]
pub struct SeasonCrewMember {
    pub job: String,
    pub department: String,
    pub credit_id: String,
    pub adult: bool,
    pub gender: Option<u8>,
    pub id: u32,
    pub known_for_department: String,
    pub name: String,
    pub original_name: String,
    pub popularity: f32,
    pub profile_path: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SeasonGuestStar {
    pub character: String,
    pub credit_id: String,
    pub order: u32,
    pub adult: bool,
    pub gender: Option<u8>,
    pub id: u32,
    pub known_for_department: String,
    pub name: String,
    pub original_name: String,
    pub popularity: f32,
    pub profile_path: Option<String>,
}
