use crate::{
    the_movie_db::{MovieResponse, SeasonEpisode, SeasonResponse, TvResponse},
    models::title_info::TitleInfo,
    state::AppState,
};
use serde::Serialize;
use std::{fs, path::PathBuf};

/// Wrapper for MovieResponse to support multipart and edition info for movies.
#[derive(Serialize, Clone)]
pub struct MoviePartEdition {
    pub movie: MovieResponse,
    pub part: Option<u16>,
    pub edition: Option<String>,
}

impl MoviePartEdition {
    /// Returns the runtime of this movie in seconds, if available.
    pub fn runtime_seconds(&self) -> u64 {
        self.movie.runtime_seconds()
    }

    /// Returns the runtime range of this movie (±5 minutes), used for matching titles.
    pub fn runtime_range(&self) -> std::ops::Range<u64> {
        self.movie.runtime_range()
    }
}

#[derive(Serialize, Clone)]
pub struct TitleVideo {
    pub title: TitleInfo,
    pub video: Video,
}

impl TitleVideo {
    // pub fn is_tv(&self) -> bool {
    //     matches!(self.video, Video::Tv(_))
    // }

    // pub fn is_movie(&self) -> bool {
    //     matches!(self.video, Video::Movie(_))
    // }

    /// Move the ripped video file to its final, Plex-compliant location and update the internal path.
    ///
    /// Purpose:
    /// - After a video is ripped (e.g., from disc), it may be placed in a temporary or generic location.
    /// - This function renames (moves) the ripped file to its correct, organized destination based on
    ///   Plex naming conventions for movies or TV episodes.
    /// - Ensures the file is discoverable by Plex and matches the expected library structure.
    /// - Updates the `ripped_file` field to reflect the new location.
    ///
    /// Why use it?
    /// - To automate the process of organizing ripped media for Plex or similar media servers.
    /// - To avoid manual file renaming and moving, reducing human error and ensuring consistency.
    /// - To prepare files for metadata scanning, artwork matching, and subtitle association.
    ///
    /// How it works:
    /// 1. Checks that a ripped file path is set and that the file exists.
    /// 2. Computes the target path using `video_path`, which generates the correct filename and directory.
    /// 3. Moves (renames) the file to the target location using `fs::rename`.
    /// 4. Updates the internal `ripped_file` field to the new path.
    /// 5. Returns the new path, or an error if the operation fails.
    ///
    /// Examples:
    /// - Ripped file: `/tmp/rip.mkv` for "Inception (2010)" ->
    ///   Moves to `/Movies/Inception (2010)/Inception (2010).mkv`
    /// - Ripped file: `/tmp/episode.mkv` for "Breaking Bad (2008)" S01E01 ->
    ///   Moves to `/TV Shows/Breaking Bad (2008)/Season 01/Breaking Bad (2008) - S01E01 - Pilot.mkv`
    ///
    /// Notes:
    /// - Will fail if the ripped file does not exist or cannot be moved (e.g., permissions).
    /// - Does not create parent directories; ensure they exist before calling.
    /// - Returns a `Result<PathBuf, String>` for error handling in calling code.
    pub fn rename_ripped_file(&self, app_state: &AppState) -> Result<PathBuf, String> {
        let target_path = self.video_path(app_state);
        let from_path = self.ripped_file_path(app_state);

        if !from_path.exists() {
            return Err(format!(
                "Ripped file does not exist at path: {}",
                from_path.display()
            ));
        }

        fs::rename(from_path.as_path(), &target_path)
            .map_err(|e| format!("Failed to rename file: {e}"))?;
        Ok(target_path)
    }

    fn ripped_file_path(&self, app_state: &AppState) -> PathBuf {
        let title_filename = self.title.filename.as_ref().unwrap();
        self.create_video_dir(app_state).join(title_filename)
    }

