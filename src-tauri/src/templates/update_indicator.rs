use crate::services::version_checker::VersionState;
use crate::templates::InlineTemplate;
use askama::Template;

#[derive(Template)]
#[template(path = "update_indicator/container.html")]
pub struct UpdateIndicator<'a> {
    pub version_state: &'a VersionState,
}

impl<'a> UpdateIndicator<'a> {
    pub fn dom_id(&self) -> &'static str {
        "update-indicator"
    }

    pub fn download_url(&self) -> &'static str {
        "https://brand-it.github.io/reelix/"
    }
}

#[derive(Template)]
#[template(path = "update_indicator/update.turbo.html")]
pub struct UpdateIndicatorTurbo<'a> {
    pub update_indicator: &'a UpdateIndicator<'a>,
}

pub fn render_update(version_state: &VersionState) -> Result<String, crate::templates::Error> {
    let update_indicator = UpdateIndicator { version_state };
    let template = UpdateIndicatorTurbo {
        update_indicator: &update_indicator,
    };
    crate::templates::render(template)
}
