//! # Settings Module
//! This module defines the settings structure and provides a function to load settings from a TOML file.

use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub folders_file: String,
    pub log_file: String,
    pub keyword: String,
    pub service_url: String,
    pub canary_file_names: Vec<String>,
    pub canary_file_extensions: Vec<String>,
    pub min_canary_files: u32,
    pub max_canary_files: u32,
    pub min_canary_file_size: u64,
    pub max_canary_file_size: u64,
    pub notification_title: String,
    pub notification_message: String,
}

impl Default for Settings {
    /// Creates a new `Settings` instance with default values.
    fn default() -> Self {
        Settings {
            folders_file: "folders.enc".to_string(),
            log_file: "plgrt.log".to_string(),
            keyword: "mustuflux".to_string(),
            service_url: "https://jerome.bousquie.fr/palangrotte/index.php".to_string(),
            canary_file_names: vec![
                "passwords".to_string(),
                "documentation".to_string(),
                "factures".to_string(),
                "confidentiel".to_string(),
                "profils".to_string(),
                "budget".to_string(),
                "personnel".to_string(),
                "secret".to_string(),
                "plans".to_string(),
                "groupes".to_string(),
                "autorisations".to_string(),
                "private".to_string(),
                "confidential".to_string(),
                "secret_keys".to_string(),
            ],
            canary_file_extensions: vec![
                "txt".to_string(),
                "pdf".to_string(),
                "docx".to_string(),
                "xlsx".to_string(),
                "pptx".to_string(),
                "jpg".to_string(),
                "png".to_string(),
            ],
            min_canary_files: 2,
            max_canary_files: 5,
            min_canary_file_size: 12288,
            max_canary_file_size: 122880,
            notification_title: "Security Alert".to_string(),
            notification_message: "A canary file has been modified. The system is shutting down."
                .to_string(),
        }
    }
}

/// Loads settings from the `palangrotte.toml` file.
/// If the file does not exist or is invalid, default settings are used.
pub fn load_settings() -> Settings {
    match fs::read_to_string("palangrotte.toml") {
        Ok(contents) => toml::from_str(&contents).unwrap_or_else(|e| {
            eprintln!(
                "Warning: Could not parse palangrotte.toml: {}. Using default settings.",
                e
            );
            Settings::default()
        }),
        Err(_) => {
            // File not found is okay, just use defaults.
            Settings::default()
        }
    }
}