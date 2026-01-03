//! Data management for the editor.
//!
//! - [`EditorSession`] - Current editing session state
//! - [`SpriteRegistry`] - Registry of available sprites
//! - [`MobAssetRegistry`] - Registry of registered mob assets

mod mob_asset_registry;
mod session;
mod sprite_registry;

pub(crate) use mob_asset_registry::{MobAssetRegistry, MobAssetSource, RegisteredMobAsset};
pub(crate) use session::{EditorSession, FileType};
pub(crate) use sprite_registry::{RegisteredSprite, SpriteRegistry, SpriteSource};
