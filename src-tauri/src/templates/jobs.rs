use crate::state::job_state::Job;
use crate::templates::InlineTemplate;
use askama::Template;

#[derive(Template)]
#[template(path = "jobs/container.html")]
pub struct JobsContainer<'a> {
    pub items: &'a [JobsItem<'a>],
    pub completed: &'a JobsCompletedSection<'a>,
}

impl<'a> JobsContainer<'a> {
    pub fn dom_id(&self) -> &'static str {
        "jobs-container"
    }
}

#[derive(Template)]
#[template(path = "jobs/item.html")]
pub struct JobsItem<'a> {
    pub job: &'a Job,
    pub summary: &'a JobsItemSummary<'a>,
    pub details: &'a JobsItemDetails<'a>,
}

impl<'a> JobsItem<'a> {
    pub fn dom_id(&self) -> String {
        format!("job-item-{}", self.job.id)
    }
}

#[derive(Template)]
#[template(path = "jobs/summary.html")]
pub struct JobsItemSummary<'a> {
    pub job: &'a Job,
}

impl<'a> JobsItemSummary<'a> {
    pub fn dom_id(&self) -> String {
        format!("job-summary-{}", self.job.id)
    }

    pub fn collapse_id(&self) -> String {
        format!("job-collapse-{}", self.job.id)
    }
}

#[derive(Template)]
#[template(path = "jobs/details.html")]
pub struct JobsItemDetails<'a> {
    pub job: &'a Job,
}

impl<'a> JobsItemDetails<'a> {
    pub fn dom_id(&self) -> String {
        format!("job-details-{}", self.job.id)
    }
}

#[derive(Template)]
#[template(path = "jobs/item.turbo.html")]
pub struct JobsItemTurbo<'a> {
    pub summary: &'a JobsItemSummary<'a>,
    pub details: &'a JobsItemDetails<'a>,
}

#[derive(Template)]
#[template(path = "jobs/completed_item.html")]
pub struct JobsCompletedItem<'a> {
    pub job: &'a Job,
}

#[derive(Template)]
#[template(path = "jobs/completed_section.html")]
pub struct JobsCompletedSection<'a> {
    pub items: &'a [JobsCompletedItem<'a>],
    pub success_count: usize,
    pub failure_count: usize,
}

impl<'a> JobsCompletedSection<'a> {
    pub fn dom_id(&self) -> &'static str {
        "jobs-completed-section"
    }

    pub fn collapse_id(&self) -> &'static str {
        "jobs-completed-collapse"
    }
}

#[derive(Template)]
#[template(path = "jobs/update.turbo.html")]
pub struct JobsUpdate<'a> {
    pub container: &'a JobsContainer<'a>,
}

pub fn render_container(jobs: &[Job]) -> Result<String, crate::templates::Error> {
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

    let container = JobsContainer {
        items: &items,
        completed: &completed_section,
    };

    let template = JobsUpdate {
        container: &container,
    };
    crate::templates::render(template)
}

pub fn render_job_item(job: &Job) -> Result<String, crate::templates::Error> {
    let summary = JobsItemSummary { job };
    let details = JobsItemDetails { job };
    let template = JobsItemTurbo {
        summary: &summary,
        details: &details,
    };
    crate::templates::render(template)
}
