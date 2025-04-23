use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

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
    pub runtime: u32,
    pub title: String,
}

#[derive(Serialize)]
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
    pub revenue: u64,
    pub runtime: u32,
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

#[derive(Serialize, Deserialize, Clone)]
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
            Some(v) => return format!("{} ({})", self.name, v.to_string()),
            None => return format!("{}", self.name),
        };
    }

    // pub fn find_season(&self, id: u32) -> Option<TvSeason> {
    //     self.seasons.iter().find(|s| s.id == id).cloned()
    // }
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

// Example conversion to a simpler view struct
#[derive(Serialize)]
pub struct TvView {
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
    pub title_year: String,
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
    pub vote_average: f64,
    pub vote_count: u32,
    pub year: Option<u32>,
}

impl From<TvResponse> for TvView {
    fn from(tv: TvResponse) -> Self {
        let year = tv.year();
        let title_year = tv.title_year();
        TvView {
            adult: tv.adult,
            backdrop_path: tv.backdrop_path,
            created_by: tv.created_by,
            episode_run_time: tv.episode_run_time,
            first_air_date: tv.first_air_date,
            genres: tv.genres,
            homepage: tv.homepage,
            id: tv.id,
            in_production: tv.in_production,
            languages: tv.languages,
            last_air_date: tv.last_air_date,
            last_episode_to_air: tv.last_episode_to_air,
            name: tv.name,
            networks: tv.networks,
            next_episode_to_air: tv.next_episode_to_air,
            number_of_episodes: tv.number_of_episodes,
            number_of_seasons: tv.number_of_seasons,
            origin_country: tv.origin_country,
            original_language: tv.original_language,
            original_name: tv.original_name,
            overview: tv.overview,
            popularity: tv.popularity,
            poster_path: tv.poster_path,
            production_companies: tv.production_companies,
            production_countries: tv.production_countries,
            seasons: tv.seasons,
            spoken_languages: tv.spoken_languages,
            status: tv.status,
            tagline: tv.tagline,
            title_year,
            vote_average: tv.vote_average,
            vote_count: tv.vote_count,
            year: year,
        }
    }
}

// ------------------------------------
// ------- TV Season Response ---------
// ------------------------------------
#[derive(Serialize, Deserialize, Clone)]
pub struct SeasonResponse {
    pub _id: String,
    pub air_date: String,
    pub episodes: Vec<SeasonEpisode>,
    pub name: String,
    pub overview: String,
    pub id: u32,
    pub poster_path: String,
    pub season_number: u32,
    pub vote_average: f32,
}

impl SeasonResponse {
    // pub fn title_year(&self) -> String {
    //     match self.year() {
    //         Some(v) => return format!("{} ({})", self.name, v.to_string()),
    //         None => return format!("{}", self.name),
    //     };
    // }

    // pub fn year(&self) -> Option<u32> {
    //     NaiveDate::parse_from_str(&self.air_date, "%Y-%m-%d")
    //         .ok()
    //         .and_then(|dt| dt.format("%Y").to_string().parse::<u32>().ok())
    // }

    // pub fn formatted_air_date(&self) -> String {
    //     NaiveDate::parse_from_str(&self.air_date, "%Y-%m-%d")
    //         .ok()
    //         .map(|date| date.format("%B %-d, %Y").to_string())
    //         .unwrap_or_else(|| "".to_string())
    // }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SeasonEpisode {
    pub air_date: String,
    pub episode_number: u32,
    pub episode_type: String,
    pub id: u32,
    pub name: String,
    pub overview: String,
    pub production_code: String,
    pub runtime: u32,
    pub season_number: u32,
    pub show_id: u32,
    pub still_path: String,
    pub vote_average: f32,
    pub vote_count: u32,
    pub crew: Vec<SeasonCrewMember>,
    pub guest_stars: Vec<SeasonGuestStar>,
}

impl SeasonEpisode {
    pub fn year(&self) -> Option<u32> {
        NaiveDate::parse_from_str(&self.air_date, "%Y-%m-%d")
            .ok()
            .and_then(|dt| dt.format("%Y").to_string().parse::<u32>().ok())
    }

    pub fn formatted_air_date(&self) -> String {
        NaiveDate::parse_from_str(&self.air_date, "%Y-%m-%d")
            .ok()
            .map(|date| date.format("%B %-d, %Y").to_string())
            .unwrap_or_else(|| "".to_string())
    }

    pub fn formatted_runtime(&self) -> String {
        let hours = self.runtime / 60;
        let minutes = self.runtime % 60;

        if hours > 0 {
            format!("{}h&nbsp;{}m", hours, minutes)
        } else {
            format!("{}m", minutes)
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

#[derive(Serialize, Deserialize)]
pub struct SeasonView {
    pub _id: String,
    pub air_date: String,
    pub episodes: Vec<SeasonEpisodeView>,
    pub name: String,
    pub overview: String,
    pub id: u32,
    pub poster_path: String,
    pub season_number: u32,
    pub vote_average: f32,
}

// The view type you will expose, now with an extra computed field.
#[derive(Serialize, Deserialize)]
pub struct SeasonEpisodeView {
    pub air_date: String,
    pub year: Option<u32>,
    pub formatted_air_date: String,
    pub formatted_runtime: String,
    pub episode_number: u32,
    pub episode_type: String,
    pub id: u32,
    pub name: String,
    pub overview: String,
    pub production_code: String,
    pub runtime: u32,
    pub season_number: u32,
    pub show_id: u32,
    pub still_path: String,
    pub vote_average: f32,
    pub vote_count: u32,
    // Computed field: converts the vote average to a percentage string.
    pub vote_average_percentage: String,
    pub crew: Vec<SeasonCrewMember>,
    pub guest_stars: Vec<SeasonGuestStar>,
}

impl From<SeasonEpisode> for SeasonEpisodeView {
    fn from(episode: SeasonEpisode) -> Self {
        let formatted_air_date = episode.formatted_air_date();
        let formatted_runtime = episode.formatted_runtime();
        let year = episode.year();
        SeasonEpisodeView {
            air_date: episode.air_date,
            formatted_air_date: formatted_air_date,
            formatted_runtime: formatted_runtime,
            year: year,
            episode_number: episode.episode_number,
            episode_type: episode.episode_type,
            id: episode.id,
            name: episode.name,
            overview: episode.overview,
            production_code: episode.production_code,
            runtime: episode.runtime,
            season_number: episode.season_number,
            show_id: episode.show_id,
            still_path: episode.still_path,
            vote_average: episode.vote_average,
            vote_count: episode.vote_count,
            // Multiply by 10 and format to 1 decimal place, then append a percent sign.
            vote_average_percentage: format!("{:.1}%", episode.vote_average * 10.0),
            crew: episode.crew,
            guest_stars: episode.guest_stars,
        }
    }
}

impl From<SeasonResponse> for SeasonView {
    fn from(season: SeasonResponse) -> Self {
        SeasonView {
            _id: season._id,
            air_date: season.air_date,
            // Convert each SeasonEpisode into a SeasonEpisodeView
            episodes: season
                .episodes
                .into_iter()
                .map(SeasonEpisodeView::from)
                .collect(),
            name: season.name,
            overview: season.overview,
            id: season.id,
            poster_path: season.poster_path,
            season_number: season.season_number,
            vote_average: season.vote_average,
        }
    }
}
