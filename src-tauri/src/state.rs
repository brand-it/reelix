use crate::models::optical_disk_info::{DiskId, OpticalDiskInfo};
use crate::services::ftp_validator;
use log::debug;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, RwLock};
use tauri_plugin_store::StoreExt;

pub mod background_process_state;
pub mod job_state;
pub mod title_video;
pub mod upload_state;
pub mod uploaded_state;

#[derive(Clone)]
pub struct FtpConfig {
    pub host: Option<String>,
    pub movie_upload_path: Option<PathBuf>,
    pub tv_upload_path: Option<PathBuf>,
    pub pass: Option<String>,
    pub user: Option<String>,
    pub checker: ftp_validator::FtpChecker,
}

impl FtpConfig {
    pub fn new() -> Self {
        Self {
            host: None,
            user: None,
            pass: None,
            movie_upload_path: None,
            tv_upload_path: None,
            checker: ftp_validator::FtpChecker::new(),
        }
    }

    pub fn is_configured(&self) -> bool {
        self.host.is_some() && self.user.is_some() && self.pass.is_some()
    }
}

impl PartialEq for FtpConfig {
    fn eq(&self, other: &Self) -> bool {
        self.host == other.host
            && self.user == other.user
            && self.pass == other.pass
            && self.movie_upload_path == other.movie_upload_path
            && self.tv_upload_path == other.tv_upload_path
    }
}

impl Eq for FtpConfig {}

pub struct FtpHostGuard<'a>(MutexGuard<'a, FtpConfig>);

impl<'a> std::ops::Deref for FtpHostGuard<'a> {
    type Target = Option<String>;

    fn deref(&self) -> &Self::Target {
        &self.0.host
    }
}

impl<'a> std::ops::DerefMut for FtpHostGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0.host
    }
}

pub struct FtpUserGuard<'a>(MutexGuard<'a, FtpConfig>);

impl<'a> std::ops::Deref for FtpUserGuard<'a> {
    type Target = Option<String>;

    fn deref(&self) -> &Self::Target {
        &self.0.user
    }
}

impl<'a> std::ops::DerefMut for FtpUserGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0.user
    }
}

pub struct FtpPassGuard<'a>(MutexGuard<'a, FtpConfig>);

impl<'a> std::ops::Deref for FtpPassGuard<'a> {
    type Target = Option<String>;

    fn deref(&self) -> &Self::Target {
        &self.0.pass
    }
}

impl<'a> std::ops::DerefMut for FtpPassGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0.pass
    }
}

pub struct FtpMovieUploadPathGuard<'a>(MutexGuard<'a, FtpConfig>);

impl<'a> std::ops::Deref for FtpMovieUploadPathGuard<'a> {
    type Target = Option<PathBuf>;

    fn deref(&self) -> &Self::Target {
        &self.0.movie_upload_path
    }
}

impl<'a> std::ops::DerefMut for FtpMovieUploadPathGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0.movie_upload_path
    }
}

pub struct FtpTvUploadPathGuard<'a>(MutexGuard<'a, FtpConfig>);

impl<'a> std::ops::Deref for FtpTvUploadPathGuard<'a> {
    type Target = Option<PathBuf>;

    fn deref(&self) -> &Self::Target {
        &self.0.tv_upload_path
    }
}

impl<'a> std::ops::DerefMut for FtpTvUploadPathGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0.tv_upload_path
    }
}

// Structure to hold shared state, thread safe version
pub struct AppState {
    pub ftp_config: Arc<Mutex<FtpConfig>>,
    pub optical_disks: Arc<RwLock<Vec<Arc<RwLock<OpticalDiskInfo>>>>>,
    pub query: Arc<Mutex<String>>,
    pub selected_optical_disk_id: Arc<RwLock<Option<DiskId>>>,
    pub the_movie_db_key: Arc<Mutex<String>>,
    pub movies_dir: Arc<RwLock<PathBuf>>,
    pub tv_shows_dir: Arc<RwLock<PathBuf>>,
    pub current_video: Arc<Mutex<Option<title_video::Video>>>,
    pub latest_version: Arc<Mutex<Option<String>>>,
}

