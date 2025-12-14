use std::fs;
use std::path::Path;

fn main() {
    // Create the dist directory if it doesn't exist
    // This prevents the proc macro panic when running `cargo build` directly
    // without first running the npm build step. The frontendDist path in
    // tauri.conf.json is set to "../dist" and Tauri expects this directory
    // to exist at build time, even if it's empty.
    let dist_path = Path::new("../dist");
    if !dist_path.exists() {
        fs::create_dir_all(dist_path).expect("Failed to create dist directory");
    }

    tauri_build::build();
}
