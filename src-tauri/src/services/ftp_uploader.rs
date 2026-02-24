use crate::progress_tracker::{self, ProgressOptions};
use crate::state::job_state::{emit_progress, Job};
use crate::state::title_video::TitleVideo;
use crate::state::AppState;
use crate::the_movie_db::{SeasonResponse, TvResponse};
use log::{debug, error};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::io::{BufReader, Read};
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, RwLock};
use suppaftp::types::FileType;
use suppaftp::FtpError as SuppaFtpError;
use suppaftp::FtpStream;
use tauri::{AppHandle, Manager, State};

const CHUNK_SIZE: usize = 8192; // 8KB chunk size for streaming upload

struct FileInfo {
    file_size: u64,
    reader: BufReader<File>,
}

/// Structured error information for FTP validation failures
#[derive(Clone, PartialEq, Eq)]
pub struct FtpValidationError {
    pub errors: Vec<FtpError>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct FtpError {
    pub message: String,
    pub path: Option<String>,
    pub error_detail: Option<String>,
    pub suggestions: Vec<String>,
    pub error_type: FtpErrorType,
}

#[derive(Clone, PartialEq, Eq)]
pub enum FtpErrorType {
    MissingConfig,
    ConnectionFailed,
    PathNotFound,
    Other,
}

impl FtpValidationError {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn add_error(
        &mut self,
        message: String,
        error_type: FtpErrorType,
        path: Option<String>,
        error_detail: Option<String>,
        suggestions: Vec<String>,
    ) {
        self.errors.push(FtpError {
            message,
            path,
            error_detail,
            suggestions,
            error_type,
        });
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

pub fn file_exists(relative_mkv_file_path: &String, state: &State<'_, AppState>) -> bool {
    let movie_upload_path = match state.lock_ftp_movie_upload_path().clone() {
        Some(value) => format!("{}/{relative_mkv_file_path}", value.to_string_lossy()),
        None => return false,
    };
    let mut ftp = match connect_to_ftp(state) {
        Ok(ftp) => ftp,
        Err(_) => return false,
    };
    ftp.transfer_type(FileType::Binary)
        .expect("failed to set binary mode");

    let exists = ftp.size(&movie_upload_path).is_ok();
    debug!("FTP Exists: {movie_upload_path} {exists}");
    match ftp.quit() {
        Ok(_) => debug!("FTP Connection Closed"),
        Err(error) => error!("Failed to close FTP connection {error:?}"),
    }
    exists
}

pub fn tv_ripped_episode_numbers(
    tv: &TvResponse,
    season: &SeasonResponse,
    state: &State<'_, AppState>,
) -> HashSet<u32> {
    let tv_upload_path = match state.lock_ftp_tv_upload_path().clone() {
        Some(value) => value,
        None => return HashSet::new(),
    };

    let season_dir = tv_upload_path
        .join(tv.title_year())
        .join(format!("Season {:02}", season.season_number));

    let mut ftp = match connect_to_ftp(state) {
        Ok(ftp) => ftp,
        Err(_) => return HashSet::new(),
    };

    let mut ripped_episode_numbers = HashSet::new();

    if ftp.cwd(season_dir.to_string_lossy()).is_ok() {
        if let Ok(entries) = ftp.nlst(None) {
            for entry in entries {
                let file_name = entry.rsplit('/').next().unwrap_or(&entry).trim();
                if let Some(episode_number) = parse_episode_number_from_tv_filename(
                    file_name,
                    &tv.title_year(),
                    season.season_number,
                ) {
                    ripped_episode_numbers.insert(episode_number);
                }
            }
        }
    }

    match ftp.quit() {
        Ok(_) => debug!("FTP Connection Closed"),
        Err(error) => error!("Failed to close FTP connection {error:?}"),
    }

    ripped_episode_numbers
}

fn parse_episode_number_from_tv_filename(
    file_name: &str,
    tv_title_year: &str,
    season_number: u32,
) -> Option<u32> {
    parse_episode_info_from_tv_filename(file_name, tv_title_year, season_number)
        .map(|(episode_number, _)| episode_number)
}

fn parse_episode_info_from_tv_filename(
    file_name: &str,
    tv_title_year: &str,
    season_number: u32,
) -> Option<(u32, Option<u16>)> {
    let lower_name = file_name.to_lowercase();
    if !lower_name.ends_with(".mkv") {
        return None;
    }

    let prefix = format!("{} - s{:02}e", tv_title_year.to_lowercase(), season_number);
    if !lower_name.starts_with(&prefix) {
        return None;
    }

    let rest = &lower_name[prefix.len()..];
    let episode_digits: String = rest.chars().take_while(|ch| ch.is_ascii_digit()).collect();

    if episode_digits.is_empty() {
        return None;
    }

    let after_episode_digits = &rest[episode_digits.len()..];
    if !after_episode_digits.starts_with(" -") {
        return None;
    }

    let episode_number = episode_digits.parse::<u32>().ok()?;
    let part = parse_part_suffix(&lower_name);
    Some((episode_number, part))
}

fn parse_part_suffix(lower_name: &str) -> Option<u16> {
    let suffix = lower_name.strip_suffix(".mkv")?;
    let marker = "-pt";
    let index = suffix.rfind(marker)?;
    let part_digits = &suffix[index + marker.len()..];
    if part_digits.is_empty() || !part_digits.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    part_digits.parse::<u16>().ok()
}

fn build_tv_episode_filename(
    tv: &TvResponse,
    season: &SeasonResponse,
    episode: &crate::the_movie_db::SeasonEpisode,
    part: Option<u16>,
    extension: &str,
) -> String {
    let episode_title = episode.name.replace('/', "-");
    let mut file_name = format!(
        "{} - S{:02}E{:02} - {}.{extension}",
        tv.title_year(),
        season.season_number,
        episode.episode_number,
        episode_title
    );

    if let Some(part) = part {
        let base = file_name.trim_end_matches(&format!(".{extension}"));
        file_name = format!("{base}-pt{part}.{extension}");
    }

    file_name
}

pub fn reorder_tv_episode_files(
    tv: &TvResponse,
    season: &SeasonResponse,
    swaps: &[(u32, u32)],
    state: &State<'_, AppState>,
) -> Result<usize, String> {
    if swaps.is_empty() {
        return Ok(0);
    }

    let tv_upload_path = match state.lock_ftp_tv_upload_path().clone() {
        Some(value) => value,
        None => return Err("FTP TV upload path is not configured".to_string()),
    };

    let season_dir = tv_upload_path
        .join(tv.title_year())
        .join(format!("Season {:02}", season.season_number));

    let mut ftp =
        connect_to_ftp(state).map_err(|e| format!("Failed to connect to FTP server: {e:?}"))?;
    ftp.transfer_type(FileType::Binary)
        .map_err(|e| format!("Failed to set FTP binary mode: {e:?}"))?;

    ftp.cwd(season_dir.to_string_lossy())
        .map_err(|e| format!("Failed to access season directory on FTP: {e:?}"))?;

    let entries = ftp
        .nlst(None)
        .map_err(|e| format!("Failed to list season directory on FTP: {e:?}"))?;

    let mut episode_files: HashMap<u32, Vec<String>> = HashMap::new();
    let mut existing_files: HashSet<String> = HashSet::new();

    for entry in entries {
        let file_name = entry
            .rsplit('/')
            .next()
            .unwrap_or(&entry)
            .trim()
            .to_string();
        if let Some((episode_number, _)) =
            parse_episode_info_from_tv_filename(&file_name, &tv.title_year(), season.season_number)
        {
            episode_files
                .entry(episode_number)
                .or_default()
                .push(file_name.clone());
            existing_files.insert(file_name);
        }
    }

    let mut episode_lookup = HashMap::new();
    for episode in &season.episodes {
        episode_lookup.insert(episode.episode_number, episode);
    }

    let mut move_ops: Vec<(String, String)> = Vec::new();
    let mut source_files: HashSet<String> = HashSet::new();
    let mut target_files: HashSet<String> = HashSet::new();

    for (from_episode, to_episode) in swaps {
        let source_files_for_episode = episode_files
            .get(from_episode)
            .ok_or_else(|| format!("No FTP files found for episode {from_episode}"))?;

        if source_files_for_episode.len() > 1 {
            let all_have_parts = source_files_for_episode.iter().all(|name| {
                let lower_name = name.to_lowercase();
                parse_part_suffix(&lower_name).is_some()
            });
            if !all_have_parts {
                return Err(format!(
                    "Episode {from_episode} has multiple files without part suffixes"
                ));
            }
        }

        let target_episode = episode_lookup
            .get(to_episode)
            .ok_or_else(|| format!("Episode {to_episode} does not exist in this season"))?;

        for source_file in source_files_for_episode {
            let lower_name = source_file.to_lowercase();
            let part = parse_part_suffix(&lower_name);
            let extension = std::path::Path::new(source_file)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("mkv");

            let target_file =
                build_tv_episode_filename(tv, season, target_episode, part, extension);

            if !target_files.insert(target_file.clone()) {
                return Err(format!(
                    "Multiple files are mapped to the same destination: {target_file}"
                ));
            }

            source_files.insert(source_file.clone());
            move_ops.push((source_file.clone(), target_file));
        }
    }

    for target in &target_files {
        if existing_files.contains(target) && !source_files.contains(target) {
            return Err(format!("Destination file already exists: {target}"));
        }
    }

    let swap_stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);

    let mut temp_moves: Vec<(String, String)> = Vec::new();

    for (index, (source, target)) in move_ops.iter().enumerate() {
        let extension = std::path::Path::new(source)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("mkv");
        let temp_name = format!(".reelix-swap-{swap_stamp}-{index}.{extension}");
        ftp.rename(source, &temp_name)
            .map_err(|e| format!("Failed to rename {source} to temp file: {e:?}"))?;
        temp_moves.push((temp_name, target.clone()));
    }

    for (temp_name, target_name) in temp_moves {
        ftp.rename(&temp_name, &target_name)
            .map_err(|e| format!("Failed to rename {temp_name} to {target_name}: {e:?}"))?;
    }

    ftp.quit()
        .map_err(|e| format!("Failed to close FTP connection: {e:?}"))?;

    Ok(move_ops.len())
}

#[cfg(test)]
mod tests {
    use super::{
        parse_episode_info_from_tv_filename, parse_episode_number_from_tv_filename,
        parse_part_suffix,
    };

