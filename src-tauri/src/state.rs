use std::sync::{Arc, Mutex};
use tera::Tera;
// Structure to hold shared state, thread safe version
pub struct AppState {
    pub tera: Arc<Tera>,
    pub the_movie_db_key: Arc<Mutex<String>>,
}
