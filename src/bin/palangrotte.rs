//! # Palangrotte Daemon
//! This is the main binary for the canary file monitoring daemon.
//! It initializes the watcher, reads the encrypted folder configuration, and listens for file system events.

use notify::{RecommendedWatcher, Watcher};
use palangrotte::canary::{handle_event, register_canary_folder};
use palangrotte::encryption::{decrypt_file, EncryptedFile, PBKDF2_SALT_LEN};
use palangrotte::logger::log_message;
use palangrotte::settings::{load_settings, Settings};
use ring::aead::NONCE_LEN;
use std::fs;
use std::io::Read;
use std::process;
use std::sync::mpsc::channel;
use std::sync::Arc;

/// Reads and decrypts the canary folders file.
///
/// # Arguments
///
/// * `password` - The password to decrypt the file.
/// * `settings` - The application settings.
///
/// # Returns
///
/// * `Ok(Vec<String>)` - A vector of folder paths.
/// * `Err(Box<dyn std::error::Error>)` - If there was an error reading or decrypting the file.
fn read_canary_folders(
    password: &str,
    settings: &Settings,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut encrypted_file = fs::File::open(&settings.folders_file)?;
    let mut salt = [0u8; PBKDF2_SALT_LEN];
    encrypted_file.read_exact(&mut salt)?;
    let mut nonce = [0u8; NONCE_LEN];
    encrypted_file.read_exact(&mut nonce)?;
    let mut ciphertext_with_tag = Vec::new();
    encrypted_file.read_to_end(&mut ciphertext_with_tag)?;

    let read_enc_data = EncryptedFile {
        salt,
        nonce,
        ciphertext_with_tag,
    };

    let decrypted_data = decrypt_file(read_enc_data, password)
        .map_err(|_| "Failed to decrypt folders file. Incorrect password or corrupted data.")?;

    let decrypted_string = String::from_utf8(decrypted_data)?;
    Ok(decrypted_string.lines().map(String::from).collect())
}

/// The main function for the palangrotte daemon.
///
/// This function initializes the watcher, reads the encrypted folder configuration,
/// registers the folders for monitoring, and then enters a loop to handle file system events.
#[tokio::main]
async fn main() {
    let settings = Arc::new(load_settings());
    let log_file = settings.log_file.clone();

    let password = &settings.keyword;

    let (tx, rx) = channel();

    let settings_clone = Arc::clone(&settings);
    let mut watcher: RecommendedWatcher = match Watcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                if !matches!(event.kind, notify::event::EventKind::Access(_)) {
                    if let Err(e) = tx.send((event, Arc::clone(&settings_clone))) {
                        let msg = format!("Failed to send event through channel: {}", e);
                        log_message(&settings_clone.log_file, &msg);
                        eprintln!("{}", msg);
                    }
                }
            }
        },
        Default::default(),
    ) {
        Ok(watcher) => watcher,
        Err(e) => {
            let msg = format!("Failed to create watcher: {}", e);
            log_message(&log_file, &msg);
            eprintln!("{}", msg);
            process::exit(1);
        }
    };

    match read_canary_folders(password, &settings) {
        Ok(folders) => {
            if folders.is_empty() {
                let msg = format!("{} is empty.", settings.folders_file);
                log_message(&log_file, &msg);
            } else {
                let mut successful_registrations = 0;
                for folder in &folders {
                    match register_canary_folder(folder, &mut watcher, &settings) {
                        Ok(_) => {
                            successful_registrations += 1;
                            println!("Registered folder for monitoring: {}", folder);
                        }
                        Err(e) => log_message(&log_file, &e),
                    }
                }

                if successful_registrations == 0 {
                    let msg = "No canary folders could be registered. Exiting.";
                    log_message(&log_file, msg);
                    eprintln!("{}", msg);
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            let msg = format!("Failed to read canary folders: {}", e);
            log_message(&log_file, &msg);
            eprintln!("{}", msg);
            process::exit(1);
        }
    }

    // The receiver will block the main thread until a message is received
    for (event, settings) in rx {
        handle_event(event, settings).await;
    }
}
