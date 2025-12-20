use std::{fs, io, path::PathBuf};

use bevy::prelude::*;

/// Errors that can occur during file operations
#[derive(Debug)]
pub enum FileError {
    Io(io::Error),
    Toml(toml::de::Error),
    TomlSer(toml::ser::Error),
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

impl std::fmt::Display for FileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileError::Io(e) => write!(f, "IO error: {}", e),
            FileError::Toml(e) => write!(f, "TOML parse error: {}", e),
            FileError::TomlSer(e) => write!(f, "TOML serialize error: {}", e),
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

    /// Save a TOML value to a file with backup
    pub fn save_file(path: &PathBuf, value: &toml::Value) -> Result<(), FileError> {
        // Create backup if file exists
        if path.exists() {
            Self::create_backup(path)?;
        }

        // Serialize to TOML string
        let content = toml::to_string_pretty(value)?;

        // Write atomically using temp file
        let temp_path = path.with_extension("tmp");
        fs::write(&temp_path, content)?;
        fs::rename(&temp_path, path)?;

        Ok(())
    }

    /// Create a backup of an existing file
    pub fn create_backup(path: &PathBuf) -> Result<(), FileError> {
        if path.exists() {
            let backup_path = path.with_extension(format!(
                "{}.bak",
                path.extension()
                    .map(|e| e.to_string_lossy().to_string())
                    .unwrap_or_default()
            ));
            fs::copy(path, backup_path)?;
        }
        Ok(())
    }

    /// Delete a file (moves to .deleted folder for recovery)
    pub fn delete_file(path: &PathBuf) -> Result<(), FileError> {
        if !path.exists() {
            return Err(FileError::NotFound);
        }

        let Some(parent) = path.parent() else {
            return Err(FileError::InvalidPath);
        };

        let deleted_dir = parent.join(".deleted");
        fs::create_dir_all(&deleted_dir)?;

        let Some(filename) = path.file_name() else {
            return Err(FileError::InvalidPath);
        };

        let deleted_path = deleted_dir.join(filename);

        // If a deleted version already exists, add a timestamp
        let final_path = if deleted_path.exists() {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            deleted_dir.join(format!(
                "{}.{}",
                timestamp,
                filename.to_string_lossy()
            ))
        } else {
            deleted_path
        };

        fs::rename(path, final_path)?;
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

    /// Validate a mob TOML value (basic validation)
    pub fn validate(value: &toml::Value) -> Vec<String> {
        let mut errors = Vec::new();

        if let Some(table) = value.as_table() {
            // Check for required name field
            if !table.contains_key("name") {
                errors.push("Missing required field: name".to_string());
            } else if let Some(name) = table.get("name") {
                if let Some(s) = name.as_str() {
                    if s.is_empty() {
                        errors.push("Name cannot be empty".to_string());
                    }
                } else {
                    errors.push("Name must be a string".to_string());
                }
            }

            // Validate health if present
            if let Some(health) = table.get("health") {
                if let Some(h) = health.as_integer() {
                    if h <= 0 {
                        errors.push("Health must be positive".to_string());
                    }
                } else {
                    errors.push("Health must be an integer".to_string());
                }
            }

            // Validate colliders if present
            if let Some(colliders) = table.get("colliders") {
                if let Some(arr) = colliders.as_array() {
                    for (i, collider) in arr.iter().enumerate() {
                        if let Some(c) = collider.as_table() {
                            if !c.contains_key("shape") {
                                errors.push(format!("Collider {} missing shape", i));
                            }
                        } else {
                            errors.push(format!("Collider {} is not a table", i));
                        }
                    }
                }
            }
        } else {
            errors.push("Root must be a TOML table".to_string());
        }

        errors
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
