//! # Settings Module
//! This module defines constants for configuration, such as file names and service URLs.

/// The name of the file containing the folders to monitor.
pub const FOLDERS_FILE: &str = "folders.enc";
/// The name of the log file.
pub const LOG_FILE: &str = "plgrt.log";
/// The URL of the service to send notifications to.
pub const SERVICE_URL: &str = "https://jerome.bousquie.fr/palangrotte/index.php";
/// The canary file names
pub const CANARY_FILE_NAMES: &[&str] = &[
    "passwords",
    "documentation",
    "factures",
    "confidentiel",
    "profils",
    "budget",
    "personnel",
    "secret",
    "plans",
    "groupes",
    "autorisations",
];
/// The canary file extensions
pub const CANARY_FILE_EXTENSIONS: &[&str] = &["txt", "pdf", "docx", "xlsx", "pptx", "jpg", "png"];
/// The minimum number of canary files to create in a folder.
pub const MIN_CANARY_FILES: usize = 2;
/// The maximum number of canary files to create in a folder.
pub const MAX_CANARY_FILES: usize = 5;
/// The minimum size of a canary file in bytes.
pub const MIN_CANARY_FILE_SIZE: usize = 12 * 1024;
/// The maximum size of a canary file in bytes.
pub const MAX_CANARY_FILE_SIZE: usize = 120 * 1024;