impl AppState {
    const STORE_NAME: &'static str = "store.json";

    pub fn new() -> Self {
        Self {
            current_video: Arc::new(Mutex::new(None)),
            ftp_config: Arc::new(Mutex::new(FtpConfig::new())),
            latest_version: Arc::new(Mutex::new(None)),
            movies_dir: Arc::new(RwLock::new(Self::default_movies_dir())),
            optical_disks: Arc::new(RwLock::new(Vec::<Arc<RwLock<OpticalDiskInfo>>>::new())),
            query: Arc::new(Mutex::new(String::new())),
            selected_optical_disk_id: Arc::new(RwLock::new(None)),
            the_movie_db_key: Arc::new(Mutex::new(String::new())),
            tv_shows_dir: Arc::new(RwLock::new(Self::default_tv_shows_dir())),
        }
    }

    /// Load state from the persistent store file
    pub fn load_from_store(&self, app_handle: &tauri::AppHandle) -> Result<(), String> {
        let store = app_handle
            .store(Self::STORE_NAME)
            .map_err(|e| format!("Failed to load store: {e}"))?;

        for key in store.keys() {
            if let Some(value) = store.get(&key) {
                if let Some(value_str) = value.as_str() {
                    // Load values directly without triggering save
                    let cleaned: Option<String> = if value_str.trim().is_empty() {
                        None
                    } else {
                        Some(value_str.trim().to_string())
                    };

                    match key.as_str() {
                        "ftp_host" => {
                            let mut ftp_config = self.lock_ftp_config();
                            ftp_config.host = cleaned;
                        }
                        "ftp_user" => {
                            let mut ftp_config = self.lock_ftp_config();
                            ftp_config.user = cleaned;
                        }
                        "ftp_pass" => {
                            let mut ftp_config = self.lock_ftp_config();
                            ftp_config.pass = cleaned;
                        }
                        "ftp_movie_upload_path" => {
                            let mut ftp_config = self.lock_ftp_config();
                            ftp_config.movie_upload_path = cleaned.map(PathBuf::from);
                        }
                        "ftp_tv_upload_path" => {
                            let mut ftp_config = self.lock_ftp_config();
                            ftp_config.tv_upload_path = cleaned.map(PathBuf::from);
                        }
                        "the_movie_db_key" => {
                            if let Some(val) = cleaned {
                                let mut the_movie_db_key = self.lock_the_movie_db_key();
                                *the_movie_db_key = val;
                            }
                        }
                        "movies_dir" => {
                            if let Some(val) = cleaned {
                                let path = PathBuf::from(&val);
                                if path.exists() {
                                    let mut movies_dir =
                                        self.movies_dir.write().expect("failed to lock movies_dir");
                                    *movies_dir = path;
                                } else {
                                    debug!("Skipping movies_dir load: path does not exist: {val}");
                                }
                            }
                        }
                        "tv_shows_dir" => {
                            if let Some(val) = cleaned {
                                let path = PathBuf::from(&val);
                                if path.exists() {
                                    let mut tv_shows_dir = self
                                        .tv_shows_dir
                                        .write()
                                        .expect("failed to lock tv_shows_dir");
                                    *tv_shows_dir = path;
                                } else {
                                    debug!(
                                        "Skipping tv_shows_dir load: path does not exist: {val}"
                                    );
                                }
                            }
                        }
                        "latest_version" => {
                            let mut lv = self.latest_version.lock().unwrap();
                            *lv = cleaned;
                        }
                        _ => debug!("Unknown key in store: {key}"),
                    }
                    debug!("Loaded key from store: {key}");
                }
            }
        }

        Ok(())
    }

