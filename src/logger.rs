//! # Logger Module
//! This module provides a simple logging function to write messages to the log file.

use chrono::Local;
use crate::settings;
use std::fs::OpenOptions;
use std::io::Write;

/// Logs a message to the plgrt.log file.
///
/// # Arguments
///
/// * `message` - The message to log.
pub fn log_message(message: &str) {
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
