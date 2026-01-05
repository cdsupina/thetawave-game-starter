//! MobRegistry resource for resolving mob references to loaded assets.

use std::fmt;

use bevy::{
    asset::{Asset, Assets, Handle},
    log::{debug, info, warn},
    platform::collections::HashMap,
    prelude::Resource,
    reflect::Reflect,
};
use bevy_behave::{Behave, prelude::Tree};
use serde::{Deserialize, Serialize};
use thetawave_core::merge_toml_values;

use super::{
    ExtendedMobPatches, ExtendedMobs, MobAsset, MobAssets, MobPatch, ModMobPatches, ModMobs,
    RawMob,
};
use crate::behavior::build_behavior_tree;

/// A strongly-typed reference to a mob definition.
///
/// This newtype wrapper prevents accidentally passing other string types
/// (like sprite keys or entity names) where a mob reference is expected.
///
/// MobRef values are automatically normalized during creation:
/// - "mobs/xhitara/grunt.mob" → "xhitara/grunt"
/// - "xhitara/grunt" → "xhitara/grunt" (already normalized)
///
/// # Example
/// ```ignore
/// let mob_ref = MobRef::new("mobs/xhitara/grunt.mob");
/// assert_eq!(mob_ref.as_str(), "xhitara/grunt");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize)]
pub struct MobRef(String);

impl MobRef {
    /// Create a new MobRef, normalizing the path automatically.
    pub fn new(path: impl Into<String>) -> Self {
        Self(normalize_mob_ref(&path.into()))
    }

    /// Get the normalized mob reference as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume the MobRef and return the inner String.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Display for MobRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for MobRef {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<&str> for MobRef {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for MobRef {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&String> for MobRef {
    fn from(s: &String) -> Self {
        Self::new(s.as_str())
    }
}