    /// Save current state to the persistent store file
    pub fn save(&self, app_handle: &tauri::AppHandle) -> Result<(), String> {
        use tauri_plugin_store::StoreExt;

        let store = app_handle
            .store(Self::STORE_NAME)
            .map_err(|e| format!("Failed to load store: {e}"))?;

        let ftp_config = self.lock_ftp_config().clone();

        // Save FTP settings
        if let Some(ref host) = ftp_config.host {
            store.set("ftp_host", serde_json::json!(host));
        } else {
            store.delete("ftp_host");
        }
        if let Some(ref user) = ftp_config.user {
            store.set("ftp_user", serde_json::json!(user));
        } else {
            store.delete("ftp_user");
        }
        if let Some(ref pass) = ftp_config.pass {
            store.set("ftp_pass", serde_json::json!(pass));
        } else {
            store.delete("ftp_pass");
        }
        if let Some(ref path) = ftp_config.movie_upload_path {
            if let Some(path_str) = path.to_str() {
                store.set("ftp_movie_upload_path", serde_json::json!(path_str));
            }
        } else {
            store.delete("ftp_movie_upload_path");
        }
        if let Some(ref path) = ftp_config.tv_upload_path {
            if let Some(path_str) = path.to_str() {
                store.set("ftp_tv_upload_path", serde_json::json!(path_str));
            }
        } else {
            store.delete("ftp_tv_upload_path");
        }

        // Save The Movie DB key
        let tmdb_key = self.lock_the_movie_db_key();
        if !tmdb_key.is_empty() {
            store.set("the_movie_db_key", serde_json::json!(tmdb_key.as_str()));
        }

        // Save directory paths
        let movies_dir = self
            .movies_dir
            .read()
            .expect("failed to lock movies_dir for read");
        if let Some(path_str) = movies_dir.to_str() {
            store.set("movies_dir", serde_json::json!(path_str));
        }
        let tv_shows_dir = self
            .tv_shows_dir
            .read()
            .expect("failed to lock tv_shows_dir for read");
        if let Some(path_str) = tv_shows_dir.to_str() {
            store.set("tv_shows_dir", serde_json::json!(path_str));
        }

        // Save version info
        let latest_version_guard = self
            .latest_version
            .lock()
            .expect("failed to lock latest_version");
        if let Some(version) = latest_version_guard.as_ref() {
            store.set("latest_version", serde_json::json!(version));
        }

        store
            .save()
            .map_err(|e| format!("Failed to save store: {e}"))?;
        debug!("State saved to store successfully");
        Ok(())
    }

    pub fn save_current_video(&self, video: Option<title_video::Video>) {
        let mut guard = self
            .current_video
            .lock()
            .expect("failed to lock current_video");
        *guard = video;
    }

    pub fn save_query(&self, search: &str) {
        let mut query = self.query.lock().unwrap();
        *query = search.to_string();
    }

    fn default_movies_dir() -> PathBuf {
        dirs::home_dir()
            .expect("failed to find home dir")
            .join("Movies")
    }

    fn default_tv_shows_dir() -> PathBuf {
        dirs::home_dir()
            .expect("failed to find home dir")
            .join("TV Shows")
    }

