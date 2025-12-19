//! MobRegistry resource for resolving mob references to loaded assets.

use bevy::{
    asset::{Asset, Assets, Handle},
    log::{debug, info, warn},
    platform::collections::HashMap,
    prelude::Resource,
};
use bevy_behave::{Behave, prelude::Tree};
use thetawave_core::merge_toml_values;

use super::{ExtendedMobPatches, ExtendedMobs, MobAsset, MobAssets, MobPatch, RawMob};
use crate::behavior::build_behavior_tree;

/// Extract the normalized key from any asset handle's path.
fn get_normalized_key<T: Asset>(handle: &Handle<T>) -> Option<String> {
    handle
        .path()
        .map(|p| normalize_mob_ref(p.path().to_str().unwrap_or("")))
}

/// Runtime registry that resolves mob references to loaded assets.
///
/// This is built after all MobAssets are loaded and provides:
/// - Fast lookup of mob assets by reference path
/// - Pre-built behavior trees for each mob
/// - Support for field-level merging of extended mobs with base mobs
#[derive(Resource)]
pub struct MobRegistry {
    /// Map from normalized key (e.g., "xhitara/grunt") to mob asset
    /// Note: We store MobAsset directly instead of handles because merged
    /// mobs are created at runtime and don't exist in the asset system.
    mobs: HashMap<String, MobAsset>,
    /// Pre-built behavior trees for each mob
    behaviors: HashMap<String, Tree<Behave>>,
}

impl MobRegistry {
    /// Build the registry from loaded RawMob assets and MobPatches.
    ///
    /// Processing order:
    /// 1. Collect raw TOML values from base mobs
    /// 2. Add extended mobs (new complete mobs)
    /// 3. Merge patches into base/extended values (if patches exist)
    /// 4. Deserialize merged values to MobAsset
    /// 5. Build behavior trees for all mobs
    pub fn build(
        base_assets: &MobAssets,
        extended_mobs: &ExtendedMobs,
        extended_patches: &ExtendedMobPatches,
        raw_mob_assets: &Assets<RawMob>,
        patch_assets: &Assets<MobPatch>,
    ) -> Self {
        info!(
            "Building MobRegistry: {} base handles, extended_mobs is {}, extended_patches is {}",
            base_assets.mobs.len(),
            if extended_mobs.mobs.is_some() { "Some" } else { "None" },
            if extended_patches.patches.is_some() { "Some" } else { "None" }
        );

        let mut raw_values = Self::collect_base_mobs(base_assets, raw_mob_assets);
        Self::add_extended_mobs(&mut raw_values, extended_mobs, raw_mob_assets);
        Self::apply_patches(&mut raw_values, extended_patches, patch_assets);
        let mobs = Self::deserialize_all(raw_values);
        let behaviors = Self::build_all_behaviors(&mobs);

        Self { mobs, behaviors }
    }

    /// Collect raw TOML values from base mob assets.
    fn collect_base_mobs(
        base_assets: &MobAssets,
        raw_mob_assets: &Assets<RawMob>,
    ) -> HashMap<String, toml::Value> {
        let mut raw_values = HashMap::new();

        for (stem, handle) in &base_assets.mobs {
            let Some(key) = get_normalized_key(handle) else {
                warn!("Base mob handle has no path, stem: {:?}", stem);
                continue;
            };

            if let Some(raw_mob) = raw_mob_assets.get(handle) {
                debug!("Loaded base mob: {}", key);
                raw_values.insert(key, raw_mob.value.clone());
            } else {
                warn!("Could not get base mob asset for: {}", key);
            }
        }

        info!("Loaded {} base mob values", raw_values.len());
        raw_values
    }

    /// Add extended mobs (complete new mobs) to the raw values map.
    fn add_extended_mobs(
        raw_values: &mut HashMap<String, toml::Value>,
        extended_mobs: &ExtendedMobs,
        raw_mob_assets: &Assets<RawMob>,
    ) {
        let Some(extended) = &extended_mobs.mobs else {
            info!("No extended mobs to process");
            return;
        };

        info!("Processing {} extended mobs", extended.len());

        for (_stem, handle) in extended {
            let Some(key) = get_normalized_key(handle) else {
                warn!("Extended mob handle has no path");
                continue;
            };

            if let Some(raw_mob) = raw_mob_assets.get(handle) {
                if raw_values.contains_key(&key) {
                    info!("Extended mob '{}' overrides base mob", key);
                } else {
                    debug!("Adding extended mob: {}", key);
                }
                raw_values.insert(key, raw_mob.value.clone());
            } else {
                warn!("Could not get extended mob asset for: {}", key);
            }
        }
    }

