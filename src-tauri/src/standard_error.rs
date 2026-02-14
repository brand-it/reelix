use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StandardError {
    pub title: String,
    pub message: String,
}

impl StandardError {
    pub fn new(title: String, message: String) -> Self {
        Self { title, message }
    }
}

impl std::fmt::Display for StandardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error {}: {}", self.title, self.message)
    }
}
