//! # Palangrotte Library
//! This crate provides the core logic for the canary file monitoring daemon.
//! It includes modules for handling canary files, logging, settings, encryption, and notifications.

pub mod canary;
pub mod logger;
pub mod settings;
pub mod encryption;
pub mod notify_access;
pub mod linux_notification;
