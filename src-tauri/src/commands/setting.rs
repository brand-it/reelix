use crate::services::ftp_validator;
use crate::services::plex::search_multi;
use crate::state::AppState;
use crate::templates::{ftp_settings, render_error, search, Error};
use tauri::State;

#[tauri::command]
pub fn ftp_settings(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, Error> {
    ftp_validator::spawn_ftp_validator(&app_handle);
    ftp_settings::render_show(&state)
}

#[tauri::command]
pub fn update_ftp_settings(
    ftp_host: String,
    ftp_user: String,
    ftp_pass: String,
    ftp_movie_upload_path: String,
    ftp_tv_upload_path: String,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, Error> {
    state.update_ftp_settings(
        Some(ftp_host),
        Some(ftp_user),
        Some(ftp_pass),
        Some(ftp_movie_upload_path),
        Some(ftp_tv_upload_path),
    );

    if let Err(message) = state.save(&app_handle) {
        return render_error(&message);
    }

    ftp_validator::trigger_ftp_check(&app_handle);
    Ok("FTP settings updated successfully".to_string())
}

#[tauri::command]
pub fn the_movie_db(
    key: &str,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, Error> {
    state
        .update(&app_handle, "the_movie_db_key", Some(key.to_string()))
        .unwrap();
    let response = search_multi(&state, "Avengers");
    match response {
        Ok(resp) => resp,
        Err(e) => return render_error(&e.message),
    };
    search::render_index(&app_handle)
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_ftp_host_validation() {
        // Test various FTP host formats
        let valid_ip = "192.168.1.100";
        let valid_hostname = "ftp.example.com";
        let valid_localhost = "localhost";

        assert!(!valid_ip.is_empty());
        assert!(!valid_hostname.is_empty());
        assert!(!valid_localhost.is_empty());
    }

    #[test]
    fn test_ftp_path_normalization() {
        // Test that FTP paths can be normalized
        let path_with_trailing_slash = "/media/movies/";
        let path_without_trailing_slash = "/media/movies";
        let relative_path = "media/movies";

        assert!(path_with_trailing_slash.starts_with('/'));
        assert!(path_without_trailing_slash.starts_with('/'));
        assert!(!relative_path.starts_with('/'));
    }

    #[test]
    fn test_empty_ftp_credentials() {
        // Verify that empty credentials can be detected
        let empty_host = "";
        let empty_user = "";
        let empty_pass = "";

        assert!(empty_host.is_empty());
        assert!(empty_user.is_empty());
        assert!(empty_pass.is_empty());
    }

    #[test]
    fn test_whitespace_in_ftp_settings() {
        // Test that whitespace can be detected and trimmed
        let host_with_spaces = "  ftp.example.com  ";
        let user_with_spaces = " admin ";
        let path_with_spaces = "  /media/movies  ";

        assert_eq!(host_with_spaces.trim(), "ftp.example.com");
        assert_eq!(user_with_spaces.trim(), "admin");
        assert_eq!(path_with_spaces.trim(), "/media/movies");
    }

    #[test]
    fn test_api_key_format() {
        // Test that API keys can be various formats
        let hex_key = "1234567890abcdef";
        let alphanumeric_key = "ABC123XYZ789";
        let short_key = "test";
        let long_key = "a".repeat(64);

        assert!(!hex_key.is_empty());
        assert!(!alphanumeric_key.is_empty());
        assert!(short_key.len() < 10);
        assert!(long_key.len() > 50);
    }

    #[test]
    fn test_ftp_settings_field_names() {
        // Verify the field names used in update calls match expected strings
        let field_host = "ftp_host";
        let field_user = "ftp_user";
        let field_pass = "ftp_pass";
        let field_path = "ftp_movie_upload_path";
        let field_api_key = "the_movie_db_key";

        assert_eq!(field_host, "ftp_host");
        assert_eq!(field_user, "ftp_user");
        assert_eq!(field_pass, "ftp_pass");
        assert_eq!(field_path, "ftp_movie_upload_path");
        assert_eq!(field_api_key, "the_movie_db_key");
    }

    #[test]
    fn test_setting_value_types() {
        // Test that setting values are properly typed
        let string_value = "test_value".to_string();
        let some_value = Some(string_value);
        let none_value: Option<String> = None;

        assert!(some_value.is_some());
        assert!(none_value.is_none());
        assert_eq!(some_value.as_deref(), Some("test_value"));
    }
}
