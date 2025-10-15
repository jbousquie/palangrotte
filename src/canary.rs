//! # Canary Module
//! This module manages canary folder and file operations, including creation, timestamp updates,
//! and registering folders with the file watcher.

use crate::logger::log_message;
use crate::notify_access::notify_service;
use std::sync::Arc;
use crate::settings::Settings;
use filetime::{set_file_mtime, FileTime};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use rand::Rng;
use std::fs::{self, File};
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use system_shutdown;

/// Registers a canary folder for monitoring.
///
/// This function checks if the folder exists. If not, it attempts to create it.
/// If the creation fails, it logs an error message.
///
/// # Arguments
///
/// * `folder_path` - The path to the canary folder.
/// * `watcher` - A mutable reference to the file watcher.
/// * `settings` - The application settings.
///
/// # Returns
///
/// * `Ok(())` - If the folder was successfully registered for monitoring.
/// * `Err(String)` - If there was an error.
pub fn register_canary_folder(
    folder_path: &str,
    watcher: &mut RecommendedWatcher,
    settings: &Settings,
) -> Result<(), String> {
    let path = Path::new(folder_path);
    if !path.exists() {
        match fs::create_dir_all(path) {
            Ok(_) => {
                let msg = format!("Folder {} created successfully.", folder_path);
                log_message(&settings.log_file, &msg);
                create_canary_files(folder_path, settings);
                return Ok(());
            }
            Err(e) => {
                return Err(format!("Failed to create folder {}: {}", folder_path, e));
            }
        }
    }

    // Folder exists, check if it is empty
    match fs::read_dir(path) {
        Ok(entries) => {
            let mut has_files = false;
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() {
                        has_files = true;
                        // Touch the file
                        if let Err(e) = set_file_mtime(&path, FileTime::now()) {
                            let msg = format!("Failed to touch file {}: {}", path.display(), e);
                            log_message(&settings.log_file, &msg);
                        }
                    }
                }
            }
            if !has_files {
                create_canary_files(folder_path, settings);
            }
            // Now there are files, start monitoring
            match watcher.watch(path, RecursiveMode::Recursive) {
                Ok(_) => {
                    let msg = format!("Started monitoring folder {}.", folder_path);
                    log_message(&settings.log_file, &msg);
                    Ok(())
                }
                Err(e) => Err(format!(
                    "Failed to start monitoring folder {}: {}",
                    folder_path, e
                )),
            }
        }
        Err(e) => Err(format!("Failed to read directory {}: {}", folder_path, e)),
    }
}

/// Creates canary files in the given folder.
///
/// This function creates a random number of files (between 2 and 5) with random names and extensions.
/// The files are filled with random data to have a size between 12 KB and 120 KB.
///
/// # Arguments
///
/// * `folder_path` - The path to the folder where canary files will be created.
/// * `settings` - The application settings.
fn create_canary_files(folder_path: &str, settings: &Settings) {
    let mut rng = rand::thread_rng();
    let num_files = rng.gen_range(settings.min_canary_files..=settings.max_canary_files);

    for _ in 0..num_files {
        let name = settings
            .canary_file_names
            .get(rng.gen_range(0..settings.canary_file_names.len()))
            .unwrap();
        let ext = settings
            .canary_file_extensions
            .get(rng.gen_range(0..settings.canary_file_extensions.len()))
            .unwrap();
        let file_path = Path::new(folder_path).join(format!("{}.{}", name, ext));

        let size = rng.gen_range(settings.min_canary_file_size..=settings.max_canary_file_size);
        let mut data = vec![0u8; size.try_into().unwrap()];
        rng.fill(&mut data[..]);

        match File::create(&file_path) {
            Ok(mut file) => {
                if let Err(e) = file.write_all(&data) {
                    let msg = format!("Failed to write to file {}: {}", file_path.display(), e);
                    log_message(&settings.log_file, &msg);
                }

                #[cfg(unix)]
                {
                    use std::fs::Permissions;
                    if let Err(e) = fs::set_permissions(&file_path, Permissions::from_mode(0o666)) {
                        let msg = format!(
                            "Failed to set permissions for file {}: {}",
                            file_path.display(),
                            e
                        );
                        log_message(&settings.log_file, &msg);
                    }
                }
            }
            Err(e) => {
                let msg = format!("Failed to create file {}: {}", file_path.display(), e);
                log_message(&settings.log_file, &msg);
            }
        }
    }
    let msg = format!("Created {} canary files in {}.", num_files, folder_path);
    log_message(&settings.log_file, &msg);
}

