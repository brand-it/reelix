use crate::services::converter;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
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
    pub revenue: i32,
    pub runtime: i32,
    pub title: String,
}

#[derive(Serialize, Debug)]
pub struct MovieView {
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
    pub revenue: i32,
    pub runtime: i32,
    pub title_year: String,
    pub title: String,
    pub year: Option<u32>,
}

impl From<MovieResponse> for MovieView {
    fn from(movie: MovieResponse) -> Self {
        let year = movie.year(); // your existing logic
        let title_year = movie.title_year(); // also your logic

        MovieView {
            adult: movie.adult,
            backdrop_path: movie.backdrop_path,
            genres: movie.genres,
            homepage: movie.homepage,
            id: movie.id,
            imdb_id: movie.imdb_id,
            origin_country: movie.origin_country,
            original_language: movie.original_language,
            original_title: movie.original_title,
            overview: movie.overview,
            popularity: movie.popularity,
            poster_path: movie.poster_path,
            release_date: movie.release_date,
            revenue: movie.revenue,
            runtime: movie.runtime,
            title_year,
            title: movie.title,
            year,
        }
    }
}

impl MovieResponse {
    pub fn year(&self) -> Option<u32> {
        self.release_date.as_ref().and_then(|date_str| {
            NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .ok()
                .and_then(|dt| dt.format("%Y").to_string().parse::<u32>().ok())
        })
    }

    pub fn title_year(&self) -> String {
        match self.year() {
            Some(v) => return format!("{} ({})", self.title, v.to_string()),
            None => return format!("{}", self.title),
        };
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MovieGenre {
    pub id: u32,
    pub name: String,
}

// Struct to represent the full response
#[derive(Serialize, Deserialize)]
pub struct SearchResponse {
    page: u32,
    results: Vec<SearchResult>,
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
    id: u32,
    media_type: String,
    #[serde(default)]
    original_language: String,
    original_title: Option<String>,
    #[serde(default)]
    overview: String,
    popularity: f64,
    profile_path: Option<String>,
    poster_path: Option<String>,
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

#[derive(Serialize, Deserialize)]
pub struct MovieReleaseDatesResponse {
    pub id: u32,
    pub results: Vec<CountryReleaseDates>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CountryReleaseDates {
    pub iso_3166_1: String,
    pub release_dates: Vec<ReleaseDate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReleaseDate {
    pub certification: String,
    pub descriptors: Vec<String>,
    pub iso_639_1: String,
    pub note: String,
    pub release_date: String,
    #[serde(rename = "type")]
    pub release_type: u32,
}
