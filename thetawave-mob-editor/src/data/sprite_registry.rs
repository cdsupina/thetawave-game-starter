use bevy::prelude::*;

/// Source of a registered sprite
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteSource {
    /// From assets/game.assets.ron
    Base,
    /// From thetawave-test-game/assets/game.assets.ron
    Extended,
}

/// A registered sprite entry
#[derive(Debug, Clone)]
pub struct RegisteredSprite {
    /// The path as stored in .mob files (e.g., "media/aseprite/xhitara_grunt_mob.aseprite")
    pub asset_path: String,
    /// Display name for UI (file stem, e.g., "xhitara_grunt_mob")
    pub display_name: String,
    /// Whether this is from base or extended assets
    pub source: SpriteSource,
}

impl RegisteredSprite {
    /// Get the path to use when saving to a .mob file
    pub fn mob_path(&self) -> String {
        self.asset_path.clone()
    }

    /// Get the path to use when saving to a .mobpatch file
    /// Extended sprites get the extended:// prefix
    pub fn mobpatch_path(&self) -> String {
        match self.source {
            SpriteSource::Extended => format!("extended://{}", self.asset_path),
            SpriteSource::Base => self.asset_path.clone(),
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
    /// Find a sprite by its asset path (with or without extended:// prefix)
    pub fn find_by_path(&self, path: &str) -> Option<&RegisteredSprite> {
        let normalized = path.strip_prefix("extended://").unwrap_or(path);
        self.sprites.iter().find(|s| s.asset_path == normalized)
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
            .filter(|s| s.source == SpriteSource::Base)
    }

    /// Get all extended sprites
    pub fn extended_sprites(&self) -> impl Iterator<Item = &RegisteredSprite> {
        self.sprites
            .iter()
            .filter(|s| s.source == SpriteSource::Extended)
    }
}