    #[test]
    fn parses_standard_episode_filename() {
        let result = parse_episode_number_from_tv_filename(
            "Example Show (2023) - S01E03 - Third Episode.mkv",
            "Example Show (2023)",
            1,
        );

        assert_eq!(result, Some(3));
    }

    #[test]
    fn parses_multipart_episode_filename() {
        let result = parse_episode_number_from_tv_filename(
            "Example Show (2023) - S01E03 - Third Episode-pt2.mkv",
            "Example Show (2023)",
            1,
        );

        assert_eq!(result, Some(3));
    }

    #[test]
    fn ignores_other_show_and_non_mkv_files() {
        let wrong_show = parse_episode_number_from_tv_filename(
            "Different Show (2023) - S01E03 - Third Episode.mkv",
            "Example Show (2023)",
            1,
        );
        let wrong_extension = parse_episode_number_from_tv_filename(
            "Example Show (2023) - S01E03 - Third Episode.mp4",
            "Example Show (2023)",
            1,
        );

        assert_eq!(wrong_show, None);
        assert_eq!(wrong_extension, None);
    }

    #[test]
    fn ignores_wrong_season() {
        let result = parse_episode_number_from_tv_filename(
            "Example Show (2023) - S02E03 - Third Episode.mkv",
            "Example Show (2023)",
            1,
        );

        assert_eq!(result, None);
    }

