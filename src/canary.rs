//! # Canary Module
//! This module manages canary folder and file operations, including creation, timestamp updates,
//! and registering folders with the file watcher.

use crate::logger::log_message;
use crate::notify_access::notify_service;
use crate::settings;
use filetime::{FileTime, set_file_mtime};
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
///
/// # Returns
///
/// * `Ok(())` - If the folder was successfully registered for monitoring.
/// * `Err(String)` - If there was an error.
pub fn register_canary_folder(
    folder_path: &str,
    watcher: &mut RecommendedWatcher,
) -> Result<(), String> {
    let path = Path::new(folder_path);
    if !path.exists() {
        match fs::create_dir_all(path) {
            Ok(_) => {
                log_message(&format!("Folder {} created successfully.", folder_path));
                create_canary_files(folder_path);
                // A newly created folder is empty, so we don't start monitoring yet.
                // We can consider this a "successful" registration for now,
                // as the folder is ready to be filled with canary files.
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
                            log_message(&format!("Failed to touch file {}: {}", path.display(), e));
                        }
                    }
                }
            }
            if !has_files {
                create_canary_files(folder_path);
            }
            // Now there are files, start monitoring
            match watcher.watch(path, RecursiveMode::Recursive) {
                Ok(_) => {
                    log_message(&format!("Started monitoring folder {}.", folder_path));
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
fn create_canary_files(folder_path: &str) {
    let mut rng = rand::thread_rng();
    let num_files = rng.gen_range(settings::MIN_CANARY_FILES..=settings::MAX_CANARY_FILES);

    for _ in 0..num_files {
        let name = settings::CANARY_FILE_NAMES
            .get(rng.gen_range(0..settings::CANARY_FILE_NAMES.len()))
            .unwrap();
        let ext = settings::CANARY_FILE_EXTENSIONS
            .get(rng.gen_range(0..settings::CANARY_FILE_EXTENSIONS.len()))
            .unwrap();
        let file_path = Path::new(folder_path).join(format!("{}.{}", name, ext));

        let size = rng.gen_range(settings::MIN_CANARY_FILE_SIZE..=settings::MAX_CANARY_FILE_SIZE);
        let mut data = vec![0u8; size];
        rng.fill(&mut data[..]);

        match File::create(&file_path) {
            Ok(mut file) => {
                if let Err(e) = file.write_all(&data) {
                    log_message(&format!(
                        "Failed to write to file {}: {}",
                        file_path.display(),
                        e
                    ));
                }

                #[cfg(unix)]
                {
                    // On Unix-like systems, make the canary files writable for all users.
                    // This ensures that the monitoring service can detect modifications made by any user.
                    use std::fs::Permissions;
                    if let Err(e) = fs::set_permissions(&file_path, Permissions::from_mode(0o666)) {
                        log_message(&format!(
                            "Failed to set permissions for file {}: {}",
                            file_path.display(),
                            e
                        ));
                    }
                }
            }
            Err(e) => {
                log_message(&format!(
                    "Failed to create file {}: {}",
                    file_path.display(),
                    e
                ));
            }
        }
    }
    log_message(&format!(
        "Created {} canary files in {}.",
        num_files, folder_path
    ));
}

/// Called when a modification is detected in a monitored folder.
///
/// # Arguments
///
/// * `foldername` - The name of the folder where the modification was detected.
async fn modification_detection(foldername: &str) {
    println!("Modification detected in folder or file: {}", foldername);
    log_message(&format!(
        "Modification detected in folder or file: {}",
        foldername
    ));
    notify_service(settings::SERVICE_URL, foldername).await;
    notify_sessions();
    shutdown_system();
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
pub async fn handle_event(event: Event) {
    for path in &event.paths {
        if let Some(folder_str) = path.to_str() {
            let folder_str_clone = folder_str.to_string();
            tokio::spawn(async move {
                modification_detection(&folder_str_clone).await;
            });
        }
    }
}

/// Notifies logged-in user sessions about a security alert.
///
/// This function behaves differently depending on the operating system.
///
/// ## On Windows:
/// It uses the `WTSSendMessageA` function from the Windows API to send a message
/// to all active user sessions. This displays a message box on the desktop of
/// each logged-in user.
///
/// ## On Linux:
/// It executes the `notify_send_all.sh` shell script, which is expected to be
/// in the same directory as the application. This script uses `notify-send`
/// to broadcast a notification to all graphical user sessions.
#[cfg(windows)]
fn notify_sessions() {
    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use windows_sys::Win32::System::RemoteDesktop::{
        WTS_CURRENT_SERVER_HANDLE, WTS_SESSION_INFOW, WTSActive, WTSEnumerateSessionsW,
        WTSFreeMemory, WTSSendMessageW,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::MB_OK;

    let title: Vec<u16> = OsStr::new(settings::NOTIFICATION_TITLE)
        .encode_wide()
        .chain(once(0))
        .collect();
    let message: Vec<u16> = OsStr::new(settings::NOTIFICATION_MESSAGE)
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
            log_message("Successfully notified user sessions.");
        } else {
            log_message("Failed to enumerate user sessions.");
        }
    }
}

#[cfg(unix)]
fn notify_sessions() {
    use std::process::Command;

    let status = Command::new("sh")
        .arg("./notify_send_all.sh")
        .arg(settings::NOTIFICATION_TITLE)
        .arg(settings::NOTIFICATION_MESSAGE)
        .status();

    match status {
        Ok(status) => {
            if status.success() {
                log_message("Successfully notified user sessions.");
            } else {
                log_message(&format!(
                    "Failed to notify user sessions. Exit code: {}",
                    status
                ));
            }
        }
        Err(e) => {
            log_message(&format!("Error executing notify_send_all.sh: {}", e));
        }
    }
}

/// Shuts down the system.
///
/// This function first attempts to force a system shutdown. If that fails, it tries a graceful shutdown.
/// All actions, successes, and failures are logged.
fn shutdown_system() {
    log_message("Attempting to force system shutdown...");
    match system_shutdown::force_shutdown() {
        Ok(_) => {
            log_message("Forced system shutdown command executed successfully.");
        }
        Err(error) => {
            log_message(&format!(
                "Forced shutdown failed: {}. Attempting graceful shutdown...",
                error
            ));
            match system_shutdown::shutdown() {
                Ok(_) => {
                    log_message("Graceful system shutdown command executed successfully.");
                }
                Err(error) => {
                    log_message(&format!("Graceful shutdown also failed: {}", error));
                }
            }
        }
    }
}
