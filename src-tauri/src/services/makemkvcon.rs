use crate::models::mkv::PRGV;
use crate::models::optical_disk_info::{self, DiskId};
use crate::models::{mkv, title_info};
use crate::progress_tracker::{self, ProgressOptions};
use crate::services::{makemkvcon_parser, template};
use crate::state::AppState;
use std::ffi::OsStr;
use std::path::PathBuf;
use tauri::async_runtime::Receiver;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;
use tera::Context;

#[cfg(all(target_os = "windows", target_pointer_width = "64"))]
const MAKEMKVCON: &str = "makemkvcon64";

#[cfg(not(all(target_os = "windows", target_pointer_width = "64")))]
const MAKEMKVCON: &str = "makemkvcon";

#[derive(Debug)]
#[allow(dead_code)]
pub struct RunResults {
    pub title_infos: Vec<title_info::TitleInfo>,
    pub drives: Vec<mkv::DRV>,
    pub messages: Vec<mkv::MSG>,
}
// makemkvcon [options] Command Parameters
// https://www.makemkv.com/developers/usage.txt
//
// General options:
//
// --messages=file
// Output all messages to file. Following special file names are recognized:
// -stdout - stdout
// -stderr - stderr
// -null - disable output
// Default is stdout
//
// --progress=file
// Output all progress messages to file. The same special file names as in --messages are recognized with additional
// value "-same" to output to the same file as messages. Naturally --progress should follow --messages in this case.
// Default is no output.
//
// --debug[=file]
// Enables debug messages and optionally changes the location of debug file. Default: program preferences.
//
// --directio=true/false
// Enables or disables direct disc access. Default: program preferences.
//
// --noscan
// Don't access any media during disc scan and do not check for media insertion and removal. Helpful when other
// applications already accessing discs in other drives.
//
// --cache=size
// Specifies size of read cache in megabytes used by MakeMKV. By default program uses huge amount of memory. About 128
// MB is recommended for streaming and backup, 512MB for DVD conversion and 1024MB for Blu-ray conversion.
//
// Streaming options:
//
// --upnp=true/false
// Enable or disable UPNP streaming. Default: program preferences.
//
// --bindip=address string
// Specify IP address to bind. Default: None, UPNP server binds to the first available address and web
// server listens on all available addresses.
//
// --bindport=port
// Specify web server port to bind. Default: 51000.
//
// Backup options:
//
// --decrypt
// Decrypt stream files during backup. Default: no decryption.
//
// Conversion options:
//
// --minlength=seconds
// Specify minimum title length. Default: program preferences.
//
// Automation options.
//
// -r , --robot
// Enables automation mode. Program will output more information in a format that is easier to parse. All output is
// line-based and output is flushed on line end. All strings are quoted, all control characters and quotes are
// backlash-escaped. If you automate this program it is highly recommended to use this option. Some options make
// reference to apdefs.h file that can be found in MakeMKV open-source package, included with version for Linux.
// These values will not change in future versions.
//
// Message formats:
//
// Message output
// MSG:code,flags,count,message,format,param0,param1,...
// code - unique message code, should be used to identify particular string in language-neutral way.
// flags - message flags, see AP_UIMSG_xxx flags in apdefs.h
// count - number of parameters
// message - raw message string suitable for output
// format - format string used for message. This string is localized and subject to change, unlike message code.
// paramX - parameter for message
//
// Current and total progress title
// PRGC:code,id,name
// PRGT:code,id,name
// code - unique message code
// id - operation sub-id
// name - name string
//
// Progress bar values for current and total progress
// PRGV:current,total,max
// current - current progress value
// total - total progress value
// max - maximum possible value for a progress bar, constant
//
// Drive scan messages
// DRV:index,visible,enabled,flags,drive name,disc name
// index - drive index
// visible - set to 1 if drive is present
// enabled - set to 1 if drive is accessible
// flags - media flags, see AP_DskFsFlagXXX in apdefs.h
// drive name - drive name string
// disc name - disc name string
//
// Disc information output messages
// TCOUT:count
// count - titles count
//
// Disc, title and stream information
// CINFO:id,code,value
// TINFO:id,code,value
// SINFO:id,code,value
//
// id - attribute id, see AP_ItemAttributeId in apdefs.h
// code - message code if attribute value is a constant string
// value - attribute value
//
// Examples:
//
// Copy all titles from first disc and save as MKV files:
// makemkvcon mkv disc:0 all c:\folder
//
// List all available drives
// makemkvcon -r --cache=1 info disc:9999
//
// Backup first disc decrypting all video files in automation mode with progress output
// makemkvcon backup --decrypt --cache=16 --noscan -r --progress=-same disc:0 c:\folder
//
// Start streaming server with all output suppressed on a specific address and port
// makemvcon stream --upnp=1 --cache=128 --bindip=192.168.1.102 --bindport=51000 --messages=-none
async fn run(
    disk_id: DiskId,
    mut receiver: Receiver<CommandEvent>,
    app_handle: AppHandle,
) -> RunResults {
    let mut title_infos: Vec<title_info::TitleInfo> = Vec::new();
    let mut drives: Vec<mkv::DRV> = Vec::new();
    let mut messages: Vec<mkv::MSG> = Vec::new();
    let mut tracker: Option<progress_tracker::Base> = None;
    while let Some(event) = receiver.recv().await {
        match event {
            CommandEvent::Stdout(line_bytes) => {
                let line = String::from_utf8_lossy(&line_bytes);
                let parsed_stdout = makemkvcon_parser::parse_mkv_string(&line);
                for mkv_data in parsed_stdout {
                    match mkv_data {
                        mkv::MkvData::TINFO(tinfo) => {
                            let title_info: &mut title_info::TitleInfo =
                                match title_infos.iter_mut().find(|t| t.id == tinfo.id) {
                                    Some(title) => title,
                                    None => {
                                        title_infos.push(title_info::TitleInfo::new(tinfo.id));
                                        title_infos.last_mut().unwrap()
                                    }
                                };
                            title_info.set_field(&tinfo.type_code, tinfo.value)
                        }
                        mkv::MkvData::DRV(drv) => {
                            drives.push(drv);
                        }
                        mkv::MkvData::PRGV(prgv) => {
                            update_tracker(&mut tracker, prgv);
                            update_disk_progress_state(&disk_id, &tracker, &app_handle, None, None);
                            emit_progress(&disk_id, &app_handle);
                        }
                        mkv::MkvData::PRGT(prgt) => {
                            create_tracker(&mut tracker);
                            update_disk_progress_state(
                                &disk_id,
                                &tracker,
                                &app_handle,
                                Some(&prgt.name),
                                None,
                            );
                        }
                        mkv::MkvData::PRGC(_prgc) => {
                            tracker = None;
                        }
                        mkv::MkvData::MSG(msg) => {
                            messages.push(msg.clone());
                            update_disk_progress_state(
                                &disk_id,
                                &tracker,
                                &app_handle,
                                None,
                                Some(&msg.message),
                            );
                        }
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
        title_infos,
        drives,
        messages,
    }
}

fn create_tracker(tracker: &mut Option<progress_tracker::Base>) {
    let options = ProgressOptions {
        total: Some(1 as usize),
        autostart: true,
        autofinish: true,
        starting_at: Some(0),
        projector_type: Some("smoothed".to_string()),
        projector_strength: Some(0.1),
        projector_at: Some(0.0),
    };
    *tracker = Some(progress_tracker::Base::new(Some(options)));
}

fn update_tracker(tracker: &mut Option<progress_tracker::Base>, prgv: PRGV) {
    if tracker.is_none() {
        let options = ProgressOptions {
            total: Some(prgv.pmax as usize),
            autostart: true,
            autofinish: true,
            starting_at: Some(0),
            projector_type: Some("smoothed".to_string()),
            projector_strength: Some(0.1),
            projector_at: Some(0.0),
        };
        // update the none tracker with this new one.
        *tracker = Some(progress_tracker::Base::new(Some(options)));
    }

    if let Some(ref mut base) = tracker {
        base.set_total(prgv.pmax as usize);
        base.set_progress(prgv.current as usize);
    }
}

fn spawn<I: IntoIterator<Item = S> + std::fmt::Debug + std::marker::Copy, S: AsRef<OsStr>>(
    app_handle: &AppHandle,
    disk_id: &DiskId,
    args: I,
) -> Receiver<CommandEvent> {
    let sidecar_command = app_handle
        .shell()
        .sidecar(MAKEMKVCON)
        .expect("failed to get makemkvcon");
    let (receiver, child) = sidecar_command
        .args(args)
        .spawn()
        .expect("Failed to spawn sidecar for rip_title");
    let state = app_handle.state::<AppState>();
    match state.find_optical_disk_by_id(disk_id) {
        Some(disk) => {
            disk.write()
                .expect("Failed to acquire lock on disk from disk_arc in spawn command")
                .set_pid(Some(child.pid()));
        }
        None => println!("failed to assign the sidecar to disk {:?}", disk_id),
    }
    println!("Executing command: makemkvcon {:?}", args);
    receiver
}

pub async fn rip_title(
    app_handle: &AppHandle,
    disk_id: &DiskId,
    title_id: u32,
    tmp_dir: &PathBuf,
) -> Result<RunResults, String> {
    let state = app_handle.state::<AppState>();

    match state.find_optical_disk_by_id(disk_id) {
        Some(disk) => {
            let path = {
                disk.read()
                    .expect("Failed to acquire lock on disk from disk_arc in rip_title command")
                    .disc_name
                    .clone()
            };

            let disc_arg = format!("dev:{}", path);
            let tmp_dir_str = tmp_dir.to_string_lossy();
            let args = [
                "mkv",
                &disc_arg,
                &title_id.to_string(),
                &tmp_dir_str,
                "--progress=-same",
                "--robot",
                "--profile=\"FLAC\"",
            ];

            let receiver = spawn(app_handle, disk_id, args);
            let app_handle_clone = app_handle.clone();
            let status = Ok(run(disk_id.clone(), receiver, app_handle_clone).await);

            let result = template::render(
                &state.tera,
                "disks/toast_progress.html.turbo",
                &Context::new(),
                None,
            )
            .expect("Failed to render disks/toast_progress.html.turbo");
            app_handle
                .emit("disks-changed", result)
                .expect("Failed to emit disks-changed");
            status
        }
        None => Err(format!("Failed to find disk using id {:?}", disk_id)),
    }
}

// dev:<DeviceName>  - open disc with OS device name <DeviceName>
pub async fn title_info(disk_id: DiskId, app_handle: &AppHandle, dev: &str) -> RunResults {
    let disk_args = format!("dev:{}", dev);
    let receiver = spawn(app_handle, &disk_id, ["-r", "info", &disk_args]);
    let app_handle_clone = app_handle.clone();

    run(disk_id, receiver, app_handle_clone).await
}

fn update_disk_progress_state(
    disk_id: &DiskId,
    tracker: &Option<progress_tracker::Base>,
    app_handle: &AppHandle,
    label: Option<&String>,
    message: Option<&String>,
) {
    // Early return if there's no tracker.
    let tracker = match tracker {
        Some(tracker) => tracker,
        None => return,
    };

    let state = app_handle.state::<AppState>();

    // Try to find the disk.
    let disk_arc = match state.find_optical_disk_by_id(disk_id) {
        Some(disk) => disk,
        None => {
            println!("Failed to find disk using {:?}", disk_id);
            return;
        }
    };

    // Lock the disk.
    let disk = disk_arc
        .write()
        .expect("failed to lock disk in update_disk_progress_state");

    // Lock current progress to use its label/message as default if needed.
    let current_progress = disk
        .progress
        .lock()
        .expect("failed to lock disk progress in update_disk_progress_state");
    let default_label = current_progress
        .as_ref()
        .map(|p| p.label.clone())
        .unwrap_or_default();
    let default_message = current_progress
        .as_ref()
        .map(|p| p.message.clone())
        .unwrap_or_default();
    // Drop the current_progress guard early since we don't need it anymore.
    drop(current_progress);

    // Build new progress using values from the tracker and provided options.
    let new_progress = optical_disk_info::Progress {
        eta: tracker.time_component.estimated(None),
        percentage: tracker.percentage_component.percentage(),
        label: label.unwrap_or(&default_label).to_string(),
        message: message.unwrap_or(&default_message).to_string(),
    };

    // Update the disk's progress.
    disk.set_progress(Some(new_progress));
}

fn emit_progress(disk_id: &DiskId, app_handle: &AppHandle) {
    let state = app_handle.state::<AppState>();
    let optical_disk_info = {
        match state.find_optical_disk_by_id(disk_id) {
            Some(disk) => disk.read().expect("failed to lock disk").clone(),
            None => {
                println!("failed to find disk using {:?}", disk_id);
                return;
            }
        }
    };
    let movie_title_year = match optical_disk_info
        .movie_details
        .lock()
        .expect("failed to lock movie details in emit progress")
        .as_ref()
    {
        Some(movie) => movie.title_year(),
        None => "Unknown".to_string(),
    };
    let progress = optical_disk_info
        .progress
        .lock()
        .expect("failure to lock progress");

    if progress.is_some() {
        let mut context = Context::new();
        context.insert("progress", &*progress);
        context.insert("movie_title_year", &movie_title_year);

        let result = template::render(
            &state.tera,
            "disks/toast_progress.html.turbo",
            &context,
            None,
        )
        .expect("Failed to render disks/toast_progress.html.turbo");
        app_handle
            .emit("disks-changed", result)
            .expect("Failed to emit disks-changed");
    }
}
