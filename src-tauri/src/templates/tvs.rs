use super::InlineTemplate;
use crate::models::movie_db::TvResponse;
use askama::Template;

#[derive(Template)]
#[template(path = "tvs/show.turbo.html")]
pub struct TvsShowTurbo<'a> {
    pub tv_show: &'a TvsShow<'a>,
}

#[derive(Template)]
#[template(path = "tvs/show.html")]
pub struct TvsShow<'a> {
    pub tv: &'a TvResponse,
}

pub fn render_show(tv: &TvResponse) -> Result<String, super::Error> {
    let template = TvsShowTurbo {
        tv_show: &TvsShow { tv },
    };
    super::render(template)
}
