use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct StandardError {
    pub title: String,
    pub message: String,
}

impl std::fmt::Display for StandardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error {}: {}", self.title, self.message)
    }
}
