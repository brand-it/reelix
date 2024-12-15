use std::sync::Arc;
use tera::Tera;

// Structure to hold shared state, thread safe version
pub struct AppState {
    pub tera: Arc<Tera>,
}