    /// Get the full FTP upload file path for this video (movie or TV episode).
    ///
    /// Purpose:
    /// - Computes the exact remote file path where the video should be uploaded via FTP.
    /// - Combines the FTP upload directory (from config) with the Plex-compliant filename.
    /// - Used by upload routines to determine the destination for the file on the FTP server.
    ///
    /// How it works:
    /// - For movies: gets the FTP movie upload directory, then appends the movie filename.
    /// - For TV episodes: gets the FTP TV upload directory, then appends the episode filename.
    /// - Returns `None` if the FTP upload path is not configured.
    ///
    /// Examples:
    /// - Movie "Arrival (2016)":
    ///   `/mnt/ftp/Movies/Arrival (2016)/Arrival (2016).mkv`
    /// - TV episode "Stranger Things (2016)" S02E03:
    ///   `/mnt/ftp/TV Shows/Stranger Things (2016)/Season 02/Stranger Things (2016) - S02E03 - The Pollywog.mkv`
    ///
    /// Notes:
    /// - Does not create any directories or files; only computes the path.
    /// - Returns `None` if the FTP upload path is missing or not set in config.
    /// - Ensures uploads follow Plex directory and filename conventions for reliable parsing.
    pub fn upload_file_path(&self, app_state: &AppState) -> Option<PathBuf> {
        match &self.video {
            Video::Movie(movie) => Self::upload_movie_dir(app_state, movie)
                .map(|dir| dir.join(Self::movie_filename(movie))),
            Video::Tv(tv_season_episode) => {
                Self::upload_tv_season_dir(app_state, tv_season_episode)
                    .map(|dir| dir.join(Self::tv_episode_filename(tv_season_episode)))
            }
        }
    }

    /// Returns the FTP upload directory for this video (movie or TV episode).
    ///
    /// Purpose:
    /// - Computes the directory path where the video file should be uploaded via FTP.
    /// - For movies: returns the FTP movie upload directory (from config), e.g. `/mnt/ftp/Movies/Arrival (2016)/`.
    /// - For TV episodes: returns the FTP TV upload directory (from config), e.g. `/mnt/ftp/TV Shows/Stranger Things (2016)/Season 02/`.
    /// - Returns `None` if the FTP upload path is not configured.
    ///
    /// Usage:
    /// - Use this to determine the target directory for FTP uploads or external transfers.
    /// - Does not create the directory; only computes the path.
    pub fn upload_directory(&self, app_state: &AppState) -> Option<PathBuf> {
        match &self.video {
            Video::Movie(movie) => Self::upload_movie_dir(app_state, movie),
            Video::Tv(tv_season_episode) => {
                Self::upload_tv_season_dir(app_state, tv_season_episode)
            }
        }
    }

    // /// Get the upload directory path for this video, used for FTP or external transfers.
    // ///
    // /// Purpose:
    // /// - Returns the directory where the video file should be uploaded or placed for external access.
    // /// - Used by FTP routines and external sync scripts to determine the correct destination folder.
    // /// - Does NOT create the directory; only computes the path.
    // ///
    // /// How it works:
    // /// - For movies: returns `/Movies/Movie Name (Year)/` (see `movie_dir`).
    // /// - For TV episodes: returns `/TV Shows/Show Name (Year)/Season 01/` (see `seasons_episode_dir`).
    // /// - The path is based on Plex's recommended structure for optimal media scanning and organization.
    // ///
    // /// What it generates:
    // /// - For a movie: the folder containing the movie file and any related assets (e.g., posters, subtitles).
    // /// - For a TV episode: the season folder containing all episodes for that season.
    // ///
    // /// Examples:
    // /// - Movie "Arrival (2016)":
    // ///   /Movies/Arrival (2016)/
    // /// - TV episode "Stranger Things (2016)" S02E03:
    // ///   /TV Shows/Stranger Things (2016)/Season 02/
    // ///
    // /// Notes:
    // /// - This is the *upload* path, not the final file path; the actual video file will be placed inside this directory.
    // /// - Directory creation is handled elsewhere; this function only computes the target path.
    // /// - Ensures all uploads follow the Plex directory conventions for reliable metadata matching.
    // pub fn upload_directory(&self, app_state: &AppState) -> Option<PathBuf> {
    //     match &self.video {
    //         Video::Movie(movie) => Self::upload_movie_dir(app_state, movie),
    //         Video::Tv(tv_season_episode) => {
    //             Self::upload_tv_season_dir(app_state, tv_season_episode)
    //         }
    //     }
    // }

