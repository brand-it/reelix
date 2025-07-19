use crate::services::plex::movies_dir;
use crate::state::AppState;
use std::fs::File;
use std::io::Write;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use suppaftp::types::FileType;
use suppaftp::{FtpError, FtpStream};
use tauri::State;

const CHUNK_SIZE: usize = 8192;

struct FileInfo {
    file_size: u64,
    reader: BufReader<File>,
}

pub fn validate_ftp_settings(state: &State<'_, AppState>) -> Result<(), String> {
    let movie_upload_path = match state.lock_ftp_movie_upload_path().clone() {
        Some(value) => PathBuf::from(value),
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

// Takes the relate path of where the movie was saved to and removes it from the upload_file_path
// So for example if the upload_file_path is /file/path/Movies/Aladdin (1992)/Aladdin (1992).mkv
// then it will drop /file/path/Movies & Aladdin (1992).mkv
// given me a directory path of `Aladdin (1992)`
// I then join that together with the MOVIE_UPLOAD_PATH given me this result
// /Media/Movies/Aladdin (1992)
fn relative_movie_dir(file_path: &Path) -> PathBuf {
    let upload_path = Path::new(file_path).parent().expect("Failed to get parent");
    let dir = movies_dir();
    let relative_path = upload_path
        .strip_prefix(&dir)
        .unwrap_or_else(|_| panic!("failed to strip prefix {}", dir.display()));
    relative_path.to_path_buf()
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
    let mut ftp_stream = FtpStream::connect(ftp_host)?;
    ftp_stream.login(ftp_user, ftp_pass)?;
    Ok(ftp_stream)
}

// Open the local file and capture relative info used to send the data
fn file_info(file_path: &Path) -> Result<FileInfo, String> {
    let file = match File::open(file_path) {
        Ok(f) => f,
        Err(_e) => {
            return Err(format!(
                "Failed to open file path {}",
                file_path.to_string_lossy()
            ))
        }
    };
    let metadata = match file.metadata() {
        Ok(m) => m,
        Err(_e) => {
            return Err(format!(
                "Failed to capture metadata of file {}",
                file_path.display()
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
    file_path: &Path,
) -> Result<PathBuf, String> {
    let movie_dir = relative_movie_dir(file_path);
    let movie_dir_string = movie_dir.to_string_lossy().to_string();
    let movie_upload_path = match state.lock_ftp_movie_upload_path().clone() {
        Some(value) => value,
        None => return Err("missing ftp movie upload path".to_string()),
    };
    println!("creating movie dir {file_path:?} {movie_dir_string}");
    cwd(ftp_stream, &PathBuf::from(movie_upload_path.clone()))?;

    // Check if the directory already exists
    if ftp_stream.cwd(&movie_dir_string).is_ok() {
        // Directory exists, return its path
        let existing_dir = format!("{}/{}", movie_upload_path, movie_dir.to_string_lossy());
        return Ok(Path::new(&existing_dir).to_path_buf());
    }

    ftp_stream
        .mkdir(&movie_dir_string)
        .map_err(|e| format!("failed to create dir {} {}", movie_dir.display(), e))?;
    let new_dir = format!("{}/{}", movie_upload_path, movie_dir.to_string_lossy());
    Ok(Path::new(&new_dir).to_path_buf())
}

fn cwd(ftp_stream: &mut FtpStream, path: &PathBuf) -> Result<(), String> {
    println!("CWD changing directory to {path:?}");
    match ftp_stream.cwd(path.to_string_lossy()) {
        Ok(n) => Ok(n),
        Err(e) => Err(format!("failed to CWD to {} {}", path.display(), e)),
    }
}

fn filename(filepath: &Path) -> String {
    let filename = filepath.file_name().unwrap();
    filename.to_string_lossy().to_string()
}

fn start_upload(ftp_stream: &mut FtpStream, file_path: &Path) -> Result<(), String> {
    println!(
        "Start uploading {} to {:?}",
        file_path.display(),
        ftp_stream.pwd()
    );
    let mut file_info = file_info(file_path)?;
    let filename = filename(file_path);
    println!("File name will be {filename}");
    ftp_stream
        .transfer_type(FileType::Binary)
        .expect("failed to set binary mode");

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

        // Print progress
        let percent = (total_bytes_sent as f64 / file_info.file_size as f64) * 100.0;
        println!(
            "Uploaded: {} / {} bytes ({:.2}%)",
            total_bytes_sent, file_info.file_size, percent
        );
    }

    // Finalize upload
    ftp_stream
        .finalize_put_stream(data_stream)
        .map_err(|e| format!("failed to finalize stream: {e}"))
}

// Give a file path you want to upload and it will upload that file to a location given it the same
// directory structure as it is need for plex to parse the data.
pub async fn upload(state: &State<'_, AppState>, file_path: &Path) -> Result<(), String> {
    let mut ftp_stream =
        connect_to_ftp(state).map_err(|e| format!("Failed to login and change directory {e}"))?;

    let output_dir = create_movie_dir(state, &mut ftp_stream, file_path)?;
    cwd(&mut ftp_stream, &output_dir)?;
    start_upload(&mut ftp_stream, file_path)?;

    ftp_stream
        .quit()
        .map_err(|e| format!("Failed to close or quit connection: {e}"))?;

    println!("Upload complete.");
    Ok(())
}
