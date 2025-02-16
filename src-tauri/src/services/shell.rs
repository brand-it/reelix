use crate::services::makemkvcon_parser;
use crate::state::AppState;
use sysinfo::Disks;
use tauri::State;
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

pub fn list_disks() {
    println!("=> disks:");
    let disks = Disks::new_with_refreshed_list();
    println!("#--------------------------------#");
    for disk in &disks {
        println!("Name: {:?}", disk.name());
        println!("Mount Point: {:?}", disk.mount_point());
        println!("Available Space: {}", disk.available_space());
        println!("Total Space: {}", disk.total_space());
        println!("Kind: {}", disk.kind());
        println!("File System: {:?}", disk.file_system());
        println!("Is Removable: {}", disk.is_removable());
        println!("Is Read Only: {}", disk.is_read_only());
        println!("Usage: {:?}", disk.usage());
        println!("#--------------------------------#");
    }
}

pub fn makemkvcon(app_handle: tauri::AppHandle) {
    let sidecar_command = app_handle.shell().sidecar("makemkvcon").unwrap();
    let (mut rx, mut _child) = sidecar_command
        .args(["-r", "--cache=1", "info", "disc:9999"])
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

    let disks = Disks::new_with_refreshed_list();
    for disk in disks.list() {
        let fs_bytes = disk.file_system();
        let fs_str = fs_bytes.to_str().unwrap();

        // Check if removable + known optical file system
        if disk.is_removable() && (fs_str.contains("udf") || fs_str.contains("iso9660")) {
            println!("Likely optical media:");
            println!("  Name:        {:?}", disk.name());
            println!("  Mount point: {:?}", disk.mount_point());
            println!("  File system: {}", fs_str);
        } else {
            println!("Non-optical or unrecognized: {:?}", disk);
        }
    }
}