    ///   Ensure the necessary directory structure exists for this video file and return its path.
    ///
    /// Purpose:
    /// - Primary method called before writing video files to disk to guarantee the parent
    ///   directory structure is in place.
    /// - Handles both movies and TV episodes with appropriate Plex-compliant folder hierarchies.
    /// - Idempotent: safe to call multiple times; won't fail if directories already exist.
    ///
    /// How it works:
    /// 1. Checks the `Video` enum variant (Movie or Tv) to determine content type.
    /// 2. For movies: calls `create_movie_dir` which creates `/Movies/Movie Name (Year)/`
    /// 3. For TV: calls `create_tv_season_episode_dir` which creates
    ///    `/TV Shows/Show Name (Year)/Season 01/`
    /// 4. Uses `fs::create_dir_all()` internally, which recursively creates all missing
    ///    parent directories in the path without error if they already exist.
    ///
    /// Returns:
    /// - `PathBuf`: The created (or existing) directory path where the video file should be placed.
    ///
    /// Examples:
    /// - Movie "Inception (2010)" returns and ensures:
    ///   /Movies/Inception (2010)/
    /// - TV episode "Breaking Bad (2008)" S01E01 returns and ensures:
    ///   /TV Shows/Breaking Bad (2008)/Season 01/
    ///
    /// Note:
    /// - This creates the directory container, not the video file itself.
    /// - Panics if directory creation fails (e.g., permission issues).
    pub fn create_video_dir(&self, app_state: &AppState) -> PathBuf {
        match &self.video {
            Video::Movie(movie) => Self::create_movie_dir(app_state, movie),
            Video::Tv(tv_season_episode) => {
                Self::create_tv_season_episode_dir(app_state, tv_season_episode)
            }
        }
    }

    // pub fn video_dir(&self, app_state: &AppState) -> PathBuf {
    //     match &self.video {
    //         Video::Movie(movie) => Self::movie_dir(app_state, movie),
    //         Video::Tv(tv_season_episode) => Self::seasons_episode_dir(app_state, tv_season_episode),
    //     }
    // }

    /// Get the FTP upload directory for a movie, if configured.
    ///
    /// Purpose:
    /// - Returns the target directory for uploading a movie file via FTP or external sync.
    /// - Uses the configured FTP movie upload path from `AppState`.
    /// - Returns `None` if no FTP path is set.
    ///
    /// How it works:
    /// 1. Locks and reads the `ftp_movie_upload_path` from `AppState`.
    /// 2. If set, appends the movie's title and year to form the destination directory.
    /// 3. Returns the full path as `Some(PathBuf)`, or `None` if not configured.
    ///
    /// Example:
    /// - FTP path: `/mnt/ftp/Movies`, Movie: "Arrival (2016)" ->
    ///   `/mnt/ftp/Movies/Arrival (2016)/`
    ///
    /// Notes:
    /// - Does not create the directory; only computes the path.
    /// - Used for external transfers, not local Plex organization.
    fn upload_movie_dir(app_state: &AppState, movie: &MoviePartEdition) -> Option<PathBuf> {
        let movies_dir = app_state
            .ftp_movie_upload_path
            .lock()
            .expect("failed to lock ftp_movie_upload_path");
        movies_dir
            .as_ref()
            .map(|dir| dir.join(movie.movie.title_year()))
    }

    /// Get the FTP upload directory for a TV episode, if configured.
    ///
    /// Purpose:
    /// - Returns the target directory for uploading a TV episode file via FTP or external sync.
    /// - Uses the configured FTP TV upload path from `AppState`.
    /// - Returns `None` if no FTP path is set.
    ///
    /// How it works:
    /// 1. Locks and reads the `ftp_tv_upload_path` from `AppState`.
    /// 2. If set, appends the full Plex-compliant episode path (show, season, episode filename).
    /// 3. Returns the full path as `Some(PathBuf)`, or `None` if not configured.
    ///
    /// Example:
    /// - FTP path: `/mnt/ftp/TV Shows`, Show: "Stranger Things (2016)", Season: 2, Episode: 3 ->
    ///   `/mnt/ftp/TV Shows/Stranger Things (2016)/Season 02/Stranger Things (2016) - S02E03 - The Pollywog.mkv`
    ///
    /// Notes:
    /// - Does not create the directory; only computes the path.
    /// - Used for external transfers, not local Plex organization.
    /// - Returns the full episode file path, not just the season folder.
    fn upload_tv_season_dir(
        app_state: &AppState,
        tv_season_episode: &TvSeasonEpisode,
    ) -> Option<PathBuf> {
        let tv_shows_dir = app_state
            .ftp_tv_upload_path
            .lock()
            .expect("failed to lock ftp_tv_upload_path");
        tv_shows_dir
            .as_ref()
            .map(|dir| dir.join(Self::tv_season_episode_path(app_state, tv_season_episode)))
    }

