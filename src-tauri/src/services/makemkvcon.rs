use crate::models::{mkv, title_info};
use crate::services::makemkvcon_parser;
use std::collections::HashMap;
use tauri::async_runtime::Receiver;
use tauri::AppHandle;
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

async fn run(mut receiver: Receiver<CommandEvent>) -> HashMap<i32, title_info::TitleInfo> {
    let mut title_info: HashMap<i32, title_info::TitleInfo> = HashMap::new();

    // read events such as stdout
    while let Some(event) = receiver.recv().await {
        match event {
            CommandEvent::Stdout(line_bytes) => {
                let line = String::from_utf8_lossy(&line_bytes);
                let parsed_stdout = makemkvcon_parser::parse_mkv_string(&line);
                for mkv_data in parsed_stdout {
                    println!("mkv_data {:?}", mkv_data);

                    match mkv_data {
                        mkv::MkvData::TINFO(tinfo) => {
                            let title_info = title_info
                                .entry(tinfo.id)
                                .or_insert_with(|| title_info::TitleInfo::new(tinfo.id));
                            title_info.set_field(&tinfo.type_code, tinfo.value)
                        }
                        _ => {}
                    }
                }
            }

            CommandEvent::Stderr(line_bytes) => {
                let line = String::from_utf8_lossy(&line_bytes);
                eprintln!("Stderr: {}", line);
            }
            other => {
                eprintln!("Other command event: {:?}", other);
            }
        }
    }
    title_info
}

pub async fn info(
    app_handle: &AppHandle,
    path: &str,
) -> Result<HashMap<i32, title_info::TitleInfo>, tauri::Error> {
    let sidecar_command = app_handle.shell().sidecar("makemkvcon").unwrap();
    let disc_arg = format!("disc:{}", path);
    let (receiver, mut _child) = sidecar_command
        .args(["-r", "--cache=1", "info", &disc_arg])
        .spawn()
        .expect("Failed to spawn sidecar");
    println!("mkvcommand {}", disc_arg);
    tauri::async_runtime::spawn(run(receiver)).await
}
