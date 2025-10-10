//! # Notify Module
//! This module contains the logic for sending notifications to a remote service.

use crate::logger::log_message;
use serde::Serialize;

#[derive(Serialize)]
struct Notification<'a> {
    file: &'a str,
}

/// Sends a notification to the specified URL.
///
/// This function sends an asynchronous POST HTTP request to the given URL.
/// The request body contains the name of the modified file/folder in JSON format.
///
/// # Arguments
///
/// * `url` - The URL to send the notification to.
/// * `file_name` - The name of the modified file or folder.
pub async fn notify_service(url: &str, file_name: &str) {
    let client = reqwest::Client::new();
    let notification = Notification { file: file_name };

    match client.post(url).json(&notification).send().await {
        Ok(response) => {
            if response.status().is_success() {
                let msg = format!("Successfully sent notification for file: {}", file_name);
                log_message(&msg);
            } else {
                let msg = format!(
                    "Failed to send notification for file: {}. Status: {}",
                    file_name,
                    response.status()
                );
                log_message(&msg);
            }
        }
        Err(e) => {
            let msg = format!(
                "Failed to send notification for file: {}. Error: {}",
                file_name, e
            );
            log_message(&msg);
        }
    }
}