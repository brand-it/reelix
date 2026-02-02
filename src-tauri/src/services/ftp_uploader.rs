use crate::progress_tracker::{self, ProgressOptions};
use crate::state::job_state::{emit_progress, Job};
use crate::state::title_video::TitleVideo;
use crate::state::AppState;
use log::{debug, error};
use std::fs::File;
use std::io::Write;
use std::io::{BufReader, Read};
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, RwLock};
use suppaftp::types::FileType;
use suppaftp::{FtpError, FtpStream};
use tauri::{AppHandle, Manager, State};

const CHUNK_SIZE: usize = 8192;

struct FileInfo {
    file_size: u64,
    reader: BufReader<File>,
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

pub fn validate_ftp_settings(state: &State<'_, AppState>) -> Result<(), String> {
    let movie_upload_path = match state.lock_ftp_movie_upload_path().clone() {
        Some(value) => value,
        None => return Err("missing ftp movie upload path".to_string()),
    };
    let mut ftp_stream =
        connect_to_ftp(state).map_err(|e| format!("Failed to login and change directory {e}"))?;

    cwd(&mut ftp_stream, &movie_upload_path)?;
    ftp_stream
        .quit()
        .map_err(|e| format!("Failed to close connection: {e}"))?;

    Ok(())
}

/// Connects, authenticates, and Changes current directory to MOVIE_UPLOAD_PATH
fn connect_to_ftp(state: &State<'_, AppState>) -> Result<FtpStream, FtpError> {
    let ftp_host = match state.lock_ftp_host().clone() {
        Some(ftp_host) => ftp_host,
        None => {
            return Err(FtpError::ConnectionError(std::io::Error::other(
                "ftp host missing",
            )));
        }
    };
    let ftp_pass = match state.lock_ftp_pass().clone() {
        Some(ftp_pass) => ftp_pass,
        None => {
            return Err(FtpError::ConnectionError(std::io::Error::other(
                "ftp pass missing",
            )));
        }
    };
    let ftp_user = match state.lock_ftp_user().clone() {
        Some(ftp_user) => ftp_user,
        None => {
            return Err(FtpError::ConnectionError(std::io::Error::other(
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

fn create_movie_dir(
    state: &State<'_, AppState>,
    ftp_stream: &mut FtpStream,
    _job: &Arc<RwLock<Job>>,
    title_video: &Arc<RwLock<TitleVideo>>,
) -> Result<PathBuf, String> {
    let movie_dir = title_video
        .read()
        .unwrap()
        .upload_directory(state)
        .ok_or_else(|| "Failed to get movie directory".to_string())?;

    debug!("creating movie dir movie_dir={movie_dir:?}");

    ensure_remote_dir_recursive(ftp_stream, &movie_dir)?;

    Ok(movie_dir)
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

fn cwd(ftp_stream: &mut FtpStream, path: &PathBuf) -> Result<(), String> {
    debug!("CWD changing directory to {path:?}");
    match ftp_stream.cwd(path.to_string_lossy()) {
        Ok(n) => Ok(n),
        Err(e) => Err(format!("failed to CWD to {} {}", path.display(), e)),
    }
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
    let upload_file_path = title_video.read().unwrap().upload_file_path(&state);
    if upload_file_path.is_none() {
        return Err("Failed to get upload file path".to_string());
    }
    let upload_file_path = upload_file_path.unwrap();
    let local_file_path = title_video.read().unwrap().video_path(&state);
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
/// ```rust,ignore
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

    create_movie_dir(&state, &mut ftp_stream, job, title_video)?;

    start_upload(app_handle, &mut ftp_stream, job, title_video)?;

    ftp_stream
        .quit()
        .map_err(|e| format!("Failed to close or quit connection: {e}"))?;

    debug!("Upload complete.");
    Ok(())
}
