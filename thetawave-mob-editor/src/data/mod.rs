//! Data management for the editor.
//!
//! - [`EditorSession`] - Current editing session state
//! - [`SpriteRegistry`] - Registry of available sprites
//! - [`MobAssetRegistry`] - Registry of registered mob assets
//! - [`AssetSource`] - Shared enum for base/game/mods asset origin

mod mob_asset_registry;
mod session;
mod sprite_registry;

/// Source of a registered asset (base, game, or mods).
///
/// Used by both [`SpriteRegistry`] and [`MobAssetRegistry`] to track
/// whether an asset comes from the base game, game assets, or mods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetSource {
    /// From the base assets directory (e.g., assets/)
    Base,
    /// From the game assets directory (e.g., thetawave-test-game/assets/)
    Game,
    /// From the mods assets directory (e.g., mods/)
    Mods,
}

impl AssetSource {
    /// Check if this is any non-base source (game or mods)
    pub fn is_extended(&self) -> bool {
        matches!(self, AssetSource::Game | AssetSource::Mods)
    }
}

pub(crate) use mob_asset_registry::{MobAssetRegistry, RegisteredMobAsset};
pub(crate) use session::{EditorSession, FileType};
pub(crate) use sprite_registry::{RegisteredSprite, SpriteRegistry};
