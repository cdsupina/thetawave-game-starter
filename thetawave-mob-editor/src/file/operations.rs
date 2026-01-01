use std::{fs, io, path::{Path, PathBuf}};

use bevy::log::{info, warn};
use thiserror::Error;
use toml::Value;

// Re-export from thetawave_core to avoid duplication
pub use thetawave_core::merge_toml_values;

/// Errors that can occur during file operations
#[derive(Debug, Error)]
pub enum FileError {
    /// IO error (reading/writing files).
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// TOML parsing error.
    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),

    /// TOML serialization error.
    #[error("TOML serialize error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    /// Error moving file to trash.
    #[error("Trash error: {0}")]
    Trash(#[from] trash::Error),

    /// File already exists (when creating new files).
    #[error("File already exists")]
    AlreadyExists,

    /// File not found.
    #[error("File not found")]
    NotFound,
}

/// File operations for mob files
pub struct FileOperations;

impl FileOperations {
    /// Load a .mob or .mobpatch file and return its TOML value
    pub fn load_file(path: &Path) -> Result<toml::Value, FileError> {
        if !path.exists() {
            return Err(FileError::NotFound);
        }

        let content = fs::read_to_string(path)?;
        let value: toml::Value = toml::from_str(&content)?;
        Ok(value)
    }

    /// Save a TOML value to a file
    pub fn save_file(path: &Path, value: &toml::Value) -> Result<(), FileError> {
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
    pub fn delete_file(path: &Path) -> Result<(), FileError> {
        if !path.exists() {
            return Err(FileError::NotFound);
        }

        trash::delete(path)?;
        Ok(())
    }

    /// Create a new mob file with default content
    pub fn create_new_file(
        path: &Path,
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
    pub fn find_base_mob(patch_path: &Path) -> Option<PathBuf> {
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
    pub fn load_patch_with_base(patch_path: &Path) -> Result<(Value, Option<Value>, Option<Value>), FileError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Helper to create a valid mob TOML value
    fn create_test_mob(name: &str) -> toml::Value {
        let toml_str = format!(
            r#"
            name = "{}"
            sprite = "media/aseprite/test.aseprite"
            spawnable = true
            health = 50

            [[colliders]]
            shape = {{ Rectangle = [10.0, 10.0] }}
            position = [0.0, 0.0]
            rotation = 0.0
            "#,
            name
        );
        toml::from_str(&toml_str).unwrap()
    }

    #[test]
    fn load_file_success() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test.mob");

        let content = r#"
            name = "Test Mob"
            sprite = "media/aseprite/test.aseprite"
        "#;
        fs::write(&path, content).unwrap();

        let result = FileOperations::load_file(&path);
        assert!(result.is_ok());

        let value = result.unwrap();
        assert_eq!(
            value.get("name").and_then(|v| v.as_str()),
            Some("Test Mob")
        );
    }

    #[test]
    fn load_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("nonexistent.mob");

        let result = FileOperations::load_file(&path);
        assert!(matches!(result, Err(FileError::NotFound)));
    }

    #[test]
    fn load_file_invalid_toml() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("invalid.mob");

        fs::write(&path, "this is not valid toml [[[").unwrap();

        let result = FileOperations::load_file(&path);
        assert!(matches!(result, Err(FileError::Toml(_))));
    }

    #[test]
    fn save_file_creates_parent_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("nested/dir/test.mob");

        let value = create_test_mob("Test");

        let result = FileOperations::save_file(&path, &value);
        assert!(result.is_ok());
        assert!(path.exists());