    fn create_movie_dir(app_state: &AppState, movie: &MoviePartEdition) -> PathBuf {
        let dir = Self::movie_dir(app_state, &movie.movie);
        if !dir.exists() {
            fs::create_dir_all(&dir)
                .unwrap_or_else(|_| panic!("Failed to create {}", dir.display()));
        }
        dir
    }

    fn create_tv_season_episode_dir(
        app_state: &AppState,
        tv_season_episode: &TvSeasonEpisode,
    ) -> PathBuf {
        let dir = Self::seasons_episode_dir(app_state, tv_season_episode);
        if !dir.exists() {
            fs::create_dir_all(&dir)
                .unwrap_or_else(|_| panic!("Failed to create {}", dir.display()));
        }
        dir
    }

    /// Resolve the filesystem directory for a movie following Plex's recommended structure.
    ///
    /// Layout produced:
    ///   /Movies/Movie Name (Year)/
    ///
    /// Purpose:
    /// - Provides the directory path where a movie's video file should be stored.
    /// - Each movie gets its own subdirectory under the main movies folder, containing
    ///   the movie file and any associated assets (posters, subtitles, extras, etc.).
    /// - Used by `movie_path` to construct the complete file path, by `create_movie_dir`
    ///   to ensure the directory exists, and by `upload_directory` for FTP operations.
    ///
    /// Steps:
    /// 1. Lock and read `movies_dir` from `AppState` (configured base path for all movies).
    /// 2. Append the movie's title with year: `Movie Name (Year)`.
    /// 3. Return the composed `PathBuf` without filesystem interaction (no creation/validation).
    ///
    /// Examples:
    /// - "Inception" (2010) ->
    ///   /Movies/Inception (2010)/
    /// - "The Matrix" (1999) ->
    ///   /Movies/The Matrix (1999)/
    /// - "Dune" (2021) ->
    ///   /Movies/Dune (2021)/
    ///
    /// Note:
    /// - This only constructs the path; directory creation is handled separately by
    ///   `create_movie_dir` when needed.
    fn movie_dir(app_state: &AppState, movie: &MovieResponse) -> PathBuf {
        let movies_dir = app_state
            .movies_dir
            .read()
            .expect("failed to lock movies_dir");
        movies_dir.join(movie.title_year())
    }

    /// Resolve the filesystem directory for a specific TV season (used as the parent
    /// directory for all episode files belonging to that season) following Plex's
    /// recommended structure.
    ///
    /// Layout produced:
    ///   /TV Shows/Show Name (Year)/Season 01/
    ///
    /// Purpose:
    /// - Central place to create or reference the season folder before writing episode files.
    /// - Ensures consistent zero-padded season numbering ("Season 01" vs "Season 1") for
    ///   predictable lexical ordering and compatibility with typical Plex scanning patterns.
    /// - Used by `tv_season_episode_path` to append the episode filename, and by
    ///   `create_tv_season_episode_dir` to ensure the directory exists on disk.
    ///
    /// Steps:
    /// 1. Lock and read `tv_shows_dir` from `AppState` (base root for all TV content).
    /// 2. Append the show directory using title + year: `Show Name (Year)`.
    /// 3. Append zero-padded season directory: `Season 01`.
    /// 4. Return the composed `PathBuf` without creating it (creation handled elsewhere).
    ///
    /// Examples:
    /// - Show: "Example Show" (2023), Season: 1 ->
    ///   /TV Shows/Example Show (2023)/Season 01/
    /// - Show: "Mystery Saga" (2019), Season: 11 ->
    ///   /TV Shows/Mystery Saga (2019)/Season 11/
    ///
    /// Note:
    /// - Only path construction occurs here; existence checks/creation are done in
    ///   `create_tv_season_episode_dir`.
    fn seasons_episode_dir(app_state: &AppState, tv_season_episode: &TvSeasonEpisode) -> PathBuf {
        let tv_shows_dir = app_state
            .tv_shows_dir
            .read()
            .expect("failed to lock tv_shows_dir");
        let dir = tv_shows_dir
            .join(tv_season_episode.tv.title_year())
            .join(format!(
                "Season {:02}",
                tv_season_episode.season.season_number
            ));
        dir
    }

