use serde::Serialize;
use std::error::Error;
use std::fmt;
use tera::{Context, Tera};

type ErrorHandler = fn(&tera::Error) -> ApiError;

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub code: u16,
    pub message: String,
    pub api_key: Option<String>,
}
impl Error for ApiError {}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error {}: {}", self.code, self.message)
    }
}

pub fn render(
    tera: &Tera,
    template_path: &str,
    context: &Context,
    on_error: Option<ErrorHandler>,
) -> Result<String, ApiError> {
    match tera.render(template_path, context) {
        Ok(result) => Ok(result),
        Err(e) => {
            eprintln!("Template rendering error: {:#?}", e);
            // Custom error handler if provided
            if let Some(handler) = on_error {
                Err(handler(&e))
            } else {
                Err(ApiError {
                    code: 500,
                    message: format!("An error occurred during template rendering: {e}"),
                    api_key: None,
                })
            }
        }
    }
}