    pub fn lock_the_movie_db_key(&self) -> MutexGuard<'_, String> {
        self.the_movie_db_key
            .lock()
            .expect("failed to lock the_movie_db_key")
    }

    pub fn lock_ftp_config(&self) -> MutexGuard<'_, FtpConfig> {
        self.ftp_config.lock().expect("failed to lock ftp_config")
    }

    pub fn lock_ftp_host(&self) -> FtpHostGuard<'_> {
        FtpHostGuard(self.lock_ftp_config())
    }

    pub fn lock_ftp_user(&self) -> FtpUserGuard<'_> {
        FtpUserGuard(self.lock_ftp_config())
    }

    pub fn lock_ftp_pass(&self) -> FtpPassGuard<'_> {
        FtpPassGuard(self.lock_ftp_config())
    }

    pub fn lock_ftp_movie_upload_path(&self) -> FtpMovieUploadPathGuard<'_> {
        FtpMovieUploadPathGuard(self.lock_ftp_config())
    }

    pub fn lock_ftp_tv_upload_path(&self) -> FtpTvUploadPathGuard<'_> {
        FtpTvUploadPathGuard(self.lock_ftp_config())
    }

    pub fn update_ftp_settings(
        &self,
        ftp_host: Option<String>,
        ftp_user: Option<String>,
        ftp_pass: Option<String>,
        ftp_movie_upload_path: Option<String>,
        ftp_tv_upload_path: Option<String>,
    ) {
        let clean = |value: Option<String>| {
            value.and_then(|s| {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            })
        };

        let mut ftp_config = self.lock_ftp_config();
        ftp_config.host = clean(ftp_host);
        ftp_config.user = clean(ftp_user);
        ftp_config.pass = clean(ftp_pass);
        ftp_config.movie_upload_path = clean(ftp_movie_upload_path).map(PathBuf::from);
        ftp_config.tv_upload_path = clean(ftp_tv_upload_path).map(PathBuf::from);
    }

    pub fn update(
        &self,
        app_handle: &tauri::AppHandle,
        key: &str,
        value: Option<String>,
    ) -> Result<(), String> {
        let cleaned: Option<String> = value.and_then(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });
        debug!("Updating State {key} {cleaned:?}");
        match key {
            "ftp_host" => {
                let mut ftp_host = self.lock_ftp_host();
                *ftp_host = cleaned;
            }
            "ftp_user" => {
                let mut ftp_user = self.lock_ftp_user();
                *ftp_user = cleaned;
            }
            "ftp_pass" => {
                let mut ftp_pass = self.lock_ftp_pass();
                *ftp_pass = cleaned;
            }
            "ftp_movie_upload_path" => {
                let mut ftp_movie_upload_path = self.lock_ftp_movie_upload_path();
                *ftp_movie_upload_path = cleaned.map(PathBuf::from);
            }
            "ftp_tv_upload_path" => {
                let mut ftp_tv_upload_path = self.lock_ftp_tv_upload_path();
                *ftp_tv_upload_path = cleaned.map(PathBuf::from);
            }
            "the_movie_db_key" => {
                if let Some(val) = cleaned {
                    let mut the_movie_db_key = self.lock_the_movie_db_key();
                    *the_movie_db_key = val;
                };
            }
            "movies_dir" => {
                if let Some(val) = cleaned {
                    let mut movies_dir = self
                        .movies_dir
                        .write()
                        .expect("failed to lock movies_dir for write");
                    // validate path exists
                    if !movies_dir.exists() {
                        return Err(format!("movies_dir path does not exist: {val}"));
                    }
                    *movies_dir = PathBuf::from(val);
                };
            }
            "tv_shows_dir" => {
                if let Some(val) = cleaned {
                    let mut tv_shows_dir = self
                        .tv_shows_dir
                        .write()
                        .expect("failed to lock tv_shows_dir for write");
                    // validate path exists
                    if !tv_shows_dir.exists() {
                        return Err(format!("tv_shows_dir path does not exist: {val}"));
                    }
                    *tv_shows_dir = PathBuf::from(val);
                };
            }
            "latest_version" => {
                let mut lv = self.latest_version.lock().unwrap();
                *lv = cleaned;
            }
            _ => return Err(format!("can't update {key}")),
        }

        // Automatically persist the change to the store
        self.save(app_handle)?;
        Ok(())
    }

    pub fn clone_optical_disks(&self) -> Vec<OpticalDiskInfo> {
        let guard = self.optical_disks.read().unwrap();
        guard
            .iter()
            .map(|disk_arc| disk_arc.read().unwrap().to_owned())
            .collect()
    }

    pub fn selected_disk(&self) -> Option<Arc<RwLock<OpticalDiskInfo>>> {
        let disk_id = self
            .selected_optical_disk_id
            .read()
            .expect("failed to lock selected_optical_disk_id in find_selected_disk");
        match disk_id.as_ref() {
            Some(disk_id) => self.find_optical_disk_by_id(disk_id),
            None => None,
        }
    }

    pub fn find_optical_disk_by_id(
        &self,
        disk_id: &DiskId,
    ) -> Option<Arc<RwLock<OpticalDiskInfo>>> {
        let disks = self
            .optical_disks
            .read()
            .expect("Failed to acquire lock on optical_disks in find_optical_disk_by_id command");
        for disk in disks.iter() {
            let disk_guard = disk
                .read()
                .expect("Failed to acquire lock on disk in find_optical_disk_by_id command");
            if &disk_guard.id == disk_id {
                return Some(Arc::clone(disk));
            }
        }
        None
    }

    pub fn get_version_state(
        &self,
        app_handle: &tauri::AppHandle,
    ) -> crate::services::version_checker::VersionState {
        let current_version_str = app_handle.package_info().version.to_string();
        let current_version =
            crate::services::semantic_version::SemanticVersion::parse(&current_version_str)
                .unwrap_or_else(|_| crate::services::semantic_version::SemanticVersion::none());
        let latest_version = self
            .latest_version
            .lock()
            .expect("failed to lock latest_version")
            .clone()
            .and_then(|v| crate::services::semantic_version::SemanticVersion::parse(&v).ok())
            .unwrap_or_else(crate::services::semantic_version::SemanticVersion::none);

        crate::services::version_checker::VersionState::new(current_version, latest_version)
    }

    // Helper method for tests to update FTP settings without needing app_handle
    #[cfg(test)]
    pub fn test_update_ftp_setting(&self, key: &str, value: Option<String>) {
        let cleaned: Option<String> = value.and_then(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });

        match key {
            "ftp_host" => {
                let mut ftp_host = self.lock_ftp_host();
                *ftp_host = cleaned;
            }
            "ftp_user" => {
                let mut ftp_user = self.lock_ftp_user();
                *ftp_user = cleaned;
            }
            "ftp_pass" => {
                let mut ftp_pass = self.lock_ftp_pass();
                *ftp_pass = cleaned;
            }
            "ftp_movie_upload_path" => {
                let mut ftp_movie_upload_path = self.lock_ftp_movie_upload_path();
                *ftp_movie_upload_path = cleaned.map(PathBuf::from);
            }
            "ftp_tv_upload_path" => {
                let mut ftp_tv_upload_path = self.lock_ftp_tv_upload_path();
                *ftp_tv_upload_path = cleaned.map(PathBuf::from);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_converts_empty_string_to_none() {
        let state = AppState::new();

        // Set initial value
        state.test_update_ftp_setting("ftp_host", Some("example.com".to_string()));
        assert_eq!(*state.lock_ftp_host(), Some("example.com".to_string()));

        // Update to empty string
        state.test_update_ftp_setting("ftp_host", Some("".to_string()));
        assert_eq!(*state.lock_ftp_host(), None);
    }

    #[test]
    fn test_update_trims_whitespace() {
        let state = AppState::new();

        // Update with whitespace-only string
        state.test_update_ftp_setting("ftp_host", Some("   ".to_string()));
        assert_eq!(*state.lock_ftp_host(), None);
    }

    #[test]
    fn test_update_preserves_valid_values() {
        let state = AppState::new();

        // Test FTP host
        state.test_update_ftp_setting("ftp_host", Some("ftp.example.com".to_string()));
        assert_eq!(*state.lock_ftp_host(), Some("ftp.example.com".to_string()));

        // Test FTP user
        state.test_update_ftp_setting("ftp_user", Some("testuser".to_string()));
        assert_eq!(*state.lock_ftp_user(), Some("testuser".to_string()));

        // Test FTP pass
        state.test_update_ftp_setting("ftp_pass", Some("password123".to_string()));
        assert_eq!(*state.lock_ftp_pass(), Some("password123".to_string()));
    }

    #[test]
    fn test_update_all_ftp_fields() {
        let state = AppState::new();

        // Update all FTP fields
        state.test_update_ftp_setting("ftp_host", Some("ftp.example.com".to_string()));
        state.test_update_ftp_setting("ftp_user", Some("user".to_string()));
        state.test_update_ftp_setting("ftp_pass", Some("pass".to_string()));
        state.test_update_ftp_setting("ftp_movie_upload_path", Some("/movies".to_string()));

        assert_eq!(*state.lock_ftp_host(), Some("ftp.example.com".to_string()));
        assert_eq!(*state.lock_ftp_user(), Some("user".to_string()));
        assert_eq!(*state.lock_ftp_pass(), Some("pass".to_string()));
        assert_eq!(
            *state.lock_ftp_movie_upload_path(),
            Some(PathBuf::from("/movies"))
        );
    }

    #[test]
    fn test_update_then_clear_ftp_host() {
        let state = AppState::new();

        // Set a value
        state.test_update_ftp_setting("ftp_host", Some("ftp.example.com".to_string()));
        assert_eq!(*state.lock_ftp_host(), Some("ftp.example.com".to_string()));

        // Clear it
        state.test_update_ftp_setting("ftp_host", Some("".to_string()));
        assert_eq!(*state.lock_ftp_host(), None);
    }

    #[test]
    fn test_multiple_clear_and_set_cycles() {
        let state = AppState::new();

        // Cycle 1: Set, then clear
        state.test_update_ftp_setting("ftp_host", Some("host1.com".to_string()));
        assert_eq!(*state.lock_ftp_host(), Some("host1.com".to_string()));
        state.test_update_ftp_setting("ftp_host", Some("".to_string()));
        assert_eq!(*state.lock_ftp_host(), None);

        // Cycle 2: Set different value, then clear
        state.test_update_ftp_setting("ftp_host", Some("host2.com".to_string()));
        assert_eq!(*state.lock_ftp_host(), Some("host2.com".to_string()));
        state.test_update_ftp_setting("ftp_host", Some("".to_string()));
        assert_eq!(*state.lock_ftp_host(), None);
    }

    #[test]
    fn test_update_with_tabs_and_newlines() {
        let state = AppState::new();

        // Test with tabs and newlines
        state.test_update_ftp_setting("ftp_host", Some("\t\n  \r\n".to_string()));
        assert_eq!(*state.lock_ftp_host(), None);
    }

    #[test]
    fn test_update_path_field() {
        let state = AppState::new();

        // Set path
        state.test_update_ftp_setting("ftp_movie_upload_path", Some("/mnt/movies".to_string()));
        assert_eq!(
            *state.lock_ftp_movie_upload_path(),
            Some(PathBuf::from("/mnt/movies"))
        );

        // Clear path
        state.test_update_ftp_setting("ftp_movie_upload_path", Some("".to_string()));
        assert_eq!(*state.lock_ftp_movie_upload_path(), None);
    }

    // NEW COMPREHENSIVE TESTS FOR ALL UPDATE FIELDS

    #[test]
    fn test_update_ftp_host_field() {
        let state = AppState::new();

        // Test setting value
        state.test_update_ftp_setting("ftp_host", Some("192.168.1.100".to_string()));
        assert_eq!(*state.lock_ftp_host(), Some("192.168.1.100".to_string()));

        // Test updating value
        state.test_update_ftp_setting("ftp_host", Some("ftp.newserver.com".to_string()));
        assert_eq!(
            *state.lock_ftp_host(),
            Some("ftp.newserver.com".to_string())
        );

        // Test clearing value
        state.test_update_ftp_setting("ftp_host", None);
        assert_eq!(*state.lock_ftp_host(), None);
    }

    #[test]
    fn test_update_ftp_user_field() {
        let state = AppState::new();

        state.test_update_ftp_setting("ftp_user", Some("admin".to_string()));
        assert_eq!(*state.lock_ftp_user(), Some("admin".to_string()));

        state.test_update_ftp_setting("ftp_user", Some("  spaced_user  ".to_string()));
        assert_eq!(*state.lock_ftp_user(), Some("spaced_user".to_string()));

        state.test_update_ftp_setting("ftp_user", Some("".to_string()));
        assert_eq!(*state.lock_ftp_user(), None);
    }

    #[test]
    fn test_update_ftp_pass_field() {
        let state = AppState::new();

        state.test_update_ftp_setting("ftp_pass", Some("SecureP@ssw0rd!".to_string()));
        assert_eq!(*state.lock_ftp_pass(), Some("SecureP@ssw0rd!".to_string()));

        // Test that password with spaces is trimmed
        state.test_update_ftp_setting("ftp_pass", Some("  password123  ".to_string()));
        assert_eq!(*state.lock_ftp_pass(), Some("password123".to_string()));
    }

    #[test]
    fn test_update_ftp_movie_upload_path_field() {
        let state = AppState::new();

        // Test absolute paths
        state.test_update_ftp_setting("ftp_movie_upload_path", Some("/Media/Movies".to_string()));
        assert_eq!(
            *state.lock_ftp_movie_upload_path(),
            Some(PathBuf::from("/Media/Movies"))
        );

        // Test relative paths
        state.test_update_ftp_setting("ftp_movie_upload_path", Some("movies/folder".to_string()));
        assert_eq!(
            *state.lock_ftp_movie_upload_path(),
            Some(PathBuf::from("movies/folder"))
        );

        // Test clearing
        state.test_update_ftp_setting("ftp_movie_upload_path", Some("".to_string()));
        assert_eq!(*state.lock_ftp_movie_upload_path(), None);
    }

    #[test]
    fn test_update_ftp_tv_upload_path_field() {
        let state = AppState::new();

        // THIS IS THE CRITICAL TEST - previously this field wasn't being updated!
        state.test_update_ftp_setting("ftp_tv_upload_path", Some("/Media/TV Shows".to_string()));
        assert_eq!(
            *state.lock_ftp_tv_upload_path(),
            Some(PathBuf::from("/Media/TV Shows"))
        );

        // Test updating to different path
        state.test_update_ftp_setting("ftp_tv_upload_path", Some("/mnt/tv".to_string()));
        assert_eq!(
            *state.lock_ftp_tv_upload_path(),
            Some(PathBuf::from("/mnt/tv"))
        );

        // Test with spaces in path (common for TV Shows)
        state.test_update_ftp_setting(
            "ftp_tv_upload_path",
            Some("/Media/TV Shows/Anime".to_string()),
        );
        assert_eq!(
            *state.lock_ftp_tv_upload_path(),
            Some(PathBuf::from("/Media/TV Shows/Anime"))
        );

        // Test clearing
        state.test_update_ftp_setting("ftp_tv_upload_path", None);
        assert_eq!(*state.lock_ftp_tv_upload_path(), None);
    }

    #[test]
    fn test_update_all_ftp_paths_independently() {
        let state = AppState::new();

        // Set both paths
        state.test_update_ftp_setting("ftp_movie_upload_path", Some("/movies".to_string()));
        state.test_update_ftp_setting("ftp_tv_upload_path", Some("/tv".to_string()));

        // Verify both are set correctly
        assert_eq!(
            *state.lock_ftp_movie_upload_path(),
            Some(PathBuf::from("/movies"))
        );
        assert_eq!(*state.lock_ftp_tv_upload_path(), Some(PathBuf::from("/tv")));

        // Clear movie path, TV should remain
        state.test_update_ftp_setting("ftp_movie_upload_path", Some("".to_string()));
        assert_eq!(*state.lock_ftp_movie_upload_path(), None);
        assert_eq!(*state.lock_ftp_tv_upload_path(), Some(PathBuf::from("/tv")));

        // Clear TV path
        state.test_update_ftp_setting("ftp_tv_upload_path", Some("".to_string()));
        assert_eq!(*state.lock_ftp_tv_upload_path(), None);
    }

    #[test]
    fn test_update_the_movie_db_key_field() {
        let state = AppState::new();

        // Note: the_movie_db_key uses a different pattern - it's always a String, not Option<String>
        // So we directly set the value
        let mut key = state.lock_the_movie_db_key();
        *key = "test_api_key_12345".to_string();
        drop(key);

        assert_eq!(
            *state.lock_the_movie_db_key(),
            "test_api_key_12345".to_string()
        );
    }

    #[test]
    fn test_update_latest_version_field() {
        let state = AppState::new();

        // Set version
        let mut version = state.latest_version.lock().unwrap();
        *version = Some("1.2.3".to_string());
        drop(version);

        assert_eq!(
            *state.latest_version.lock().unwrap(),
            Some("1.2.3".to_string())
        );

        // Clear version
        let mut version = state.latest_version.lock().unwrap();
        *version = None;
        drop(version);

        assert_eq!(*state.latest_version.lock().unwrap(), None);
    }

    #[test]
    fn test_update_all_fields_at_once() {
        let state = AppState::new();

        // Set all FTP fields at once
        state.test_update_ftp_setting("ftp_host", Some("192.168.1.100".to_string()));
        state.test_update_ftp_setting("ftp_user", Some("plex".to_string()));
        state.test_update_ftp_setting("ftp_pass", Some("password".to_string()));
        state.test_update_ftp_setting("ftp_movie_upload_path", Some("/Media/Movies".to_string()));
        state.test_update_ftp_setting("ftp_tv_upload_path", Some("/Media/TV Shows".to_string()));

        // Verify all are set
        assert_eq!(*state.lock_ftp_host(), Some("192.168.1.100".to_string()));
        assert_eq!(*state.lock_ftp_user(), Some("plex".to_string()));
        assert_eq!(*state.lock_ftp_pass(), Some("password".to_string()));
        assert_eq!(
            *state.lock_ftp_movie_upload_path(),
            Some(PathBuf::from("/Media/Movies"))
        );
        assert_eq!(
            *state.lock_ftp_tv_upload_path(),
            Some(PathBuf::from("/Media/TV Shows"))
        );
    }

    #[test]
    fn test_update_ftp_tv_path_with_special_characters() {
        let state = AppState::new();

        // Test path with spaces, dashes, and unicode
        state.test_update_ftp_setting(
            "ftp_tv_upload_path",
            Some("/Media/TV-Shows/Scié-Fi & Fantasy".to_string()),
        );
        assert_eq!(
            *state.lock_ftp_tv_upload_path(),
            Some(PathBuf::from("/Media/TV-Shows/Scié-Fi & Fantasy"))
        );
    }

    #[test]
    fn test_update_empty_vs_whitespace_values() {
        let state = AppState::new();

        // Empty string should become None
        state.test_update_ftp_setting("ftp_host", Some("".to_string()));
        assert_eq!(*state.lock_ftp_host(), None);

        // Whitespace-only should become None
        state.test_update_ftp_setting("ftp_host", Some("   ".to_string()));
        assert_eq!(*state.lock_ftp_host(), None);

        // Tab and newline should become None
        state.test_update_ftp_setting("ftp_host", Some("\t\n".to_string()));
        assert_eq!(*state.lock_ftp_host(), None);
    }

    #[test]
    fn test_update_path_trimming() {
        let state = AppState::new();

        // Path with trailing/leading whitespace should be trimmed
        state.test_update_ftp_setting(
            "ftp_movie_upload_path",
            Some("  /Media/Movies  ".to_string()),
        );
        assert_eq!(
            *state.lock_ftp_movie_upload_path(),
            Some(PathBuf::from("/Media/Movies"))
        );

        // Same for TV path
        state.test_update_ftp_setting(
            "ftp_tv_upload_path",
            Some("\t/Media/TV Shows\n".to_string()),
        );
        assert_eq!(
            *state.lock_ftp_tv_upload_path(),
            Some(PathBuf::from("/Media/TV Shows"))
        );
    }
}
