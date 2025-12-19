//! Asset management for mob definitions.
//!
//! This module provides the infrastructure for loading .mob files as Bevy assets.
//!
//! ## File Types
//! - `.mob` files: Complete mob definitions (used for base mobs and extended mobs)
//! - `.mobpatch` files: Partial overrides merged with base mobs (used for extended overrides)

mod error;
mod loader;
mod mob_asset;
mod mob_patch;
mod registry;

pub use loader::{MobAssetLoader, RawMob};
pub use mob_asset::{JointedMobRef, MobAsset};
pub use mob_patch::{MobPatch, MobPatchLoader};
pub use registry::{MobRegistry, normalize_mob_ref};

use bevy::{asset::Handle, platform::collections::HashMap, prelude::{Res, Resource}};
use bevy_asset_loader::{asset_collection::AssetCollection, mapped::AssetFileStem};

/// Base mob assets loaded from embedded assets.
///
/// These are raw TOML values that get merged with patches (if any)
/// and then deserialized to MobAsset in the MobRegistry.
#[derive(Resource, AssetCollection)]
pub struct MobAssets {
    /// All loaded raw mob definitions, keyed by file stem
    /// e.g., "xhitara/grunt" from "mobs/xhitara/grunt.mob"
    #[asset(key = "mobs", collection(typed, mapped))]
    pub mobs: HashMap<AssetFileStem, Handle<RawMob>>,
}

/// Extended complete mobs loaded from filesystem (optional).
///
/// These are complete mob definitions that add new mobs to the game.
/// Use .mob extension for these files.
#[derive(Resource, Default, AssetCollection)]
pub struct ExtendedMobs {
    /// Extended complete mobs, keyed by file stem
    /// e.g., "custom/my_enemy" from "mobs/custom/my_enemy.mob"
    #[asset(key = "extended_mobs", collection(typed, mapped), optional)]
    pub mobs: Option<HashMap<AssetFileStem, Handle<RawMob>>>,
}

/// Extended mob patches loaded from filesystem (optional).
///
/// These are partial TOML overrides that get merged with base mobs.
/// Use .mobpatch extension for these files.
#[derive(Resource, Default, AssetCollection)]
pub struct ExtendedMobPatches {
    /// Extended mob patches, keyed by file stem
    /// e.g., "xhitara/spitter" from "mobs/xhitara/spitter.mobpatch"
    #[asset(key = "extended_mob_patches", collection(typed, mapped), optional)]
    pub patches: Option<HashMap<AssetFileStem, Handle<MobPatch>>>,
}