    #[test]
    fn parses_episode_info_with_part_suffix() {
        let result = parse_episode_info_from_tv_filename(
            "Example Show (2023) - S01E05 - Finale-pt2.mkv",
            "Example Show (2023)",
            1,
        );

        assert_eq!(result, Some((5, Some(2))));
    }

    #[test]
    fn parses_episode_info_without_part_suffix() {
        let result = parse_episode_info_from_tv_filename(
            "Example Show (2023) - S01E07 - Episode Seven.mkv",
            "Example Show (2023)",
            1,
        );

        assert_eq!(result, Some((7, None)));
    }

    #[test]
    fn ignores_invalid_part_suffix() {
        let result = parse_part_suffix("example show (2023) - s01e01 - pilot-ptx.mkv");

        assert_eq!(result, None);
    }
}

/// Connects, authenticates, and Changes current directory to MOVIE_UPLOAD_PATH
pub fn connect_to_ftp(state: &State<'_, AppState>) -> Result<FtpStream, SuppaFtpError> {
    let ftp_host = match state.lock_ftp_host().clone() {
        Some(ftp_host) => ftp_host,
        None => {
            return Err(SuppaFtpError::ConnectionError(std::io::Error::other(
                "ftp host missing",
            )));
        }
    };
    let ftp_pass = match state.lock_ftp_pass().clone() {
        Some(ftp_pass) => ftp_pass,
        None => {
            return Err(SuppaFtpError::ConnectionError(std::io::Error::other(
                "ftp pass missing",
            )));
        }
    };
    let ftp_user = match state.lock_ftp_user().clone() {
        Some(ftp_user) => ftp_user,
        None => {
            return Err(SuppaFtpError::ConnectionError(std::io::Error::other(
                "ftp user missing",
            )));
        }
    };

    // Ensure the host has a port; default to FTP standard port 21 if not provided
    // This is only been a problem on linux where windows & macos ftp libraries auto add :21
    let ftp_addr = if ftp_host.contains(':') {
        ftp_host.clone()
    } else {
        format!("{ftp_host}:21")
    };

    debug!("Connecting to FTP server at: {ftp_addr}");
    let mut ftp_stream = FtpStream::connect(&ftp_addr)?;
    ftp_stream.login(ftp_user, ftp_pass)?;
    Ok(ftp_stream)
}

// Open the local file and capture relative info used to send the data
fn file_info(filepath: &Path) -> Result<FileInfo, String> {
    let file = match File::open(filepath) {
        Ok(f) => f,
        Err(e) => {
            return Err(format!(
                "Failed to open file path {}: {e}",
                filepath.to_string_lossy()
            ))
        }
    };
    let metadata = match file.metadata() {
        Ok(m) => m,
        Err(_e) => {
            return Err(format!(
                "Failed to capture metadata of file {}",
                filepath.display()
            ))
        }
    };
    let file_size = metadata.len();
    let reader = BufReader::new(file);
    Ok(FileInfo { file_size, reader })
}

fn create_upload_dir(
    state: &State<'_, AppState>,
    ftp_stream: &mut FtpStream,
    _job: &Arc<RwLock<Job>>,
    title_video: &Arc<RwLock<TitleVideo>>,
) -> Result<PathBuf, String> {
    let title_video_guard = title_video.read().unwrap();

    // Determine content type for better error messages
    let content_type = match &title_video_guard.video {
        crate::state::title_video::Video::Movie(_) => "movie",
        crate::state::title_video::Video::Tv(_) => "TV show",
    };

    let upload_dir = title_video_guard
        .upload_directory(state)
        .ok_or_else(|| {
            format!(
                "FTP upload path not configured for {}. Please configure FTP {} upload path in settings.",
                content_type,
                if content_type == "movie" { "movie" } else { "TV" }
            )
        })?;

    debug!("creating upload dir upload_dir={upload_dir:?}");

    ensure_remote_dir_recursive(ftp_stream, &upload_dir)?;

    Ok(upload_dir)
}

/// Ensures that each directory component in the given path exists on the FTP server.
///
/// How it works:
/// - Iterates through each component of the provided `dir` path.
/// - For each normal directory component:
///   - Attempts to change into the directory (`cwd`).
///   - If the directory does not exist (`cwd` fails), creates it (`mkdir`) and then changes into it.
///   - This guarantees that the full path exists and the FTP server's working directory is set to the deepest component.
/// - Other path components (prefix, root, current, parent) are not handled and will panic if encountered (via `todo!()`).
///
/// Usage:
/// - Call this function after connecting to the FTP server and before uploading a file to ensure the remote directory structure exists.
/// - The FTP server's working directory will be set to the final directory in the path after this function completes.
fn ensure_remote_dir_recursive(ftp_stream: &mut FtpStream, dir: &Path) -> Result<(), String> {
    let mut first = true;

    for component in dir.components() {
        match component {
            Component::RootDir => {
                // Absolute path: reset to FTP root/entrypoint once
                // Most FTP servers treat "/" as the user's root anyway.
                if first {
                    ftp_stream
                        .cwd("/")
                        .map_err(|e| format!("failed to cd to ftp root '/': {e}"))?;
                }
            }
            Component::CurDir => {
                // "." → do nothing
            }
            Component::ParentDir => {
                // ".." → go up one directory
                ftp_stream
                    .cdup()
                    .map_err(|e| format!("failed to cd to parent dir '..': {e}"))?;
            }
            Component::Normal(name) => {
                let name = name.to_string_lossy();

                // Try to cd into the directory; if it fails, create it then cd into it.
                match ftp_stream.cwd(&name) {
                    Ok(_) => {
                        // Directory exists, now inside it
                    }
                    Err(_) => {
                        ftp_stream
                            .mkdir(&name)
                            .map_err(|e| format!("failed to create dir {name}: {e}"))?;

                        ftp_stream
                            .cwd(&name)
                            .map_err(|e| format!("failed to cd into dir {name}: {e}"))?;
                    }
                }
            }
            Component::Prefix(prefix) => {
                // Windows-style nonsense like "C:\"; not expected on Unix-y FTP paths.
                return Err(format!("unexpected path prefix component: {prefix:?}"));
            }
        }

        first = false;
    }

    Ok(())
}

/// Extracts the filename from a given file path and returns it as a String.
///
/// Purpose:
/// - Used to get just the file name (e.g., "movie.mkv") from a full path (e.g., "/path/to/movie.mkv").
/// - Converts the filename to a UTF-8 string, handling non-UTF-8 filenames gracefully.
///
/// Usage:
/// - Call this when you need the file name for FTP upload, logging, or display purposes.
/// - Panics if the path does not have a file name component.
fn filename(filepath: &Path) -> String {
    let filename = filepath.file_name().unwrap();
    filename.to_string_lossy().to_string()
}

fn start_upload(
    app_handle: &AppHandle,
    ftp_stream: &mut FtpStream,
    job: &Arc<RwLock<Job>>,
    title_video: &Arc<RwLock<TitleVideo>>,
) -> Result<(), String> {
    let state = app_handle.state::<AppState>();
    let multiple_parts = job
        .read()
        .expect("Failed to acquire read lock on job")
        .has_multiple_parts(&title_video.read().unwrap());
    let upload_file_path = title_video
        .read()
        .unwrap()
        .upload_file_path(&state, multiple_parts);
    if upload_file_path.is_none() {
        return Err("Failed to get upload file path".to_string());
    }
    let upload_file_path = upload_file_path.unwrap();
    let local_file_path = title_video
        .read()
        .unwrap()
        .video_path(&state, multiple_parts);
    debug!(
        "Start uploading {} to {:?}",
        upload_file_path.display(),
        ftp_stream.pwd()
    );

    let mut file_info = file_info(&local_file_path)?;
    let filename = filename(&local_file_path);
    debug!("File name will be {filename}");
    ftp_stream
        .transfer_type(FileType::Binary)
        .expect("failed to set binary mode");
    let tracker = new_tracker();
    job.write()
        .expect("Failed to acquire write lock on job")
        .update_title(&title_video.read().unwrap().clone());
    job.write()
        .expect("Failed to acquire write lock on job")
        .subtitle = Some(format!("Uploading {filename}"));
    job.read()
        .expect("Failed to acquire read lock on job")
        .emit_progress_change(app_handle);
    // Start uploading stream by creating a data stream object
    let mut data_stream = ftp_stream
        .put_with_stream(filename)
        .map_err(|e| format!("failed to open data stream {e}"))?;
    // Making extra sure there is nothing hanging around.
    data_stream
        .flush()
        .map_err(|e| format!("failed to flush stream: {e}"))?;
    // Upload in chunks and track progress
    let mut buffer = [0u8; CHUNK_SIZE];
    let mut total_bytes_sent: u64 = 0;
    loop {
        let bytes_read = file_info
            .reader
            .read(&mut buffer)
            .map_err(|e| format!("failed to read file info {e}"))?;
        if bytes_read == 0 {
            break;
        }

        data_stream
            .write_all(&buffer[..bytes_read])
            .map_err(|e| format!("failed to upload file {e}"))?;
        total_bytes_sent += bytes_read as u64;

        let percent = (total_bytes_sent as f64 / file_info.file_size as f64) * 100.0;
        tracker.set_progress(percent as usize);

        job.write()
            .expect("Failed to acquire write lock on job")
            .update_progress(&tracker);
        emit_progress(app_handle, job, false);
    }

    // Finalize upload
    ftp_stream
        .finalize_put_stream(data_stream)
        .map_err(|e| format!("failed to finalize stream: {e}"))
}

fn new_tracker() -> progress_tracker::Base {
    let options = ProgressOptions {
        total: Some(100),
        autostart: true,
        autofinish: true,
        starting_at: Some(0),
        projector_type: Some("smoothed".to_string()),
        projector_strength: Some(0.1),
        projector_at: Some(0.0),
    };
    // update the none tracker with this new one.
    progress_tracker::Base::new(Some(options))
}

/// Upload a local video file to the FTP server, preserving Plex-compatible directory structure.
///
/// # Inputs
/// - `app_handle`: Reference to the running Tauri application, used to access global state and configuration.
/// - `file_path`: Path to the local file to upload (should be the final, properly named video file).
///
/// # How to use
/// Call this function with the application handle and the path to the file you want to upload.
/// The function will:
/// 1. Connect to the FTP server using credentials/config from `AppState`.
/// 2. Create the necessary remote directory structure so Plex can parse the file after upload.
/// 3. Change to the correct remote directory.
/// 4. Upload the file in binary mode, streaming in chunks and reporting progress.
/// 5. Cleanly close the FTP connection.
///
/// # Example
/// ```text
/// // Upload a movie file to the FTP server
/// let result = upload(app_handle, Path::new("/local/path/Movies/Inception (2010)/Inception (2010).mkv")).await;
/// if let Err(e) = result {
///     eprintln!("Upload failed: {}", e);
/// }
/// ```
///
/// # Returns
/// - `Ok(())` if upload succeeds
/// - `Err(String)` if any step fails (connection, directory creation, upload, or quit)
///
/// # Notes
/// - The file must exist locally and be accessible.
/// - FTP credentials and upload path must be configured in `AppState`.
/// - The remote directory will be created if it does not exist.
/// - The function is asynchronous and should be awaited.
pub async fn upload(
    app_handle: &AppHandle,
    job: &Arc<RwLock<Job>>,
    title_video: &Arc<RwLock<TitleVideo>>,
) -> Result<(), String> {
    let state = app_handle.state::<AppState>();
    let mut ftp_stream =
        connect_to_ftp(&state).map_err(|e| format!("Failed to login and change directory {e}"))?;

    create_upload_dir(&state, &mut ftp_stream, job, title_video)?;

    start_upload(app_handle, &mut ftp_stream, job, title_video)?;

    ftp_stream
        .quit()
        .map_err(|e| format!("Failed to close or quit connection: {e}"))?;

    debug!("Upload complete.");
    Ok(())
}

pub fn cwd(ftp_stream: &mut FtpStream, path: &Path) -> Result<(), String> {
    match ftp_stream.cwd(path.to_string_lossy()) {
        Ok(_n) => Ok(()),
        Err(e) => Err(format!("failed to CWD to {} {}", path.display(), e)),
    }
}

/// List directories at a given path on the FTP server
pub fn list_directories(ftp_stream: &mut FtpStream, path: &str) -> Result<Vec<String>, String> {
    // Try to change to the directory first
    if ftp_stream.cwd(path).is_err() {
        return Err(format!("Cannot access directory: {path}"));
    }

    // Use NLST to preserve names exactly (including spaces), then probe each entry.
    // LIST parsing is unreliable for names like "TV Shows" because spacing is ambiguous.
    let list = ftp_stream
        .nlst(None)
        .map_err(|e| format!("Failed to list directory names: {e}"))?;

    // Determine which entries are directories by attempting CWD into each one.
    let mut dirs = Vec::new();
    for raw_entry in list {
        let raw_entry = raw_entry.trim();
        if raw_entry.is_empty() {
            continue;
        }

        let entry_name = raw_entry.rsplit('/').next().unwrap_or(raw_entry).trim();
        if entry_name == "." || entry_name == ".." || entry_name.is_empty() {
            continue;
        }

        if ftp_stream.cwd(entry_name).is_ok() {
            dirs.push(entry_name.to_string());
            let _ = ftp_stream.cdup();
        }
    }

    Ok(dirs)
}