/// Called when a modification is detected in a monitored folder.
///
/// # Arguments
///
/// * `foldername` - The name of the folder where the modification was detected.
/// * `settings` - The application settings.
async fn modification_detection(foldername: &str, settings: &Settings) {
    println!("Modification detected in folder or file: {}", foldername);
    let msg = format!("Modification detected in folder or file: {}", foldername);
    log_message(&settings.log_file, &msg);
    notify_service(&settings.service_url, foldername, &settings.log_file).await;
    notify_sessions(settings);
    shutdown_system(settings);
}

/// Handles a file system event.
///
/// This function is called when a file system event is received from the watcher.
/// It iterates over the paths in the event and spawns a new Tokio task for each path
/// to call `modification_detection` asynchronously.
///
/// # Arguments
///
/// * `event` - The file system event.
/// * `settings` - The application settings.
pub async fn handle_event(event: Event, settings: Arc<Settings>) {
    for path in &event.paths {
        if let Some(folder_str) = path.to_str() {
            let folder_str_clone = folder_str.to_string();
            let settings_clone = Arc::clone(&settings);
            tokio::spawn(async move {
                modification_detection(&folder_str_clone, &settings_clone).await;
            });
        }
    }
}

/// Notifies logged-in user sessions about a security alert.
///
/// # Arguments
///
/// * `settings` - The application settings.
#[cfg(windows)]
fn notify_sessions(settings: &Settings) {
    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use windows_sys::Win32::System::RemoteDesktop::{
        WTS_CURRENT_SERVER_HANDLE, WTS_SESSION_INFOW, WTSActive, WTSEnumerateSessionsW,
        WTSFreeMemory, WTSSendMessageW,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::MB_OK;

    let title: Vec<u16> = OsStr::new(&settings.notification_title)
        .encode_wide()
        .chain(once(0))
        .collect();
    let message: Vec<u16> = OsStr::new(&settings.notification_message)
        .encode_wide()
        .chain(once(0))
        .collect();

    let mut session_info_ptr: *mut WTS_SESSION_INFOW = ptr::null_mut();
    let mut count = 0;

    unsafe {
        if WTSEnumerateSessionsW(
            WTS_CURRENT_SERVER_HANDLE,
            0,
            1,
            &mut session_info_ptr,
            &mut count,
        ) != 0
        {
            let session_info = std::slice::from_raw_parts(session_info_ptr, count as usize);
            for session in session_info {
                if session.State == WTSActive {
                    let mut response = 0;
                    WTSSendMessageW(
                        WTS_CURRENT_SERVER_HANDLE,
                        session.SessionId,
                        title.as_ptr() as *mut _,
                        (title.len() - 1) as u32 * 2,
                        message.as_ptr() as *mut _,
                        (message.len() - 1) as u32 * 2,
                        MB_OK,
                        30, // timeout 30 seconds
                        &mut response,
                        1, // wait for response
                    );
                }
            }
            WTSFreeMemory(session_info_ptr as *mut _);
            log_message(&settings.log_file, "Successfully notified user sessions.");
        } else {
            log_message(&settings.log_file, "Failed to enumerate user sessions.");
        }
    }
}

#[cfg(unix)]
fn notify_sessions(settings: &Settings) {
    use crate::linux_notification::NOTIFY_SCRIPT;
    use std::process::Command;

    let status = Command::new("sh")
        .arg("-c")
        .arg(NOTIFY_SCRIPT)
        .arg("notify-send-all") // This is $0 for the script
        .arg(&settings.notification_title)
        .arg(&settings.notification_message)
        .status();

    match status {
        Ok(status) => {
            if status.success() {
                log_message(&settings.log_file, "Successfully notified user sessions.");
            } else {
                let msg = format!(
                    "Failed to notify user sessions. Exit code: {}",
                    status
                );
                log_message(&settings.log_file, &msg);
            }
        }
        Err(e) => {
            let msg = format!("Error executing embedded notify script: {}", e);
            log_message(&settings.log_file, &msg);
        }
    }
}

/// Shuts down the system.
///
/// # Arguments
///
/// * `settings` - The application settings.
fn shutdown_system(settings: &Settings) {
    log_message(&settings.log_file, "Attempting to force system shutdown...");
    match system_shutdown::force_shutdown() {
        Ok(_) => {
            log_message(
                &settings.log_file,
                "Forced system shutdown command executed successfully.",
            );
        }
        Err(error) => {
            let msg = format!(
                "Forced shutdown failed: {}. Attempting graceful shutdown...",
                error
            );
            log_message(&settings.log_file, &msg);
            match system_shutdown::shutdown() {
                Ok(_) => {
                    log_message(
                        &settings.log_file,
                        "Graceful system shutdown command executed successfully.",
                    );
                }
                Err(error) => {
                    let msg = format!("Graceful shutdown also failed: {}", error);
                    log_message(&settings.log_file, &msg);
                }
            }
        }
    }
}
