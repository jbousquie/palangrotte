use crate::logger::log_message;
use filetime::{set_file_mtime, FileTime};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::fs::{self, File};
use std::io::{self, BufRead};
use std::path::Path;

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
                            log_message(&format!(
                                "Failed to touch file {}: {}",
                                path.display(),
                                e
                            ));
                        }
                    }
                }
            }
            if !has_files {
                create_canary_files(folder_path);
                Ok(()) // Folder is empty, ready for canary files.
            } else {
                // If there are files, start monitoring
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
        }
        Err(e) => Err(format!("Failed to read directory {}: {}", folder_path, e)),
    }
}

/// Creates canary files in the given folder.
///
/// # Arguments
///
/// * `folder_path` - The path to the folder where canary files will be created.
fn create_canary_files(folder_path: &str) {
    // TODO: Implement canary file creation
    log_message(&format!(
        "Folder {} is empty. Canary files will be created.",
        folder_path
    ));
}

/// Reads the canary folders from the given file.
///
/// # Arguments
///
/// * `filename` - The path to the file containing the folder paths.
///
/// # Returns
///
/// * `Ok(Vec<String>)` - A vector of folder paths.
/// * `Err(io::Error)` - An error if the file could not be read.
pub fn read_canary_folders<P: AsRef<Path>>(filename: P) -> io::Result<Vec<String>> {
    let file = File::open(filename)?;
    let reader = io::BufReader::new(file);
    let mut folders = Vec::new();
    for line in reader.lines() {
        folders.push(line?);
    }
    Ok(folders)
}

/// Called when a modification is detected in a monitored folder.
///
/// # Arguments
///
/// * `foldername` - The name of the folder where the modification was detected.
fn modification_detection(foldername: &str) {
    println!("Modification detected in folder: {}", foldername);
}

pub fn handle_event(event: Event) {
    for path in &event.paths {
        if let Some(folder_str) = path.to_str() {
            modification_detection(folder_str);
        }
    }
}
