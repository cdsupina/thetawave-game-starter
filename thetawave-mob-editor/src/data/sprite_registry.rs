//! Sprite registry for tracking available sprites.
//!
//! The [`SpriteRegistry`] maintains a list of all registered sprites
//! from base, game, and mods `game.assets.ron` files.

use bevy::prelude::Resource;

use super::AssetSource;

/// A registered sprite entry
#[derive(Debug, Clone)]
pub struct RegisteredSprite {
    /// The path as stored in .mob files (e.g., "media/aseprite/xhitara_grunt_mob.aseprite")
    pub asset_path: String,
    /// Display name for UI (file stem, e.g., "xhitara_grunt_mob")
    pub display_name: String,
    /// Whether this is from base, game, or mods assets
    pub source: AssetSource,
}

impl RegisteredSprite {
    /// Get the path to use when saving to a .mob file
    pub fn mob_path(&self) -> String {
        self.asset_path.clone()
    }

    /// Get the path to use when saving to a .mobpatch file
    /// Game sprites get the game:// prefix, mods sprites get mods:// prefix
    pub fn mobpatch_path(&self) -> String {
        match self.source {
            AssetSource::Game => format!("game://{}", self.asset_path),
            AssetSource::Mods => format!("mods://{}", self.asset_path),
            AssetSource::Base => self.asset_path.clone(),
        }
    }
}

/// Resource containing all registered sprites from .assets.ron files
#[derive(Resource, Default)]
pub struct SpriteRegistry {
    /// All registered sprites
    pub sprites: Vec<RegisteredSprite>,
    /// Whether the registry needs to be rescanned
    pub needs_refresh: bool,
    /// Parse errors encountered during scanning
    pub parse_errors: Vec<String>,
}

impl SpriteRegistry {
    /// Find a sprite by its asset path (with or without game:// or mods:// prefix)
    ///
    /// When the path has a `game://` prefix, only game sprites are matched.
    /// When the path has a `mods://` prefix, only mods sprites are matched.
    /// When the path has no prefix, only base sprites are matched.
    /// This ensures that base, game, and mods sprites with the same relative path
    /// are treated as distinct entries.
    pub fn find_by_path(&self, path: &str) -> Option<&RegisteredSprite> {
        if let Some(normalized) = path.strip_prefix("game://") {
            // Path has game:// prefix - only match game sprites
            self.sprites
                .iter()
                .find(|s| s.asset_path == normalized && s.source == AssetSource::Game)
        } else if let Some(normalized) = path.strip_prefix("mods://") {
            // Path has mods:// prefix - only match mods sprites
            self.sprites
                .iter()
                .find(|s| s.asset_path == normalized && s.source == AssetSource::Mods)
        } else {
            // No prefix - only match base sprites
            self.sprites
                .iter()
                .find(|s| s.asset_path == path && s.source == AssetSource::Base)
        }
    }

    /// Check if a sprite path is registered
    pub fn is_registered(&self, path: &str) -> bool {
        self.find_by_path(path).is_some()
    }

    /// Get display name for a path, or the path itself if not found
    pub fn display_name_for(&self, path: &str) -> String {
        self.find_by_path(path)
            .map(|s| s.display_name.clone())
            .unwrap_or_else(|| {
                // Extract file stem as fallback display name
                std::path::Path::new(path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(path)
                    .to_string()
            })
    }

    /// Get all base sprites
    pub fn base_sprites(&self) -> impl Iterator<Item = &RegisteredSprite> {
        self.sprites
            .iter()
            .filter(|s| s.source == AssetSource::Base)
    }

    /// Get all game sprites
    pub fn game_sprites(&self) -> impl Iterator<Item = &RegisteredSprite> {
        self.sprites
            .iter()
            .filter(|s| s.source == AssetSource::Game)
    }

    /// Get all mods sprites
    pub fn mods_sprites(&self) -> impl Iterator<Item = &RegisteredSprite> {
        self.sprites
            .iter()
            .filter(|s| s.source == AssetSource::Mods)
    }

    /// Get all extended sprites (game + mods, for backwards compatibility)
    pub fn extended_sprites(&self) -> impl Iterator<Item = &RegisteredSprite> {
        self.sprites.iter().filter(|s| s.source.is_extended())
    }
}
