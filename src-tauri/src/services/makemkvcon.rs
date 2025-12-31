use crate::models::mkv::PRGV;
use crate::models::optical_disk_info::DiskId;
use crate::models::optical_disk_info::OpticalDiskInfo;
use crate::models::{mkv, title_info};
use crate::progress_tracker::{self, ProgressOptions};
use crate::services::makemkvcon_parser;
use crate::state::job_state::emit_progress;
use crate::state::job_state::Job;
use crate::state::title_video::TitleVideo;
use crate::state::AppState;
use crate::templates;
use log::debug;
use std::ffi::OsStr;
use std::sync::{Arc, RwLock};
use tauri::async_runtime::Receiver;
use tauri::{AppHandle, Manager};
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

#[cfg(all(target_os = "windows", target_pointer_width = "64"))]
const MAKEMKVCON: &str = "makemkvcon64";

#[cfg(not(all(target_os = "windows", target_pointer_width = "64")))]
const MAKEMKVCON: &str = "makemkvcon";

pub struct RunResults {
    pub title_infos: Vec<title_info::TitleInfo>,
    pub drives: Vec<mkv::DRV>,
    pub messages: Vec<mkv::MSG>,
    // pub err_messages: Vec<String>,
}

impl RunResults {
    // fn success(&self) -> bool {
    //     self.messages.iter().any(|message| message.code == 5003)
    // }

    // fn err_messages(&self) -> Vec<&mkv::MSG> {
    //     self.messages
    //         .iter()
    //         .filter(|message| message.code == 5003 || message.code == 5004)
    //         .collect()
    // }

    fn err_summary(&self) -> Option<&mkv::MSG> {
        self.messages.iter().find(|message| message.code == 5003)
    }
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
    job: &Arc<RwLock<Job>>,
    mut receiver: Receiver<CommandEvent>,
    app_handle: AppHandle,
) -> Result<RunResults, String> {
    let mut run_results = RunResults {
        messages: Vec::new(),
        drives: Vec::new(),
        title_infos: Vec::new(),
        // err_messages: Vec::new(),
    };

    let mut tracker: Option<progress_tracker::Base> = None;
    while let Some(event) = receiver.recv().await {
        match event {
            CommandEvent::Stdout(line_bytes) => {
                let line = String::from_utf8_lossy(&line_bytes);
                let parse_mkv_string: Vec<mkv::MkvData> =
                    makemkvcon_parser::parse_mkv_string(&line);
                convert_to_run_result(
                    &app_handle,
                    job,
                    parse_mkv_string,
                    &mut run_results,
                    &mut tracker,
                );
            }
            CommandEvent::Stderr(line_bytes) => {
                let line = String::from_utf8_lossy(&line_bytes);
                debug!("Stderr: {line}");
                Err(format!("makemkvcon stderr: {line}"))?;
            }
            CommandEvent::Error(error) => {
                debug!("Error: {error}");
                Err(format!("makemkvcon error: {error}"))?;
            }
            CommandEvent::Terminated(payload) => {
                debug!("Terminated: {payload:?}");
                if payload.code > Some(0) {
                    Err(format!("makemkvcon terminated: {payload:?}"))?;
                }
            }
            other => {
                debug!("Other command event: {other:?}");
                Err(format!("makemkvcon other event: {other:?}"))?;
            }
        }
    }
    emit_progress(&app_handle, job, true);
    Ok(run_results)
}

fn convert_to_run_result(
    app_handle: &AppHandle,
    job: &Arc<RwLock<Job>>,
    parse_mkv_string: Vec<mkv::MkvData>,
    run_results: &mut RunResults,
    tracker: &mut Option<progress_tracker::Base>,
) {
    for mkv_data in parse_mkv_string {
        match mkv_data {
            mkv::MkvData::TINFO(tinfo) => {
                set_title_info_field(&tinfo, run_results);
            }
            mkv::MkvData::DRV(drv) => {
                run_results.drives.push(drv);
            }
            mkv::MkvData::PRGV(prgv) => {
                update_tracker(tracker, prgv);
                update_job_progress(job, tracker);
                emit_progress(app_handle, job, false);
            }
            mkv::MkvData::PRGT(prgt) => {
                create_tracker(tracker);
                update_job_progress(job, tracker);
                job.write().unwrap().subtitle = Some(prgt.name.clone());
                emit_progress(app_handle, job, true);
            }
            mkv::MkvData::PRGC(_prgc) => {
                create_tracker(tracker);
                update_job_progress(job, tracker);
                emit_progress(app_handle, job, true);
            }
            mkv::MkvData::MSG(msg) => {
                run_results.messages.push(msg.clone());
                update_job_progress(job, tracker);
                job.write().unwrap().message = Some(msg.message.clone());
                emit_progress(app_handle, job, true);
            }
            _ => {}
        }
    }
}

fn create_tracker(tracker: &mut Option<progress_tracker::Base>) {
    let options = ProgressOptions {
        total: Some(1_usize),
        autostart: true,
        autofinish: true,
        starting_at: Some(0),
        projector_type: Some("smoothed".to_string()),
        projector_strength: Some(0.1),
        projector_at: Some(0.0),
    };
    *tracker = Some(progress_tracker::Base::new(Some(options)));
}

