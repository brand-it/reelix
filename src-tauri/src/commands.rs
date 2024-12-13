// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use serde::Serialize;
use tera::{Tera, Context};
use lazy_static::lazy_static;

lazy_static! {
    static ref TERA: Tera = {
        match Tera::new("templates/**/*.html") {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Parsing error(s): {e}");
                ::std::process::exit(1);
            }
        }
    };
}

#[derive(Serialize)]
struct Greeting {
    name: String,
}

#[tauri::command]
pub fn greet(name: &str) -> String {
  let greeting = Greeting {
      name: name.to_string(),
  };
  let context = Context::from_serialize(&greeting).expect("Failed to retrieve the value");

  match TERA.render("greet.html", &context) {
    Ok(result) => result,
    Err(e) => {
        eprintln!("Template rendering error: {e}");
        format!("An error occurred: {e}")
    }
  }
}

#[tauri::command]
pub fn about() -> String {
  match TERA.render("about.html", &Context::new()) {
    Ok(result) => result,
    Err(e) => {
        eprintln!("Template rendering error: {e}");
        format!("An error occurred: {e}")
    }
  }
}
