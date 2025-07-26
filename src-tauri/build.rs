use std::fs;
use std::path::PathBuf;
use tera::{Context, Tera};

fn build_index_html() {
    let project_root = PathBuf::from("../");
    let src_dir = project_root.join("src");
    let output_path = src_dir.join("index.html");

    if src_dir.exists() {
        println!("'src' directory already exists at {src_dir:?}");
    } else {
        fs::create_dir_all(&src_dir).expect("Failed to create the 'dist' directory");
        println!("Created 'src' directory at {src_dir:?}");
    }

    let tera = match Tera::new("templates/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {e}");
            ::std::process::exit(1);
        }
    };

    let rendered = tera
        .render("index.html", &Context::new())
        .expect("Failed to render index.html template");

    fs::write(&output_path, rendered).unwrap_or_else(|_| panic!("Failed to write index.html to {}",
        output_path.display()));
    println!("cargo:rerun-if-changed=templates/");
}

fn main() {
    build_index_html();
    tauri_build::build();
}
