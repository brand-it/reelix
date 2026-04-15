use super::{render, Error, InlineTemplate};
use askama::Template;

const AUTH_DOM_ID: &str = "body";

#[derive(Template)]
#[template(path = "auth/host_setup.html")]
pub struct HostSetup<'a> {
    pub current_host: &'a str,
    pub error_message: &'a str,
}

impl HostSetup<'_> {
    pub fn dom_id(&self) -> &'static str {
        AUTH_DOM_ID
    }
}

#[derive(Template)]
#[template(path = "auth/host_setup.turbo.html")]
pub struct HostSetupTurbo<'a> {
    pub host_setup: &'a HostSetup<'a>,
}

#[derive(Template)]
#[template(path = "auth/device_code.html")]
pub struct DeviceCode<'a> {
    pub host: &'a str,
    pub user_code: &'a str,
    pub verification_uri: &'a str,
    pub error_message: &'a str,
}

impl DeviceCode<'_> {
    pub fn dom_id(&self) -> &'static str {
        AUTH_DOM_ID
    }
}

#[derive(Template)]
#[template(path = "auth/device_code.turbo.html")]
pub struct DeviceCodeTurbo<'a> {
    pub device_code: &'a DeviceCode<'a>,
}

#[derive(Template)]
#[template(path = "auth/host_unreachable.html")]
pub struct HostUnreachable<'a> {
    pub host: &'a str,
}

impl HostUnreachable<'_> {
    pub fn dom_id(&self) -> &'static str {
        AUTH_DOM_ID
    }
}

#[derive(Template)]
#[template(path = "auth/host_unreachable.turbo.html")]
pub struct HostUnreachableTurbo<'a> {
    pub host_unreachable: &'a HostUnreachable<'a>,
}

pub fn render_host_unreachable(host: &str) -> Result<String, Error> {
    let host_unreachable = HostUnreachable { host };
    let template = HostUnreachableTurbo { host_unreachable: &host_unreachable };
    render(template)
}

pub fn render_host_setup(current_host: &str, error_message: &str) -> Result<String, Error> {
    let host_setup = HostSetup { current_host, error_message };
    let template = HostSetupTurbo { host_setup: &host_setup };
    render(template)
}

pub fn render_device_code(
    host: &str,
    user_code: &str,
    verification_uri: &str,
    error_message: &str,
) -> Result<String, Error> {
    let device_code = DeviceCode { host, user_code, verification_uri, error_message };
    let template = DeviceCodeTurbo { device_code: &device_code };
    render(template)
}
