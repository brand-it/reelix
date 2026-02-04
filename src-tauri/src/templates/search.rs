use crate::models::optical_disk_info;
use crate::services::auto_complete::suggestion;
use crate::services::plex::search_multi;
use crate::state::background_process_state::BackgroundProcessState;
use crate::state::job_state::Job;
use crate::state::AppState;
use crate::templates::disks::DisksOptions;
use crate::templates::jobs::{
    JobsCompletedItem, JobsCompletedSection, JobsContainer, JobsItem, JobsItemDetails,
    JobsItemSummary,
};
use crate::templates::{the_movie_db, GenericError, InlineTemplate};
use crate::the_movie_db::SearchResponse;
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
    pub disks_toast_progress: &'a JobsContainer<'a>,
}

impl<'a> SearchIndex<'a> {
    pub fn dom_id(&self) -> &'static str {
        super::INDEX_ID
    }

    #[cfg(target_os = "macos")]
    pub fn search_shortcut(&self) -> &'a str {
        "⌘+F"
    }

    #[cfg(not(target_os = "macos"))]
    pub fn search_shortcut(&self) -> &'a str {
        "Ctrl+F"
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

    // Get all jobs from background_process_state
    let jobs_vec: Vec<crate::state::job_state::Job> = {
        let jobs = background_process_state
            .jobs
            .read()
            .expect("lock jobs for read");
        jobs.iter()
            .map(|job_arc| {
                let job_guard = job_arc.read().expect("lock job for read");
                job_guard.clone()
            })
            .collect()
    };

    let job = match &selected_disk {
        Some(disk) => jobs_vec
            .iter()
            .find(|j| j.disk.as_ref().map(|d| d.id) == Some(disk.id))
            .cloned(),
        None => None,
    };

    let disks_options = DisksOptions {
        optical_disks: &app_state.clone_optical_disks(),
        selected_disk: &selected_disk,
        job: &job,
    };

    let background_process_state = app_handle.state::<BackgroundProcessState>();
    let jobs = background_process_state.clone_all_jobs();
    let mut sorted_jobs: Vec<&Job> = jobs.iter().collect();
    sorted_jobs.sort_by(|a, b| b.id.cmp(&a.id));

    let summaries: Vec<JobsItemSummary> = sorted_jobs
        .iter()
        .filter(|job| !job.is_completed())
        .map(|job| JobsItemSummary { job })
        .collect();

    let details: Vec<JobsItemDetails> = sorted_jobs
        .iter()
        .filter(|job| !job.is_completed())
        .map(|job| JobsItemDetails { job })
        .collect();

    let items: Vec<JobsItem> = sorted_jobs
        .iter()
        .filter(|job| !job.is_completed())
        .enumerate()
        .map(|(index, job)| JobsItem {
            job,
            summary: &summaries[index],
            details: &details[index],
        })
        .collect();

    let completed_jobs: Vec<&Job> = sorted_jobs
        .iter()
        .copied()
        .filter(|job| job.is_completed())
        .collect();

    let completed_items: Vec<JobsCompletedItem> = completed_jobs
        .iter()
        .map(|job| JobsCompletedItem { job })
        .collect();

    let success_count = completed_jobs
        .iter()
        .filter(|job| job.is_finished())
        .count();
    let failure_count = completed_jobs.iter().filter(|job| job.is_error()).count();

    let completed_section = JobsCompletedSection {
        items: &completed_items,
        success_count,
        failure_count,
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
            disks_toast_progress: &JobsContainer {
                items: &items,
                completed: &completed_section,
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