    /// Returns the full filesystem path for this video (movie or TV episode) following Plex naming conventions.
    ///
    /// Purpose:
    /// - Computes the final destination path for the video file, including directory and filename.
    /// - For movies: returns `/Movies/Movie Name (Year)/Movie Name (Year).mkv`.
    /// - For TV episodes: returns `/TV Shows/Show Name (Year)/Season 01/Show Name (Year) - S01E01 - Episode Title.mkv`.
    /// - Used for moving, renaming, or referencing the video file in the correct Plex-compliant location.
    ///
    /// How it works:
    /// - Checks the type of video (Movie or TV).
    /// - Calls the appropriate helper to build the full path for the video file.
    ///
    /// Usage:
    /// - Use this when you need the absolute path for storing, moving, or referencing the video file on disk.
    pub fn video_path(&self, app_state: &AppState) -> PathBuf {
        match &self.video {
            Video::Movie(movie) => Self::movie_path(app_state, movie),
            Video::Tv(tv_season_episode) => {
                Self::tv_season_episode_path(app_state, tv_season_episode)
            }
        }
    }

    /// Build the full filesystem path for a movie following Plex naming conventions.
    ///
    /// Directory layout (recommended):
    ///   /Movies/
    ///     Movie Name (Year)/
    ///       Movie Name (Year).mkv
    ///
    /// This function produces the final file path by:
    /// 1. Resolving the movie directory via `movie_dir` ("Movie Name (Year)").
    /// 2. Constructing the filename: "Movie Name (Year).mkv".
    /// 3. Joining directory + filename into a `PathBuf`.
    ///
    /// Examples:
    /// - Standard movie:
    ///   /Movies/Inception (2010)/Inception (2010).mkv
    /// - Movie with year disambiguation:
    ///   /Movies/Dune (2021)/Dune (2021).mkv
    /// - Title with punctuation (colon retained):
    ///   /Movies/Star Wars: Episode IV - A New Hope (1977)/Star Wars: Episode IV - A New Hope (1977).mkv
    /// - Title with internal slash sanitized earlier (if applied outside):
    ///   /Movies/Artist Documentary Part 1-2 (2022)/Artist Documentary Part 1-2 (2022).mkv
    /// - Edition (filename only):
    ///   /Movies/Blade Runner (1982)/Blade Runner (1982) {edition-Final Cut}.mkv
    ///
    /// Notes:
    /// - The extension is currently hard-coded to ".mkv"; adjust if supporting multiple codecs.
    /// - Edition variants (e.g. Director's Cut) would require an adjusted naming convention (not yet implemented here).
    ///   Build the full filesystem path for a movie, supporting part and edition info.
    ///
    /// Directory Layout (per Plex recommendations):
    ///   /Movies/Movie Name (Year)/Movie Name (Year) {edition-Final Cut}-pt1.mkv
    ///
    /// The directory does NOT include the edition tag, only the filename does.
    fn movie_path(app_state: &AppState, movie: &MoviePartEdition) -> PathBuf {
        let dir = Self::movie_dir(app_state, &movie.movie);
        let file_name = Self::movie_filename(movie);
        dir.join(file_name)
    }

