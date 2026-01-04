//! Registry for tracking registered mob assets.
//!
//! The [`MobAssetRegistry`] maintains a list of all registered mobs and patches
//! from both base and extended `mobs.assets.ron` files.

use std::path::Path;

use bevy::prelude::Resource;

use super::AssetSource;
use crate::plugin::EditorConfig;

/// A registered mob asset entry
#[derive(Debug, Clone)]
pub struct RegisteredMobAsset {
    /// The path as stored in .assets.ron files (e.g., "mobs/xhitara/grunt.mob")
    /// For extended assets, this is without the "extended://" prefix
    pub asset_path: String,
    /// Display name for UI (file stem, e.g., "grunt")
    pub display_name: String,
    /// Whether this is from base or extended assets
    pub source: AssetSource,
}

/// Resource containing all registered mob assets from mobs.assets.ron files
#[derive(Resource, Default)]
pub struct MobAssetRegistry {
    /// All registered mob assets
    pub entries: Vec<RegisteredMobAsset>,
    /// Whether the registry needs to be rescanned
    pub needs_refresh: bool,
    /// Parse errors encountered during scanning
    pub parse_errors: Vec<String>,
}

impl MobAssetRegistry {
    /// Check if a file path is registered
    ///
    /// Converts the absolute file path to a relative path and checks if it
    /// exists in the registry.
    pub fn is_registered(&self, path: &Path, config: &EditorConfig) -> bool {
        self.get_registration(path, config).is_some()
    }

    /// Get registration info for a file path
    ///
    /// Returns the registration entry if the file is registered, None otherwise.
    pub fn get_registration(
        &self,
        path: &Path,
        config: &EditorConfig,
    ) -> Option<&RegisteredMobAsset> {
        // Determine if this is an extended path
        let is_extended = config.is_extended_path(path);

        // Calculate the relative path
        let relative_path = self.calculate_relative_path(path, config, is_extended)?;

        // Find matching entry
        self.entries.iter().find(|entry| {
            let source_matches = matches!(
                (is_extended, entry.source),
                (true, AssetSource::Extended) | (false, AssetSource::Base)
            );

            source_matches && entry.asset_path == relative_path
        })
    }

    /// Calculate the relative path for a mob file as it would appear in mobs.assets.ron
    fn calculate_relative_path(
        &self,
        path: &Path,
        config: &EditorConfig,
        is_extended: bool,
    ) -> Option<String> {
        let assets_root = if is_extended {
            config.extended_assets_root()?
        } else {
            config.base_assets_root()?
        };

        // Strip the assets root to get the relative path
        path.strip_prefix(&assets_root)
            .ok()
            .map(|p| p.to_string_lossy().to_string())
    }

    /// Get all base entries
    pub fn base_entries(&self) -> impl Iterator<Item = &RegisteredMobAsset> {
        self.entries
            .iter()
            .filter(|e| e.source == AssetSource::Base)
    }

    /// Get all extended entries
    pub fn extended_entries(&self) -> impl Iterator<Item = &RegisteredMobAsset> {
        self.entries
            .iter()
            .filter(|e| e.source == AssetSource::Extended)
    }
}
