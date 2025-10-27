use crate::models::movie_db::SearchResponse;
use crate::models::optical_disk_info;
use crate::services::auto_complete::suggestion;
use crate::services::plex::search_multi;
use crate::state::AppState;
use crate::templates::disks::DisksOptions;
use crate::templates::{the_movie_db, InlineTemplate};
use askama::Template;
use tauri::State;

#[derive(Template)]
#[template(path = "search/index.turbo.html")]
pub struct SearchIndexTurbo<'a> {
    pub search_index: &'a SearchIndex<'a>,
}

#[derive(Template)]
#[template(path = "search/index.html")]
pub struct SearchIndex<'a> {
    pub disks_options: &'a DisksOptions<'a>,
    pub query: &'a str,
    pub search: &'a SearchResponse,
    pub suggestion: &'a SearchSuggestion<'a>,
    pub search_results: &'a SearchResults<'a>,
}

#[derive(Template)]
#[template(path = "search/suggestion.html")]
pub struct SearchSuggestion<'a> {
    pub query: &'a str,
    pub suggestion: &'a Option<String>,
}

#[derive(Template)]
#[template(path = "search/results.html")]
pub struct SearchResults<'a> {
    pub query: &'a str,
    pub search: &'a SearchResponse,
}

#[derive(Template)]
#[template(path = "search/results.turbo.html")]
pub struct SearchResultsTurbo<'a> {
    pub search_results: &'a SearchResults<'a>,
}

pub fn render_index(app_state: &State<'_, AppState>) -> Result<String, super::Error> {
    // let disk_option = build_disk_option(app_state);
    // let mut context = Context::from_serialize(&disk_option).unwrap();
    let query = app_state.query.lock().unwrap().to_string();
    let search = match search_multi(app_state, &query) {
        Ok(resp) => resp,
        Err(e) => return the_movie_db::render_show(app_state, &e.message),
    };
    let suggestion = suggestion(&query);
    let selected_disk: Option<optical_disk_info::OpticalDiskInfo> = match app_state.selected_disk()
    {
        Some(disk_arc) => {
            let guard = disk_arc.read().unwrap();
            Some(guard.to_owned())
        }
        None => None,
    };
    let disks_options = DisksOptions {
        optical_disks: &app_state.clone_optical_disks(),
        selected_disk: &selected_disk,
    };
    let search_index = SearchIndex {
        disks_options: &disks_options,
        query: &query,
        search: &search,
        suggestion: &SearchSuggestion {
            query: &query,
            suggestion: &suggestion,
        },
        search_results: &SearchResults {
            query: &query,
            search: &search,
        },
    };
    let template = SearchIndexTurbo {
        search_index: &search_index,
    };
    super::render(template)
}

pub fn render_results(query: &str, search: &SearchResponse) -> Result<String, super::Error> {
    let template = SearchResultsTurbo {
        search_results: &SearchResults { query, search },
    };
    super::render(template)
}

pub fn render_suggestion(query: &str, suggestion: &Option<String>) -> Result<String, super::Error> {
    let template = SearchSuggestion { query, suggestion };
    super::render(template)
}