    /// Build the Plex-compliant filename for a movie, supporting part and edition info.
    ///
    /// Naming format (single-part, no edition):
    ///   Movie Name (Year).mkv
    /// With part: Movie Name (Year)-pt1.mkv
    /// With edition: Movie Name (Year) {edition-Final Cut}.mkv
    /// With both: Movie Name (Year) {edition-Final Cut}-pt1.mkv
    fn movie_filename(movie: &MoviePartEdition) -> String {
        let mut base = movie.movie.title_year();
        // Add edition if present
        if let Some(ref edition) = movie.edition {
            base = format!("{base} {{edition-{edition}}}");
        }
        let mut file_name = format!("{base}.mkv");
        // Add part if present
        if let Some(part) = movie.part {
            file_name = format!("{}-pt{}", file_name.trim_end_matches(".mkv"), part);
            file_name.push_str(".mkv");
        }
        file_name
    }

    /// Build the full filesystem path for a TV episode following Plex naming conventions.
    ///
    /// Directory Layout (per Plex recommendations):
    ///   /TV Shows/
    ///     Show Name (Year)/
    ///       Season 01/
    ///         Show Name (Year) - S01E01 - Episode Title.mkv
    ///
    /// This function composes that final file path by:
    /// 1. Resolving the base show directory: `tv_shows_dir/Show Name (Year)`.
    /// 2. Appending the zero-padded season directory: `Season 01`.
    /// 3. Generating the episode filename via `tv_episode_file_name` (which includes season/episode numbers,
    ///    a sanitized title, and optional multi-part suffix `-ptN`).
    /// 4. Joining directory + filename into a single `PathBuf`.
    ///
    /// Examples (returned `PathBuf`):
    /// - Single-part episode:
    ///   /TV Shows/Example Show (2023)/Season 01/Example Show (2023) - S01E01 - Pilot.mkv
    /// - Multi-part episode (part 2):
    ///   /TV Shows/Example Show (2023)/Season 01/Example Show (2023) - S01E05 - Finale-pt2.mkv
    /// - Title containing a slash ("Act 1/Act 2") becomes:
    ///   /TV Shows/Example Show (2023)/Season 01/Example Show (2023) - S01E03 - Act 1-Act 2.mkv
    ///
    /// Notes:
    /// - Forward slashes in episode titles are replaced with `-` to avoid unintended nested directories.
    /// - Season and episode numbers are zero-padded to two digits for lexicographic ordering.
    /// - Multi-part suffix is added only when `TvSeasonEpisode.part` is `Some(n)`.
    ///
    /// See `tv_episode_file_name` for detailed filename construction logic.
    fn tv_season_episode_path(
        app_state: &AppState,
        tv_season_episode: &TvSeasonEpisode,
    ) -> PathBuf {
        let dir = Self::seasons_episode_dir(app_state, tv_season_episode);
        let file_name = Self::tv_episode_filename(tv_season_episode);
        dir.join(file_name)
    }

    /// Build the Plex-compliant filename for a TV episode.
    ///
    /// Naming format (single-part episodes):
    ///   Show Name (Year) - S01E01 - Episode Title.mkv
    /// If the episode is split into multiple parts (e.g. disc segments), a part suffix is appended:
    ///   Show Name (Year) - S01E01 - Episode Title-pt2.mkv
    ///
    /// Steps:
    /// 1. Sanitize the raw episode title by replacing forward slashes '/' with '-'. This prevents
    ///    unintended directory creation and adheres to filesystem safety.
    /// 2. Format the base filename using show title + season/episode numbers (zero-padded) + sanitized title.
    /// 3. If a `part` number exists, strip the trailing ".mkv", append the `-ptX` suffix, then restore the extension.
    /// 4. Return the final filename string.
    fn tv_episode_filename(tv_season_episode: &TvSeasonEpisode) -> String {
        // 1. Sanitize episode title to avoid path separator issues
        let episode_title = tv_season_episode.episode.name.replace('/', "-");

        // 2. Base filename with zero-padded season and episode numbers
        let mut file_name = format!(
            "{} - S{:02}E{:02} - {}.mkv",
            tv_season_episode.tv.title_year(),
            tv_season_episode.season.season_number,
            tv_season_episode.episode.episode_number,
            episode_title
        );

        // 3. Append part suffix if this is a multi-part episode
        if let Some(part) = tv_season_episode.part {
            file_name = format!("{}-pt{}", file_name.trim_end_matches(".mkv"), part);
            file_name.push_str(".mkv");
        }

        // 4. Return final filename
        file_name
    }
}

