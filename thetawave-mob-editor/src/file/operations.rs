use std::{fs, io, path::PathBuf};

use bevy::prelude::*;
use toml::Value;

/// Errors that can occur during file operations
#[derive(Debug)]
pub enum FileError {
    Io(io::Error),
    Toml(toml::de::Error),
    TomlSer(toml::ser::Error),
    Trash(trash::Error),
    AlreadyExists,
    NotFound,
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

    /// Find the base .mob file for a .mobpatch file
    /// Looks for a .mob file with the same relative path in the base assets directory
    pub fn find_base_mob(patch_path: &PathBuf) -> Option<PathBuf> {
        let path_str = patch_path.to_string_lossy();

        // Extract the relative path after "mobs/"
        let mobs_idx = path_str.find("mobs/")?;
        let relative = &path_str[mobs_idx..]; // e.g., "mobs/xhitara/spitter.mobpatch"

        // Convert to .mob extension
        let base_relative = relative.strip_suffix(".mobpatch")?.to_string() + ".mob";

        // Get current working directory
        let cwd = std::env::current_dir().ok()?;

        // Search in base assets directory
        let base_path = cwd.join("assets").join(&base_relative);
        if base_path.exists() {
            return Some(base_path);
        }

        // Also check parent directory's assets (workspace root)
        let parent_base = cwd.parent()?.join("assets").join(&base_relative);
        if parent_base.exists() {
            return Some(parent_base);
        }

        None
    }

    /// Load a .mobpatch file and merge it with its base .mob file
    /// Returns (patch_value, base_value, merged_value)
    pub fn load_patch_with_base(patch_path: &PathBuf) -> Result<(Value, Option<Value>, Option<Value>), FileError> {
        let patch = Self::load_file(patch_path)?;

        // Try to find and load the base mob
        if let Some(base_path) = Self::find_base_mob(patch_path) {
            match Self::load_file(&base_path) {
                Ok(base) => {
                    // Merge patch into a copy of base
                    let mut merged = base.clone();
                    merge_toml_values(&mut merged, patch.clone());
                    info!("Merged patch with base mob from: {:?}", base_path);
                    Ok((patch, Some(base), Some(merged)))
                }
                Err(e) => {
                    warn!("Failed to load base mob {:?}: {}", base_path, e);
                    Ok((patch, None, None))
                }
            }
        } else {
            warn!("No base mob found for patch: {:?}", patch_path);
            Ok((patch, None, None))
        }
    }
}

/// Recursively merge TOML values, with extended values taking precedence over base values.
/// This handles tables, arrays, and primitive values correctly.
pub fn merge_toml_values(base: &mut Value, extended: Value) {
    match (base, extended) {
        // Both are tables - merge recursively
        (Value::Table(base_table), Value::Table(extended_table)) => {
            for (key, extended_value) in extended_table {
                match base_table.get_mut(&key) {
                    Some(base_value) => {
                        // Key exists in base - merge recursively
                        merge_toml_values(base_value, extended_value);
                    }
                    None => {
                        // Key doesn't exist in base - add it
                        base_table.insert(key, extended_value);
                    }
                }
            }
        }
        // For non-table values, extended completely replaces base
        (base, extended) => {
            *base = extended;
        }
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
