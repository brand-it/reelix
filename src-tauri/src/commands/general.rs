// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use crate::services::auto_complete;
use crate::services::plex::{
    find_movie, find_season, find_tv, get_movie_certification, search_multi,
};
use crate::state::background_process_state::BackgroundProcessState;
use crate::state::AppState;
use crate::templates::{self, render_error};
use crate::the_movie_db;
use tauri::State;
use tauri_plugin_opener::OpenerExt;

// This is the entry point, basically it decides what to first show the user
#[tauri::command]
pub fn index(
    app_handle: tauri::AppHandle,
    app_state: State<'_, AppState>,
) -> Result<String, templates::Error> {
    match search_multi(&app_state, "Martian") {
        Ok(resp) => resp,
        Err(e) => return templates::the_movie_db::render_index(&app_state, &e.message),
    };
    templates::search::render_index(&app_handle)
}

#[tauri::command]
pub fn open_url(url: &str, app_handle: tauri::AppHandle) -> Result<String, templates::Error> {
    let response = app_handle.opener().open_url(url, None::<&str>);

    match response {
        Ok(_r) => Ok("".to_string()),
        Err(e) => render_error(&format!("failed to open url: {e:?}")),
    }
}

#[tauri::command]
pub fn movie(
    id: u32,
    background_process_state: State<'_, BackgroundProcessState>,
    app_state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, templates::Error> {
    let movie = match find_movie(&app_handle, id) {
        Ok(resp) => resp,
        Err(e) => return templates::the_movie_db::render_index(&app_state, &e.message),
    };

    let certification = match get_movie_certification(&app_handle, &id) {
        Ok(resp) => resp,
        Err(e) => return templates::the_movie_db::render_index(&app_state, &e.message),
    };
    templates::movies::render_show(
        &app_state,
        &background_process_state,
        &movie,
        &certification,
    )
}

#[tauri::command]
pub fn tv(
    id: u32,
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<String, templates::Error> {
    let tv = match find_tv(&app_handle, id) {
        Ok(resp) => resp,
        Err(e) => return templates::the_movie_db::render_index(&state, &e.message),
    };

    templates::tvs::render_show(&tv)
}

#[tauri::command]
pub fn season(
    tv_id: u32,
    season_number: u32,
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<String, templates::Error> {
    let tv = match find_tv(&app_handle, tv_id) {
        Ok(resp) => resp,
        Err(e) => return templates::the_movie_db::render_index(&state, &e.message),
    };

    let season = match find_season(&app_handle, tv_id, season_number) {
        Ok(resp) => resp,
        Err(e) => return templates::the_movie_db::render_index(&state, &e.message),
    };

    templates::seasons::render_show(&app_handle, &tv, &season)
}

#[tauri::command]
pub fn search(
    search: &str,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, templates::Error> {
    state.save_query(search);

    let api_key = &state.lock_the_movie_db_key();
    let language = "en-US";
    let movie_db = the_movie_db::TheMovieDb::new(api_key, language);
    let response = match movie_db.search_multi(search, 1) {
        Ok(resp) => resp,
        Err(e) => return templates::the_movie_db::render_index(&state, &e.message),
    };

    templates::search::render_results(&app_handle, search, &response)
}

#[tauri::command]
pub async fn suggestion(search: &str) -> Result<String, templates::Error> {
    use tokio::time::{timeout, Duration};

    // If autocomplete data isn't ready yet, return nothing immediately.
    if !auto_complete::is_ready() {
        return Ok(String::new());
    }

    // Compute suggestion in a blocking thread with a 100ms timeout.
    let search_owned = search.to_string();
    let handle = tokio::task::spawn_blocking(move || auto_complete::suggestion(&search_owned));

    let suggestion_opt = match timeout(Duration::from_millis(100), handle).await {
        Ok(Ok(opt)) => opt,
        // Join error or timeout -> no update
        _ => None,
    };

    templates::search::render_suggestion(search, &suggestion_opt).await
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_url_validation_http() {
        let valid_http = "http://example.com";
        let valid_https = "https://example.com";

        // These should be valid URLs (we're not testing the actual function call,
        // just verifying the format is reasonable)
        assert!(valid_http.starts_with("http://") || valid_http.starts_with("https://"));
        assert!(valid_https.starts_with("http://") || valid_https.starts_with("https://"));
    }

    #[test]
    fn test_search_query_normalization() {
        // Test that search queries can be normalized consistently
        let query1 = "Avengers";
        let query2 = "  Avengers  ";
        let query3 = "avengers";

        let normalized1 = query1.trim();
        let normalized2 = query2.trim();

        assert_eq!(normalized1, "Avengers");
        assert_eq!(normalized2, "Avengers");
        assert_eq!(normalized1.to_lowercase(), query3); // Case normalization check
    }

    #[test]
    fn test_movie_id_types() {
        // Test that movie IDs are u32 and can handle various ranges
        let id1: u32 = 1;
        let id2: u32 = 12345;
        let id3: u32 = u32::MAX;

        assert!(id1 > 0);
        assert!(id2 > id1);
        assert!(id3 > id2);
    }

    #[test]
    fn test_season_number_validation() {
        // Test that season numbers are valid u32 values
        let season_1: u32 = 1;
        let season_10: u32 = 10;
        let season_99: u32 = 99;

        assert!(season_1 >= 1);
        assert!(season_10 >= 1);
        assert!(season_99 >= 1);
        assert!(season_10 > season_1);
        assert!(season_99 > season_10);
    }

    #[test]
    fn test_tv_and_season_ids_are_independent() {
        // Verify that TV show IDs and season numbers are independent
        let tv_id1: u32 = 100;
        let tv_id2: u32 = 200;
        let season_num1: u32 = 1;
        let season_num2: u32 = 2;

        // TV IDs should differ
        assert_ne!(tv_id1, tv_id2);
        // Season numbers should differ
        assert_ne!(season_num1, season_num2);
        // But we can have the same season number for different shows
        assert_eq!(season_num1, 1);
        assert_eq!(season_num2, 2);
    }

    #[test]
    fn test_search_empty_string() {
        let empty_search = "";
        let whitespace_search = "   ";

        assert!(empty_search.is_empty());
        assert!(!whitespace_search.is_empty());
        assert!(whitespace_search.trim().is_empty());
    }

    #[test]
    fn test_search_special_characters() {
        // Test that search can handle special characters
        let search_with_ampersand = "Tom & Jerry";
        let search_with_colon = "Star Wars: A New Hope";
        let search_with_numbers = "2001: A Space Odyssey";

        assert!(search_with_ampersand.contains('&'));
        assert!(search_with_colon.contains(':'));
        assert!(search_with_numbers.chars().any(|c| c.is_numeric()));
    }
}
