//! Registry for tracking registered mob assets.
//!
//! The [`MobAssetRegistry`] maintains a list of all registered mobs and patches
//! from base, game, and mods `mobs.assets.ron` files.

use std::path::Path;

use bevy::prelude::Resource;

use super::AssetSource;
use crate::plugin::EditorConfig;

/// A registered mob asset entry
#[derive(Debug, Clone)]
pub struct RegisteredMobAsset {
    /// The path as stored in .assets.ron files (e.g., "mobs/xhitara/grunt.mob")
    /// For game/mods assets, this is without the prefix
    pub asset_path: String,
    /// Display name for UI (file stem, e.g., "grunt")
    pub display_name: String,
    /// Whether this is from base, game, or mods assets
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
    ///
    /// Priority order for path classification (first match wins):
    /// 1. Mods paths (mods_assets_dir)
    /// 2. Game paths (game_assets_dir)
    /// 3. Base paths (default)
    ///
    /// This ensures mod overrides are correctly identified when paths could match multiple sources.
    pub fn get_registration(
        &self,
        path: &Path,
        config: &EditorConfig,
    ) -> Option<&RegisteredMobAsset> {
        // Determine the source type
        let source = if config.is_mods_path(path) {
            AssetSource::Mods
        } else if config.is_game_path(path) {
            AssetSource::Game
        } else {
            AssetSource::Base
        };

        // Calculate the relative path based on source
        let relative_path = self.calculate_relative_path(path, config, source)?;

        // Find matching entry
        self.entries
            .iter()
            .find(|entry| entry.source == source && entry.asset_path == relative_path)
    }

    /// Calculate the relative path for a mob file as it would appear in mobs.assets.ron
    fn calculate_relative_path(
        &self,
        path: &Path,
        config: &EditorConfig,
        source: AssetSource,
    ) -> Option<String> {
        let assets_root = match source {
            AssetSource::Mods => config.mods_assets_root()?,
            AssetSource::Game => config.game_assets_root()?,
            AssetSource::Base => config.base_assets_root()?,
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

    /// Get all game entries
    pub fn game_entries(&self) -> impl Iterator<Item = &RegisteredMobAsset> {
        self.entries
            .iter()
            .filter(|e| e.source == AssetSource::Game)
    }

    /// Get all mods entries
    pub fn mods_entries(&self) -> impl Iterator<Item = &RegisteredMobAsset> {
        self.entries
            .iter()
            .filter(|e| e.source == AssetSource::Mods)
    }
}