    /// Apply patches to base/extended mob values.
    fn apply_patches(
        raw_values: &mut HashMap<String, toml::Value>,
        extended_patches: &ExtendedMobPatches,
        patch_assets: &Assets<MobPatch>,
    ) {
        let Some(patches) = &extended_patches.patches else {
            info!("No extended mob patches to process");
            return;
        };

        info!("Processing {} extended mob patches", patches.len());
        let mut merged_count = 0;

        for (_stem, handle) in patches {
            let Some(key) = get_normalized_key(handle) else {
                warn!("Patch handle has no path");
                continue;
            };

            let Some(patch) = patch_assets.get(handle) else {
                warn!("Could not get patch asset for: {}", key);
                continue;
            };

            if let Some(base_value) = raw_values.get_mut(&key) {
                info!("Merging patch '{}' into base mob", key);
                merge_toml_values(base_value, patch.value.clone());
                merged_count += 1;
            } else {
                warn!(
                    "No base mob found for patch '{}', skipping (use .mob for new mobs)",
                    key
                );
            }
        }

        if merged_count > 0 {
            info!("Merged {} patches into base mobs", merged_count);
        }
    }

    /// Deserialize all raw TOML values to MobAsset structs.
    fn deserialize_all(raw_values: HashMap<String, toml::Value>) -> HashMap<String, MobAsset> {
        let mut mobs = HashMap::new();

        for (key, value) in raw_values {
            match value.try_into::<MobAsset>() {
                Ok(mob) => {
                    mobs.insert(key, mob);
                }
                Err(e) => {
                    warn!("Failed to deserialize mob '{}': {}", key, e);
                }
            }
        }

        info!("Deserialized {} mobs", mobs.len());
        mobs
    }

    /// Build behavior trees for all mobs that have behavior definitions.
    fn build_all_behaviors(mobs: &HashMap<String, MobAsset>) -> HashMap<String, Tree<Behave>> {
        let mut behaviors = HashMap::new();

        for (key, mob) in mobs {
            if let Some(behavior_data) = &mob.behavior {
                behaviors.insert(key.clone(), build_behavior_tree(behavior_data));
            }
        }

        behaviors
    }

    /// Get a mob asset by its reference path.
    ///
    /// Accepts paths like:
    /// - "mobs/ferritharax/body.mob" (full path)
    /// - "ferritharax/body" (normalized key)
    ///
    /// Returns None if the mob is not found.
    pub fn get_mob(&self, mob_ref: &str) -> Option<&MobAsset> {
        let key = normalize_mob_ref(mob_ref);
        self.mobs.get(&key)
    }

    /// Get the behavior tree for a mob.
    ///
    /// Returns None if the mob doesn't have a behavior or doesn't exist.
    pub fn get_behavior(&self, mob_ref: &str) -> Option<&Tree<Behave>> {
        let key = normalize_mob_ref(mob_ref);
        self.behaviors.get(&key)
    }

    /// Check if a mob exists in the registry.
    pub fn contains(&self, mob_ref: &str) -> bool {
        let key = normalize_mob_ref(mob_ref);
        self.mobs.contains_key(&key)
    }

    /// Get all registered mob keys.
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.mobs.keys()
    }

    /// Get all spawnable mobs (for debug spawn menu).
    /// Filters out mobs with `spawnable = false` (e.g., jointed parts).
    pub fn spawnable_mobs(&self) -> impl Iterator<Item = (&String, &MobAsset)> {
        self.mobs.iter().filter(|(_, mob)| mob.spawnable)
    }

    /// Get the number of registered mobs.
    pub fn len(&self) -> usize {
        self.mobs.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.mobs.is_empty()
    }
}

/// Normalize a mob reference path to a registry key.
///
/// Examples:
/// - "mobs/ferritharax/body.mob" -> "ferritharax/body"
/// - "mobs/xhitara/grunt.mob" -> "xhitara/grunt"
/// - "mobs/xhitara/spitter.mobpatch" -> "xhitara/spitter"
/// - "ferritharax/body" -> "ferritharax/body" (already normalized)
/// - "mobs/xhitara/spitter" -> "xhitara/spitter"
pub fn normalize_mob_ref(mob_ref: &str) -> String {
    let without_prefix = mob_ref.strip_prefix("mobs/").unwrap_or(mob_ref);
    let without_suffix = without_prefix
        .strip_suffix(".mob")
        .or_else(|| without_prefix.strip_suffix(".mobpatch"))
        .unwrap_or(without_prefix);
    without_suffix.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_mob_ref() {
        // .mob files
        assert_eq!(
            normalize_mob_ref("mobs/ferritharax/body.mob"),
            "ferritharax/body"
        );
        assert_eq!(normalize_mob_ref("mobs/xhitara/grunt.mob"), "xhitara/grunt");
        assert_eq!(normalize_mob_ref("grunt.mob"), "grunt");

        // .mobpatch files
        assert_eq!(
            normalize_mob_ref("mobs/xhitara/spitter.mobpatch"),
            "xhitara/spitter"
        );
        assert_eq!(normalize_mob_ref("grunt.mobpatch"), "grunt");

        // Already normalized
        assert_eq!(normalize_mob_ref("ferritharax/body"), "ferritharax/body");
        assert_eq!(normalize_mob_ref("grunt"), "grunt");
    }
}
