//! Asset management for mob definitions.
//!
//! This module provides the infrastructure for loading `.mob` files as Bevy assets
//! and building the runtime [`MobRegistry`] that resolves mob references.
//!
//! # Asset Loading Pipeline
//!
//! The mob asset system uses a 5-step pipeline to load and process mob definitions:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                        MOB ASSET LOADING PIPELINE                        │
//! └─────────────────────────────────────────────────────────────────────────┘
//!
//! Step 1: Load Base Mobs (AppState::GameLoading)
//! ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//!   mobs.assets.ron → MobAssetLoader → RawMob (unparsed TOML)
//!
//!   Base .mob files are loaded from embedded assets and stored as raw
//!   TOML values in RawMob assets, keyed by file stem (e.g., "xhitara/grunt").
//!
//! Step 2: Load Extended Mobs (AppState::GameLoading)
//! ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//!   extended://mobs.assets.ron → MobAssetLoader → RawMob
//!
//!   Extended .mob files are loaded from the filesystem via the "extended://"
//!   asset source. These can add new mobs or completely override base mobs.
//!
//! Step 3: Load Mob Patches (AppState::GameLoading)
//! ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//!   extended://mobs.assets.ron → MobPatchLoader → MobPatch
//!
//!   Extended .mobpatch files are partial TOML overrides that will be
//!   merged into base/extended mobs at field level.
//!
//! Step 4: Build MobRegistry (OnEnter(AppState::Game))
//! ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//!   a) Collect raw TOML values from base mobs
//!   b) Add/override with extended complete mobs
//!   c) Merge .mobpatch files into values (field-level override)
//!   d) Deserialize merged values to MobAsset structs
//!   e) Pre-build behavior trees for each mob
//!
//! Step 5: Runtime Access (AppState::Game)
//! ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//!   MobRegistry::get_mob("xhitara/grunt") → &MobAsset
//!   MobRegistry::get_behavior("xhitara/grunt") → &Tree<Behave>
//! ```
//!
//! # File Types
//!
//! ## `.mob` Files
//! Complete mob definitions in TOML format. Example:
//!
//! ```toml
//! name = "Xhitara Grunt"
//! colliders = [{ shape = { Rectangle = [12.0, 15.0] }, position = [0.0, 0.0], rotation = 0.0 }]
//! sprite_key = "xhitara_grunt_mob"
//!
//! [behavior]
//! type = "Forever"
//! [[behavior.children]]
//! type = "Action"
//! name = "Movement"
//! behaviors = [{ action = "MoveDown" }]
//! ```
//!
//! ## `.mobpatch` Files
//! Partial TOML overrides merged with base mobs. Only specified fields are updated:
//!
//! ```toml
//! name = "Super Fast Spitter"
//! projectile_speed = 300.0
//! [projectile_spawners.spawners.south]
//! timer = 0.25
//! ```
//!
//! # Type-Safe Mob References
//!
//! Use [`MobRef`] for compile-time safety when passing mob references:
//!
//! ```ignore
//! // MobRef automatically normalizes paths
//! let mob_ref = MobRef::new("mobs/xhitara/grunt.mob");
//! assert_eq!(mob_ref.as_str(), "xhitara/grunt");
//!
//! // Use in events
//! spawn_event_writer.write(SpawnMobEvent::new(
//!     MobRef::new("xhitara/grunt"),
//!     Vec2::ZERO,
//!     0.0,
//! ));
//! ```

mod error;
mod loader;
mod mob_asset;
mod mob_patch;
mod registry;

pub use loader::{MobAssetLoader, RawMob};
pub use mob_asset::{JointedMobRef, MobAsset};
pub use mob_patch::{MobPatch, MobPatchLoader};
pub use registry::{MobRef, MobRegistry};

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
