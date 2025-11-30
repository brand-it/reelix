use crate::models::movie_db::SearchResponse;
use crate::models::optical_disk_info;
use crate::services::auto_complete::suggestion;
use crate::services::plex::search_multi;
use crate::state::background_process_state::{copy_job_state, BackgroundProcessState};
use crate::state::job_state::JobStatus;
use crate::state::AppState;
use crate::templates::disks::{
    DisksOptions, DisksToastProgress, DisksToastProgressDetails, DisksToastProgressSummary,
};
use crate::templates::{the_movie_db, GenericError, InlineTemplate};
use askama::Template;
use tauri::Manager;

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
    pub suggestion: &'a SearchSuggestion<'a>,
    pub search_results: &'a SearchResults<'a>,
    pub generic_error: &'a GenericError<'a>,
    pub disks_toast_progress: &'a DisksToastProgress<'a>,
}

impl<'a> SearchIndex<'a> {
    pub fn dom_id(&self) -> &'static str {
        super::INDEX_ID
    }
}

#[derive(Template)]
#[template(path = "search/suggestion.html")]
pub struct SearchSuggestion<'a> {
    pub query: &'a str,
    pub suggestion: &'a Option<String>,
}

impl SearchSuggestion<'_> {
    pub fn dom_id(&self) -> &'static str {
        super::SEARCH_SUGGESTION_ID
    }
}

#[derive(Template)]
#[template(path = "search/suggestion.turbo.html")]
pub struct SearchSuggestionTurbo<'a> {
    pub search_suggestion: &'a SearchSuggestion<'a>,
}

#[derive(Template)]
#[template(path = "search/results.html")]
pub struct SearchResults<'a> {
    pub query: &'a str,
    pub search: &'a SearchResponse,
}
impl SearchResults<'_> {
    pub fn dom_id(&self) -> &'static str {
        super::SEARCH_RESULTS_ID
    }
}

#[derive(Template)]
#[template(path = "search/results.turbo.html")]
pub struct SearchResultsTurbo<'a> {
    pub search_results: &'a SearchResults<'a>,
}

pub fn render_index(app_handle: &tauri::AppHandle) -> Result<String, super::Error> {
    let app_state = app_handle.state::<AppState>();
    let background_process_state = app_handle.state::<BackgroundProcessState>();
    let query = app_state.query.lock().unwrap().to_string();
    let search = match search_multi(&app_state, &query) {
        Ok(resp) => resp,
        Err(e) => return the_movie_db::render_index(&app_state, &e.message),
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
    let job = match &selected_disk {
        Some(disk) => background_process_state
            .find_job(
                Some(disk.id),
                &None,
                &[
                    JobStatus::Pending,
                    JobStatus::Ready,
                    JobStatus::Processing,
                    JobStatus::Finished,
                    JobStatus::Error,
                ],
            )
            .and_then(|j| copy_job_state(&Some(j))),
        None => None,
    };
    let disks_options = DisksOptions {
        optical_disks: &app_state.clone_optical_disks(),
        selected_disk: &selected_disk,
        job: &job,
    };

    let template = SearchIndexTurbo {
        search_index: &SearchIndex {
            disks_options: &disks_options,
            query: &query,
            suggestion: &SearchSuggestion {
                query: &query,
                suggestion: &suggestion,
            },
            search_results: &SearchResults {
                query: &query,
                search: &search,
            },
            generic_error: &GenericError { message: "" },
            disks_toast_progress: &DisksToastProgress {
                disks_toast_progress_details: &DisksToastProgressDetails { job: &job },
                disks_toast_progress_summary: &DisksToastProgressSummary { job: &job },
            },
        },
    };
    super::render(template)
}

pub fn render_results(query: &str, search: &SearchResponse) -> Result<String, super::Error> {
    let template = SearchResultsTurbo {
        search_results: &SearchResults { query, search },
    };
    super::render(template)
}

pub async fn render_suggestion(
    query: &str,
    suggestion: &Option<String>,
) -> Result<String, super::Error> {
    let template = SearchSuggestionTurbo {
        search_suggestion: &SearchSuggestion { query, suggestion },
    };
    super::render(template)
}