fn set_title_info_field(tinfo: &mkv::TINFO, run_results: &mut RunResults) {
    let title_info: &mut title_info::TitleInfo = match run_results
        .title_infos
        .iter_mut()
        .find(|t| t.id == tinfo.id)
    {
        Some(title) => title,
        None => {
            run_results
                .title_infos
                .push(title_info::TitleInfo::new(tinfo.id));
            run_results.title_infos.last_mut().unwrap()
        }
    };
    title_info.set_field(&tinfo.type_code, tinfo.value.clone())
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
    job: &Arc<RwLock<Job>>,
    args: I,
) -> Receiver<CommandEvent> {
    let mut sidecar_command = app_handle
        .shell()
        .sidecar(MAKEMKVCON)
        .expect("failed to get makemkvcon");

    // On Linux, set LD_LIBRARY_PATH to include the resource directory
    // where the shared libraries are located
    #[cfg(target_os = "linux")]
    {
        if let Ok(resource_path) = app_handle.path().resource_dir() {
            sidecar_command = sidecar_command.env("LD_LIBRARY_PATH", resource_path);
        }
    }

    let (receiver, child) = sidecar_command
        .args(args)
        .spawn()
        .expect("Failed to spawn sidecar for rip_title");
    let disk_id = job
        .read()
        .expect("failed to lock job for read")
        .disk
        .as_ref()
        .expect("There should of been a disk")
        .id;
    let state = app_handle.state::<AppState>();
    match state.find_optical_disk_by_id(&disk_id) {
        Some(disk) => {
            disk.write()
                .expect("Failed to acquire lock on disk from disk_arc in spawn command")
                .set_pid(Some(child.pid()));
        }
        None => debug!("failed to assign the sidecar to disk {disk_id}"),
    }
    debug!("Executing command: makemkvcon {args:?}");
    receiver
}

fn disk_index_args(app_handle: &AppHandle, disk_id: &DiskId) -> String {
    let state: tauri::State<'_, AppState> = app_handle.state::<AppState>();

    match state.find_optical_disk_by_id(disk_id) {
        Some(disk) => {
            let locked_disk = disk.read().expect("Failed to grab disk");
            format!("disc:{}", locked_disk.index)
        }
        None => "".to_string(),
    }
}

// This function is currently not used.
// well it used but it not really helpful since I don't have good ways to test it
// pub async fn back_disk(
//     app_handle: &AppHandle,
//     disk_id: &DiskId,
//     tmp_dir: &Path,
// ) -> Result<RunResults, String> {
//     let args = disk_index_args(disk_id, app_handle);
//     let tmp_dir_str = tmp_dir.to_string_lossy();
//     let args = [
//         "backup",
//         "--progress=-same",
//         "--robot",
//         "--noscan",
//         &args,
//         &tmp_dir_str,
//     ];

//     let receiver = spawn(app_handle, disk_id, args);
//     templates::disks::emit_disk_change(app_handle);
//     let app_handle_clone = app_handle.clone();
//     let response = run(*disk_id, &None, receiver, app_handle_clone).await;
//     match response {
//         Ok(run_results) => {
//             if let Some(err_summary) = run_results.err_summary() {
//                 Err(err_summary.message.clone())
//             } else {
//                 Ok(run_results)
//             }
//         }
//         Err(e) => Err(e),
//     }
// }

pub async fn rip_title(
    app_handle: &AppHandle,
    job: &Arc<RwLock<Job>>,
    title_video: &Arc<RwLock<TitleVideo>>,
) -> Result<RunResults, String> {
    let disk = job
        .read()
        .unwrap()
        .disk
        .clone()
        .expect("There should of been a disk");
    let args = disk_args(&disk);
    let tmp_dir = title_video
        .read()
        .unwrap()
        .create_video_dir(&app_handle.state::<AppState>());
    let args = [
        "mkv",
        &args,
        &title_video.read().unwrap().title.id.to_string(),
        &tmp_dir.to_string_lossy(),
        "--progress=-same",
        "--robot",
        "--minlength=45",
        "--cache=1024",
        "--noscan",
    ];

    let receiver = spawn(app_handle, job, args);
    templates::disks::emit_disk_change(app_handle);

    let response = run(job, receiver, app_handle.clone()).await;
    match response {
        Ok(run_results) => {
            if let Some(err_summary) = run_results.err_summary() {
                Err(err_summary.message.clone())
            } else {
                Ok(run_results)
            }
        }
        Err(e) => Err(e),
    }
}

#[cfg(target_os = "windows")]
fn disk_args(disk: &OpticalDiskInfo) -> String {
    format!("dev:{}", disk.dev)
}

#[cfg(not(target_os = "windows"))]
fn disk_args(disk: &OpticalDiskInfo) -> String {
    format!("file:{}", disk.mount_point.to_string_lossy())
}

pub async fn title_info(
    app_handle: &AppHandle,
    job: &Arc<RwLock<Job>>,
) -> Result<RunResults, String> {
    let disk_id = job
        .read()
        .expect("failed to lock job for read")
        .disk
        .as_ref()
        .expect("There should of been a disk")
        .id;
    let args = disk_index_args(app_handle, &disk_id);
    let receiver = spawn(
        app_handle,
        job,
        ["-r", "--minlength=45", "--cache=128", "info", &args],
    );
    templates::disks::emit_disk_change(app_handle);
    let app_handle_clone = app_handle.clone();

    run(job, receiver, app_handle_clone).await
}

fn update_job_progress(job: &Arc<RwLock<Job>>, tracker: &Option<progress_tracker::Base>) {
    if let Some(ref tracker) = tracker {
        let mut job_guard = job.write().expect("failed to lock job for write");
        job_guard.update_progress(tracker);
    }
}
