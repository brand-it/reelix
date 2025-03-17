use crate::models::{mkv, title_info};
use crate::services::makemkvcon_parser;
use std::collections::HashMap;
use tauri::async_runtime::Receiver;
use tauri::AppHandle;
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;
#[derive(Debug)]
pub struct RunResults {
    pub title_info: HashMap<i32, title_info::TitleInfo>,
    pub drives: Vec<mkv::DRV>,
    pub messages: Vec<mkv::MSG>,
}

async fn run(mut receiver: Receiver<CommandEvent>) -> RunResults {
    let mut title_info: HashMap<i32, title_info::TitleInfo> = HashMap::new();
    let mut drives: Vec<mkv::DRV> = Vec::new();
    let mut messages: Vec<mkv::MSG> = Vec::new();
    while let Some(event) = receiver.recv().await {
        match event {
            CommandEvent::Stdout(line_bytes) => {
                let line = String::from_utf8_lossy(&line_bytes);
                let parsed_stdout = makemkvcon_parser::parse_mkv_string(&line);
                for mkv_data in parsed_stdout {
                    match mkv_data {
                        mkv::MkvData::TINFO(tinfo) => {
                            let title_info = title_info
                                .entry(tinfo.id)
                                .or_insert_with(|| title_info::TitleInfo::new(tinfo.id));
                            title_info.set_field(&tinfo.type_code, tinfo.value)
                        }
                        mkv::MkvData::DRV(drv) => drives.push(drv),
                        mkv::MkvData::MSG(msg) => messages.push(msg),
                        _ => {}
                    }
                }
            }

            CommandEvent::Stderr(line_bytes) => {
                let line = String::from_utf8_lossy(&line_bytes);
                eprintln!("Stderr: {}", line);
            }
            CommandEvent::Error(error) => {
                eprintln!("Error: {}", error);
            }
            CommandEvent::Terminated(payload) => {
                eprintln!("Terminated: {:?}", payload);
            }
            other => {
                eprintln!("Other command event: {:?}", other);
            }
        }
    }
    RunResults {
        title_info: title_info,
        drives: drives,
        messages: messages,
    }
}

pub async fn title_info(app_handle: &AppHandle, path: &str) -> Result<RunResults, tauri::Error> {
    let sidecar_command = app_handle.shell().sidecar("makemkvcon").unwrap();
    let disc_arg = format!("file:{}", path);
    let (receiver, mut _child) = sidecar_command
        .args(["-r", "info", &disc_arg])
        .spawn()
        .expect("Failed to spawn sidecar");
    println!("mkvcommand {}", disc_arg);
    tauri::async_runtime::spawn(run(receiver)).await
}