        // Verify content was written correctly
        let loaded = FileOperations::load_file(&path).unwrap();
        assert_eq!(
            loaded.get("name").and_then(|v| v.as_str()),
            Some("Test")
        );
    }

    #[test]
    fn save_file_atomic_write() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("atomic.mob");

        let value = create_test_mob("Atomic Test");

        let result = FileOperations::save_file(&path, &value);
        assert!(result.is_ok());

        // Verify temp file was cleaned up
        let temp_path = path.with_extension("tmp");
        assert!(!temp_path.exists());

        // Verify main file exists and has correct content
        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("Atomic Test"));
    }

    #[test]
    fn create_new_file_mob() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("new_mob.mob");

        let result = FileOperations::create_new_file(&path, "New Mob", false);
        assert!(result.is_ok());
        assert!(path.exists());

        let value = result.unwrap();
        assert_eq!(
            value.get("name").and_then(|v| v.as_str()),
            Some("New Mob")
        );
        // Full mobs should have default sprite, health, colliders
        assert!(value.get("sprite").is_some());
        assert!(value.get("health").is_some());
        assert!(value.get("colliders").is_some());
    }

    #[test]
    fn create_new_file_patch() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("new_patch.mobpatch");

        let result = FileOperations::create_new_file(&path, "My Patch", true);
        assert!(result.is_ok());
        assert!(path.exists());

        let value = result.unwrap();
        assert_eq!(
            value.get("name").and_then(|v| v.as_str()),
            Some("My Patch")
        );
        // Patches should be minimal - only name
        assert!(value.get("sprite").is_none());
        assert!(value.get("health").is_none());
    }

    #[test]
    fn create_new_file_already_exists() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("existing.mob");

        // Create file first
        fs::write(&path, "name = \"Existing\"").unwrap();

        let result = FileOperations::create_new_file(&path, "Should Fail", false);
        assert!(matches!(result, Err(FileError::AlreadyExists)));
    }

    #[test]
    fn delete_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("nonexistent.mob");

        let result = FileOperations::delete_file(&path);
        assert!(matches!(result, Err(FileError::NotFound)));
    }

    // Note: delete_file_success test is skipped because trash::delete
    // behavior varies across platforms and CI environments

    #[test]
    fn find_base_mob_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let patch_path = temp_dir.path().join("mobs/test/enemy.mobpatch");

        // No base mob exists
        let result = FileOperations::find_base_mob(&patch_path);
        assert!(result.is_none());
    }

    #[test]
    fn load_patch_with_base_no_base() {
        let temp_dir = TempDir::new().unwrap();
        let patch_path = temp_dir.path().join("mobs/test/enemy.mobpatch");

        // Create parent dirs and patch file
        fs::create_dir_all(patch_path.parent().unwrap()).unwrap();
        fs::write(&patch_path, "name = \"Patched Enemy\"\nhealth = 200").unwrap();

        let result = FileOperations::load_patch_with_base(&patch_path);
        assert!(result.is_ok());

        let (patch, base, merged) = result.unwrap();
        assert_eq!(
            patch.get("name").and_then(|v| v.as_str()),
            Some("Patched Enemy")
        );
        assert!(base.is_none());
        assert!(merged.is_none());
    }

    #[test]
    fn toml_merge_basic() {
        let mut base = toml::from_str::<toml::Value>(
            r#"
            name = "Base"
            health = 100
            speed = 50
            "#,
        )
        .unwrap();

        let patch = toml::from_str::<toml::Value>(
            r#"
            name = "Patched"
            health = 200
            "#,
        )
        .unwrap();

        merge_toml_values(&mut base, patch);

        assert_eq!(base.get("name").and_then(|v| v.as_str()), Some("Patched"));
        assert_eq!(base.get("health").and_then(|v| v.as_integer()), Some(200));
        assert_eq!(base.get("speed").and_then(|v| v.as_integer()), Some(50));
    }

    #[test]
    fn toml_merge_nested() {
        let mut base = toml::from_str::<toml::Value>(
            r#"
            [spawner]
            timer = 1.0
            count = 3
            "#,
        )
        .unwrap();

        let patch = toml::from_str::<toml::Value>(
            r#"
            [spawner]
            timer = 0.5
            "#,
        )
        .unwrap();

        merge_toml_values(&mut base, patch);

        let spawner = base.get("spawner").and_then(|v| v.as_table()).unwrap();
        assert_eq!(spawner.get("timer").and_then(|v| v.as_float()), Some(0.5));
        assert_eq!(spawner.get("count").and_then(|v| v.as_integer()), Some(3));
    }
}
