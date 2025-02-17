use crate::services::makemkvcon_parser;
use tauri::AppHandle;
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

pub fn info(app_handle: &AppHandle, path: &str) {
    let sidecar_command = app_handle.shell().sidecar("makemkvcon").unwrap();
    let disc_arg = format!("disc:{}", path);
    let (mut rx, mut _child) = sidecar_command
        .args(["-r", "--cache=1", "info", &disc_arg])
        .spawn()
        .expect("Failed to spawn sidecar");
    println!("mkvcommand");

    tauri::async_runtime::spawn(async move {
        // read events such as stdout
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(line_bytes) => {
                    let line = String::from_utf8_lossy(&line_bytes);
                    let parsed_stdout = makemkvcon_parser::parse_mkv_string(&line);
                    eprintln!("Stdout: {:?}", parsed_stdout);
                }
                CommandEvent::Stderr(line_bytes) => {
                    let line = String::from_utf8_lossy(&line_bytes);
                    eprintln!("Stderr: {}", line);
                }
                other => {
                    eprintln!("Other command event: {:?}", other);
                }
            }
            // if let CommandEvent::Stdout(line_bytes) = event {
            //     let line = String::from_utf8_lossy(&line_bytes);
            //     eprintln!("commandEvent line {:#?}", line);
            //     // write to stdin
            //     // child.write("message from Rust\n".as_bytes()).unwrap();
            // }
        }
    });
}
