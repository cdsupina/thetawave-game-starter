use std::{fs, io, path::PathBuf};

use bevy::prelude::*;

/// Errors that can occur during file operations
#[derive(Debug)]
pub enum FileError {
    Io(io::Error),
    Toml(toml::de::Error),
    TomlSer(toml::ser::Error),
    Trash(trash::Error),
    AlreadyExists,
    NotFound,
    InvalidPath,
}

impl From<io::Error> for FileError {
    fn from(e: io::Error) -> Self {
        FileError::Io(e)
    }
}

impl From<toml::de::Error> for FileError {
    fn from(e: toml::de::Error) -> Self {
        FileError::Toml(e)
    }
}

impl From<toml::ser::Error> for FileError {
    fn from(e: toml::ser::Error) -> Self {
        FileError::TomlSer(e)
    }
}

impl From<trash::Error> for FileError {
    fn from(e: trash::Error) -> Self {
        FileError::Trash(e)
    }
}

impl std::fmt::Display for FileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileError::Io(e) => write!(f, "IO error: {}", e),
            FileError::Toml(e) => write!(f, "TOML parse error: {}", e),
            FileError::TomlSer(e) => write!(f, "TOML serialize error: {}", e),
            FileError::Trash(e) => write!(f, "Trash error: {}", e),
            FileError::AlreadyExists => write!(f, "File already exists"),
            FileError::NotFound => write!(f, "File not found"),
            FileError::InvalidPath => write!(f, "Invalid file path"),
        }
    }
}

/// File operations for mob files
pub struct FileOperations;

impl FileOperations {
    /// Load a .mob or .mobpatch file and return its TOML value
    pub fn load_file(path: &PathBuf) -> Result<toml::Value, FileError> {
        if !path.exists() {
            return Err(FileError::NotFound);
        }

        let content = fs::read_to_string(path)?;
        let value: toml::Value = toml::from_str(&content)?;
        Ok(value)
    }

    /// Save a TOML value to a file
    pub fn save_file(path: &PathBuf, value: &toml::Value) -> Result<(), FileError> {
        // Serialize to TOML string
        let content = toml::to_string_pretty(value)?;

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write atomically using temp file
        let temp_path = path.with_extension("tmp");
        fs::write(&temp_path, &content)?;
        fs::rename(&temp_path, path)?;

        Ok(())
    }

    /// Delete a file (moves to system trash for recovery)
    pub fn delete_file(path: &PathBuf) -> Result<(), FileError> {
        if !path.exists() {
            return Err(FileError::NotFound);
        }

        trash::delete(path)?;
        Ok(())
    }

    /// Create a new mob file with default content
    pub fn create_new_file(
        path: &PathBuf,
        name: &str,
        is_patch: bool,
    ) -> Result<toml::Value, FileError> {
        if path.exists() {
            return Err(FileError::AlreadyExists);
        }

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let value = if is_patch {
            // Minimal patch file
            let mut table = toml::value::Table::new();
            table.insert("name".to_string(), toml::Value::String(name.to_string()));
            toml::Value::Table(table)
        } else {
            // Full mob file with defaults
            crate::data::EditorSession::new_mob(name)
        };

        Self::save_file(path, &value)?;
        Ok(value)
    }

    /// Validate a mob TOML value
    /// Returns error messages as strings for backward compatibility
    pub fn validate(value: &toml::Value) -> Vec<String> {
        let result = super::validation::validate_mob(value, false);
        result.error_messages()
    }

    /// Validate a mob TOML value with full result
    pub fn validate_full(value: &toml::Value, is_patch: bool) -> super::validation::ValidationResult {
        super::validation::validate_mob(value, is_patch)
    }
}

/// Message to request loading a mob file
#[derive(bevy::ecs::message::Message)]
pub struct LoadMobEvent {
    pub path: PathBuf,
}

/// Message to request saving the current mob
#[derive(bevy::ecs::message::Message)]
pub struct SaveMobEvent {
    pub path: Option<PathBuf>, // None = save to current path
}

/// Message to request reloading the current mob from disk
#[derive(bevy::ecs::message::Message)]
pub struct ReloadMobEvent;

/// Message to create a new mob file
#[derive(bevy::ecs::message::Message)]
pub struct NewMobEvent {
    pub path: PathBuf,
    pub name: String,
    pub is_patch: bool,
}

/// Message to delete a mob file
#[derive(bevy::ecs::message::Message)]
pub struct DeleteMobEvent {
    pub path: PathBuf,
}
