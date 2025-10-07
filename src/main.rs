use chrono::Local;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, Write};
use std::path::Path;

mod settings;

fn main() {
    let folders = read_canary_folders(settings::FOLDERS_FILE);
    if let Ok(folders) = folders {
        if folders.is_empty() {
            log_message(&format!("{} is empty.", settings::FOLDERS_FILE));
        } else {
            for folder in &folders {
                register_canary_folder(folder);
            }
        }
    }
}

/// Registers a canary folder for monitoring.
///
/// This function checks if the folder exists. If not, it attempts to create it.
/// If the creation fails, it logs an error message.
///
/// # Arguments
///
/// * `folder_path` - The path to the canary folder.
fn register_canary_folder(folder_path: &str) {
    if !Path::new(folder_path).exists() {
        match fs::create_dir_all(folder_path) {
            Ok(_) => {
                log_message(&format!("Folder {} created successfully.", folder_path));
                create_canary_files(folder_path);
            }
            Err(e) => {
                log_message(&format!("Failed to create folder {}: {}", folder_path, e));
            }
        }
    } else {
        // Check if the folder is empty
        match fs::read_dir(folder_path) {
            Ok(mut dir) => {
                if dir.next().is_none() {
                    create_canary_files(folder_path);
                }
            }
            Err(e) => {
                log_message(&format!("Failed to read directory {}: {}", folder_path, e));
            }
        }
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
fn read_canary_folders<P: AsRef<Path>>(filename: P) -> io::Result<Vec<String>> {
    let file = File::open(filename)?;
    let reader = io::BufReader::new(file);
    let mut folders = Vec::new();
    for line in reader.lines() {
        folders.push(line?);
    }
    Ok(folders)
}

/// Logs a message to the plgrt.log file.
///
/// # Arguments
///
/// * `message` - The message to log.
fn log_message(message: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(settings::LOG_FILE)
    {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        if let Err(e) = writeln!(file, "[{}] {}", timestamp, message) {
            eprintln!("Couldn't write to log file: {}", e);
        }
    }
}