impl<'de> Deserialize<'de> for MobRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(MobRef::new(s))
    }
}

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
    /// Processing order (3-tier: base → game → mods):
    /// 1. Collect raw TOML values from base mobs
    /// 2. Add game mobs (can add new or override base)
    /// 3. Add mod mobs (can add new or override base/game)
    /// 4. Merge game patches into values
    /// 5. Merge mod patches into values (can override game patches)
    /// 6. Deserialize merged values to MobAsset
    /// 7. Build behavior trees for all mobs
    pub fn build(
        base_assets: &MobAssets,
        game_mobs: &ExtendedMobs,
        mod_mobs: &ModMobs,
        game_patches: &ExtendedMobPatches,
        mod_patches: &ModMobPatches,
        raw_mob_assets: &Assets<RawMob>,
        patch_assets: &Assets<MobPatch>,
    ) -> Self {
        info!(
            "Building MobRegistry: {} base, game_mobs={}, mod_mobs={}, game_patches={}, mod_patches={}",
            base_assets.mobs.len(),
            if game_mobs.mobs.is_some() { "Some" } else { "None" },
            if mod_mobs.mobs.is_some() { "Some" } else { "None" },
            if game_patches.patches.is_some() { "Some" } else { "None" },
            if mod_patches.patches.is_some() { "Some" } else { "None" }
        );

        let mut raw_values = Self::collect_base_mobs(base_assets, raw_mob_assets);
        Self::add_game_mobs(&mut raw_values, game_mobs, raw_mob_assets);
        Self::add_mod_mobs(&mut raw_values, mod_mobs, raw_mob_assets);
        Self::apply_game_patches(&mut raw_values, game_patches, patch_assets);
        Self::apply_mod_patches(&mut raw_values, mod_patches, patch_assets);
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

    /// Add game mobs (complete new mobs) to the raw values map.
    fn add_game_mobs(
        raw_values: &mut HashMap<String, toml::Value>,
        game_mobs: &ExtendedMobs,
        raw_mob_assets: &Assets<RawMob>,
    ) {
        let Some(mobs) = &game_mobs.mobs else {
            info!("No game mobs to process");
            return;
        };

        info!("Processing {} game mobs", mobs.len());

        for (_stem, handle) in mobs {
            let Some(key) = get_normalized_key(handle) else {
                warn!("Game mob handle has no path");
                continue;
            };

            if let Some(raw_mob) = raw_mob_assets.get(handle) {
                if raw_values.contains_key(&key) {
                    info!("Game mob '{}' overrides base mob", key);
                } else {
                    debug!("Adding game mob: {}", key);
                }
                raw_values.insert(key, raw_mob.value.clone());
            } else {
                warn!("Could not get game mob asset for: {}", key);
            }
        }
    }

    /// Add mod mobs (complete new mobs) to the raw values map.
    fn add_mod_mobs(
        raw_values: &mut HashMap<String, toml::Value>,
        mod_mobs: &ModMobs,
        raw_mob_assets: &Assets<RawMob>,
    ) {
        let Some(mobs) = &mod_mobs.mobs else {
            info!("No mod mobs to process");
            return;
        };

        info!("Processing {} mod mobs", mobs.len());

        for (_stem, handle) in mobs {
            let Some(key) = get_normalized_key(handle) else {
                warn!("Mod mob handle has no path");
                continue;
            };

            if let Some(raw_mob) = raw_mob_assets.get(handle) {
                if raw_values.contains_key(&key) {
                    info!("Mod mob '{}' overrides base/game mob", key);
                } else {
                    debug!("Adding mod mob: {}", key);
                }
                raw_values.insert(key, raw_mob.value.clone());
            } else {
                warn!("Could not get mod mob asset for: {}", key);
            }
        }
    }

    /// Apply game patches to base/game mob values.
    fn apply_game_patches(
        raw_values: &mut HashMap<String, toml::Value>,
        game_patches: &ExtendedMobPatches,
        patch_assets: &Assets<MobPatch>,
    ) {
        let Some(patches) = &game_patches.patches else {
            info!("No game mob patches to process");
            return;
        };

        info!("Processing {} game mob patches", patches.len());
        let mut merged_count = 0;

        for (_stem, handle) in patches {
            let Some(key) = get_normalized_key(handle) else {
                warn!("Game patch handle has no path");
                continue;
            };

            let Some(patch) = patch_assets.get(handle) else {
                warn!("Could not get game patch asset for: {}", key);
                continue;
            };

            if let Some(base_value) = raw_values.get_mut(&key) {
                info!("Merging game patch '{}' into mob", key);
                merge_toml_values(base_value, patch.value.clone());
                merged_count += 1;
            } else {
                warn!(
                    "No mob found for game patch '{}', skipping (use .mob for new mobs)",
                    key
                );
            }
        }

        if merged_count > 0 {
            info!("Merged {} game patches into mobs", merged_count);
        }
    }

    /// Apply mod patches to base/game/mod mob values.
    fn apply_mod_patches(
        raw_values: &mut HashMap<String, toml::Value>,
        mod_patches: &ModMobPatches,
        patch_assets: &Assets<MobPatch>,
    ) {
        let Some(patches) = &mod_patches.patches else {
            info!("No mod mob patches to process");
            return;
        };

        info!("Processing {} mod mob patches", patches.len());
        let mut merged_count = 0;

        for (_stem, handle) in patches {
            let Some(key) = get_normalized_key(handle) else {
                warn!("Mod patch handle has no path");
                continue;
            };

            let Some(patch) = patch_assets.get(handle) else {
                warn!("Could not get mod patch asset for: {}", key);
                continue;
            };

            if let Some(base_value) = raw_values.get_mut(&key) {
                info!("Merging mod patch '{}' into mob", key);
                merge_toml_values(base_value, patch.value.clone());
                merged_count += 1;
            } else {
                warn!(
                    "No mob found for mod patch '{}', skipping (use .mob for new mobs)",
                    key
                );
            }
        }

        if merged_count > 0 {
            info!("Merged {} mod patches into mobs", merged_count);
        }
    }

    /// Deserialize all raw TOML values to MobAsset structs.
    ///
    /// Collects all deserialization errors and reports them as a batch summary
    /// to avoid log spam when multiple mobs fail.
    fn deserialize_all(raw_values: HashMap<String, toml::Value>) -> HashMap<String, MobAsset> {
        let mut mobs = HashMap::new();
        let mut errors: Vec<(String, String)> = Vec::new();

        for (key, value) in raw_values {
            match value.try_into::<MobAsset>() {
                Ok(mob) => {
                    mobs.insert(key, mob);
                }
                Err(e) => {
                    errors.push((key, e.to_string()));
                }
            }
        }

        // Report errors as a batch summary
        if !errors.is_empty() {
            warn!(
                "Failed to deserialize {} mob(s):\n{}",
                errors.len(),
                errors
                    .iter()
                    .map(|(key, err)| format!("  - {}: {}", key, err))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
        }

        info!("Deserialized {} mobs ({} failed)", mobs.len(), errors.len());
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

    #[test]
    fn test_mob_ref_normalization() {
        // Full paths are normalized
        let ref1 = MobRef::new("mobs/xhitara/grunt.mob");
        assert_eq!(ref1.as_str(), "xhitara/grunt");

        // Already normalized paths stay the same
        let ref2 = MobRef::new("xhitara/grunt");
        assert_eq!(ref2.as_str(), "xhitara/grunt");

        // .mobpatch extension is also stripped
        let ref3 = MobRef::new("mobs/xhitara/spitter.mobpatch");
        assert_eq!(ref3.as_str(), "xhitara/spitter");
    }

    #[test]
    fn test_mob_ref_equality() {
        // Same normalized value should be equal regardless of input format
        let ref1 = MobRef::new("mobs/xhitara/grunt.mob");
        let ref2 = MobRef::new("xhitara/grunt");
        assert_eq!(ref1, ref2);
    }

    #[test]
    fn test_mob_ref_from_string() {
        let s = String::from("mobs/xhitara/grunt.mob");
        let mob_ref: MobRef = s.into();
        assert_eq!(mob_ref.as_str(), "xhitara/grunt");
    }

    #[test]
    fn test_mob_ref_from_str() {
        let mob_ref: MobRef = "mobs/xhitara/grunt.mob".into();
        assert_eq!(mob_ref.as_str(), "xhitara/grunt");
    }

    #[test]
    fn test_mob_ref_display() {
        let mob_ref = MobRef::new("xhitara/grunt");
        assert_eq!(format!("{}", mob_ref), "xhitara/grunt");
    }

    // ========================================================================
    // Integration tests for MobRegistry build pipeline
    // ========================================================================

    #[test]
    fn test_mob_asset_deserialization_minimal() {
        // Minimal valid mob definition
        let toml_str = r#"
            name = "Test Mob"
            sprite = "media/aseprite/test_mob.aseprite"
        "#;

        let value: toml::Value = toml::from_str(toml_str).unwrap();
        let mob: MobAsset = value.try_into().expect("Should deserialize minimal mob");

        assert_eq!(mob.name, "Test Mob");
        assert_eq!(mob.sprite, "media/aseprite/test_mob.aseprite");
        assert!(mob.spawnable); // default
        assert_eq!(mob.health, 50); // default
    }

    #[test]
    fn test_mob_asset_deserialization_with_colliders() {
        let toml_str = r#"
            name = "Collider Mob"
            sprite = "media/aseprite/collider_mob.aseprite"
            health = 100
            colliders = [
                { shape = { Rectangle = [12.0, 15.0] }, position = [0.0, 0.0], rotation = 0.0 }
            ]
        "#;

        let value: toml::Value = toml::from_str(toml_str).unwrap();
        let mob: MobAsset = value
            .try_into()
            .expect("Should deserialize mob with colliders");

        assert_eq!(mob.name, "Collider Mob");
        assert_eq!(mob.health, 100);
        assert_eq!(mob.colliders.len(), 1);
    }

    #[test]
    fn test_mob_asset_deserialization_with_behavior() {
        let toml_str = r#"
            name = "Behaving Mob"
            sprite = "media/aseprite/behaving_mob.aseprite"

            [behavior]
            type = "Forever"
            [[behavior.children]]
            type = "Action"
            name = "Movement"
            behaviors = [{ action = "MoveDown" }]
        "#;

        let value: toml::Value = toml::from_str(toml_str).unwrap();
        let mob: MobAsset = value
            .try_into()
            .expect("Should deserialize mob with behavior");

        assert_eq!(mob.name, "Behaving Mob");
        assert!(mob.behavior.is_some());
    }

    #[test]
    fn test_mob_asset_deserialization_with_jointed_mobs() {
        let toml_str = r#"
            name = "Parent Mob"
            sprite = "media/aseprite/parent_mob.aseprite"

            [[jointed_mobs]]
            key = "left_arm"
            mob_ref = "mobs/parts/arm.mob"
            offset_pos = [10.0, 0.0]
        "#;

        let value: toml::Value = toml::from_str(toml_str).unwrap();
        let mob: MobAsset = value
            .try_into()
            .expect("Should deserialize mob with joints");

        assert_eq!(mob.name, "Parent Mob");
        assert_eq!(mob.jointed_mobs.len(), 1);
        assert_eq!(mob.jointed_mobs[0].key, "left_arm");
        // MobRef is normalized
        assert_eq!(mob.jointed_mobs[0].mob_ref.as_str(), "parts/arm");
    }

    #[test]
    fn test_mob_asset_deserialization_rejects_unknown_fields() {
        let toml_str = r#"
            name = "Test Mob"
            sprite = "media/aseprite/test_mob.aseprite"
            unknown_field = "should fail"
        "#;

        let value: toml::Value = toml::from_str(toml_str).unwrap();
        let result: Result<MobAsset, _> = value.try_into();

        assert!(result.is_err(), "Should reject unknown fields");
    }

    #[test]
    fn test_toml_merge_basic_fields() {
        let base_toml = r#"
            name = "Base Mob"
            sprite = "media/aseprite/base_mob.aseprite"
            health = 100
            projectile_speed = 50.0
        "#;

        let patch_toml = r#"
            name = "Patched Mob"
            projectile_speed = 200.0
        "#;

        let mut base: toml::Value = toml::from_str(base_toml).unwrap();
        let patch: toml::Value = toml::from_str(patch_toml).unwrap();

        merge_toml_values(&mut base, patch);

        let mob: MobAsset = base.try_into().expect("Should deserialize merged mob");

        assert_eq!(mob.name, "Patched Mob"); // overridden
        assert_eq!(mob.health, 100); // unchanged from base
        assert_eq!(mob.projectile_speed, 200.0); // overridden
    }

    #[test]
    fn test_toml_merge_nested_tables() {
        let base_toml = r#"
            name = "Base Mob"
            sprite = "media/aseprite/base_mob.aseprite"

            [projectile_spawners]
            [projectile_spawners.spawners.north]
            timer = 1.0
            position = [0.0, 5.0]
            [projectile_spawners.spawners.south]
            timer = 2.0
            position = [0.0, -5.0]
        "#;

        let patch_toml = r#"
            [projectile_spawners.spawners.south]
            timer = 0.5
        "#;

        let mut base: toml::Value = toml::from_str(base_toml).unwrap();
        let patch: toml::Value = toml::from_str(patch_toml).unwrap();

        merge_toml_values(&mut base, patch);

        // Verify the merge: south timer changed, north unchanged
        let spawners = base
            .get("projectile_spawners")
            .and_then(|ps| ps.get("spawners"))
            .expect("spawners should exist");

        let north_timer = spawners
            .get("north")
            .and_then(|n| n.get("timer"))
            .and_then(|t| t.as_float())
            .expect("north timer should exist");
        assert_eq!(north_timer, 1.0);

        let south_timer = spawners
            .get("south")
            .and_then(|s| s.get("timer"))
            .and_then(|t| t.as_float())
            .expect("south timer should exist");
        assert_eq!(south_timer, 0.5);
    }

    #[test]
    fn test_deserialize_all_handles_errors_gracefully() {
        let mut raw_values = HashMap::new();

        // Valid mob
        let valid_toml = r#"
            name = "Valid Mob"
            sprite = "media/aseprite/valid_mob.aseprite"
        "#;
        raw_values.insert("valid/mob".to_string(), toml::from_str(valid_toml).unwrap());

        // Invalid mob (unknown field)
        let invalid_toml = r#"
            name = "Invalid Mob"
            unknown_field = true
        "#;
        raw_values.insert(
            "invalid/mob".to_string(),
            toml::from_str(invalid_toml).unwrap(),
        );

        // Another valid mob
        let valid_toml2 = r#"
            name = "Another Valid Mob"
            sprite = "media/aseprite/another_valid_mob.aseprite"
        "#;
        raw_values.insert(
            "another/valid".to_string(),
            toml::from_str(valid_toml2).unwrap(),
        );

        let mobs = MobRegistry::deserialize_all(raw_values);

        // Should have 2 valid mobs, 1 failed
        assert_eq!(mobs.len(), 2);
        assert!(mobs.contains_key("valid/mob"));
        assert!(mobs.contains_key("another/valid"));
        assert!(!mobs.contains_key("invalid/mob"));
    }

    #[test]
    fn test_mob_ref_deserialization_in_jointed_mob() {
        // Test that MobRef deserializes and normalizes correctly in JointedMobRef
        let toml_str = r#"
            key = "arm"
            mob_ref = "mobs/parts/arm.mob"
        "#;

        let value: toml::Value = toml::from_str(toml_str).unwrap();
        let jointed: super::super::JointedMobRef =
            value.try_into().expect("Should deserialize JointedMobRef");

        assert_eq!(jointed.key, "arm");
        assert_eq!(jointed.mob_ref.as_str(), "parts/arm");
    }
}
