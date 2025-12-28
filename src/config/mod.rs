//! Configuration management for lazydb
//!
//! This module handles loading and saving configuration files, including:
//! - Main configuration file (`config.yaml`)
//! - Project configuration files (`projects/*.yaml`)
//! - Query history persistence
//!
//! # Configuration Directory
//!
//! Configuration files are stored in the OS-specific config directory:
//! - Linux/macOS: `~/.config/lazydb/`
//! - Windows: `%APPDATA%\lazydb\`

mod loader;
mod models;

pub use loader::ConfigLoader;
// These types are part of the public API and may be used by external consumers
#[allow(unused_imports)]
pub use models::{Config, ConnectionConfig, ProjectConfig, ProjectFile, Settings};
