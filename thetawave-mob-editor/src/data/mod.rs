//! Data management for the editor.
//!
//! - [`EditorSession`] - Current editing session state
//! - [`SpriteRegistry`] - Registry of available sprites
//! - [`MobAssetRegistry`] - Registry of registered mob assets
//! - [`AssetSource`] - Shared enum for base vs extended asset origin

mod mob_asset_registry;
mod session;
mod sprite_registry;

/// Source of a registered asset (base or extended).
///
/// Used by both [`SpriteRegistry`] and [`MobAssetRegistry`] to track
/// whether an asset comes from the base game or extended assets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetSource {
    /// From the base assets directory (e.g., assets/)
    Base,
    /// From the extended assets directory (e.g., thetawave-test-game/assets/)
    Extended,
}

pub(crate) use mob_asset_registry::{MobAssetRegistry, RegisteredMobAsset};
pub(crate) use session::{EditorSession, FileType};
pub(crate) use sprite_registry::{RegisteredSprite, SpriteRegistry};
