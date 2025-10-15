//! # Logger Module
//! This module provides a simple logging function to write messages to the log file.

use chrono::Local;
use std::fs::OpenOptions;
use std::io::Write;

/// Logs a message to the specified log file.
///
/// # Arguments
///
/// * `log_file` - The path to the log file.
/// * `message` - The message to log.
pub fn log_message(log_file: &str, message: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(log_file)
    {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        if let Err(e) = writeln!(file, "[{}] {}", timestamp, message) {
            eprintln!("Couldn't write to log file: {}", e);
        }
    }
}