#[derive(Serialize, Clone)]
pub enum Video {
    Tv(Box<TvSeasonEpisode>),
    Movie(Box<MoviePartEdition>),
}

impl Video {
    pub fn runtime_seconds(&self) -> Option<u64> {
        match self {
            Video::Movie(movie) => Some(movie.runtime_seconds()),
            Video::Tv(tv) => tv.runtime_seconds(),
        }
    }

    pub fn runtime_range(&self) -> Option<std::ops::Range<u64>> {
        match self {
            Video::Movie(movie) => Some(movie.runtime_range()),
            Video::Tv(tv) => Some(tv.episode.runtime_range()),
        }
    }
}

#[derive(Serialize, Clone)]
pub struct TvSeasonEpisode {
    pub episode: SeasonEpisode,
    pub season: SeasonResponse,
    pub tv: TvResponse,
    pub part: Option<u16>,
}

impl TvSeasonEpisode {
    /// Returns the Plex-compliant display title for this TV episode.
    ///
    /// Format:
    ///   Show Name (Year) - SXXEYY - Episode Title
    /// Where:
    ///   - Show Name (Year): Title and year of the TV show
    ///   - SXX: Zero-padded season number
    ///   - EYY: Zero-padded episode number
    ///   - Episode Title: Name of the episode
    ///
    /// Example: "Breaking Bad (2008) - S01E01 - Pilot"
    ///
    /// This format is used for filenames and display, ensuring compatibility with Plex and other media managers.
    pub fn title(&self) -> String {
        format!(
            "{} - S{:02}E{:02} - {}",
            self.tv.title_year(),
            self.season.season_number,
            self.episode.episode_number,
            self.episode.name
        )
    }

    /// Returns the runtime of this TV episode in seconds, if available.
    ///
    /// The runtime is extracted from the episode metadata and converted to u64.
    /// Returns `None` if the runtime is not set.
    pub fn runtime_seconds(&self) -> Option<u64> {
        self.episode.runtime.map(|r| r as u64 * 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a minimal test MovieResponse
    fn create_test_movie(title: &str, year: i32, runtime_minutes: u64) -> MovieResponse {
        MovieResponse {
            adult: false,
            backdrop_path: None,
            genres: vec![],
            homepage: String::new(),
            id: 1,
            imdb_id: String::new(),
            origin_country: vec![],
            original_language: String::new(),
            original_title: title.to_string(),
            overview: "Test movie".to_string(),
            popularity: 0.0,
            poster_path: None,
            release_date: Some(format!("{year}-01-01")),
            revenue: 0,
            runtime: runtime_minutes,
            title: title.to_string(),
        }
    }

    #[test]
    fn test_movie_filename_no_part_no_edition() {
        let movie = MoviePartEdition {
            movie: create_test_movie("Inception", 2010, 120),
            part: None,
            edition: None,
        };

        let filename = TitleVideo::movie_filename(&movie);
        assert_eq!(filename, "Inception (2010).mkv");
    }

    #[test]
    fn test_movie_filename_with_part() {
        let movie = MoviePartEdition {
            movie: create_test_movie("The Lord of the Rings", 2001, 180),
            part: Some(1),
            edition: None,
        };

        let filename = TitleVideo::movie_filename(&movie);
        assert_eq!(filename, "The Lord of the Rings (2001)-pt1.mkv");
    }

    #[test]
    fn test_movie_filename_with_edition() {
        let movie = MoviePartEdition {
            movie: create_test_movie("Blade Runner", 1982, 117),
            part: None,
            edition: Some("Final Cut".to_string()),
        };

        let filename = TitleVideo::movie_filename(&movie);
        assert_eq!(filename, "Blade Runner (1982) {edition-Final Cut}.mkv");
    }

    #[test]
    fn test_movie_filename_with_part_and_edition() {
        let movie = MoviePartEdition {
            movie: create_test_movie("Kill Bill", 2003, 111),
            part: Some(2),
            edition: Some("Uncut".to_string()),
        };

        let filename = TitleVideo::movie_filename(&movie);
        assert_eq!(filename, "Kill Bill (2003) {edition-Uncut}-pt2.mkv");
    }
}
