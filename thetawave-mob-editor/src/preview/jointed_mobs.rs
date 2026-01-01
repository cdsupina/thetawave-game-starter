use std::path::PathBuf;

use bevy::{
    log::{info, warn},
    prelude::{Res, ResMut, Resource, State, Vec2},
};

use crate::data::EditorSession;
use crate::file::FileOperations;
use crate::states::EditorState;

use super::PreviewState;

/// Maximum recursion depth for jointed mob resolution
const MAX_RECURSION_DEPTH: usize = 10;

/// Cached jointed mob data for preview rendering
#[derive(Resource, Default)]
pub struct JointedMobCache {
    /// Resolved jointed mob hierarchy for the current mob
    pub resolved_mobs: Vec<ResolvedJointedMob>,
    /// Whether the cache needs to be rebuilt
    pub needs_rebuild: bool,
    /// Path of the mob this cache was built for
    current_path: Option<PathBuf>,
}

/// A resolved jointed mob ready for rendering
#[derive(Debug, Clone)]
pub struct ResolvedJointedMob {
    /// Path to the sprite (may include extended:// prefix)
    pub sprite_path: Option<String>,
    /// Cumulative offset from the main mob
    pub offset_pos: Vec2,
    /// Z-level for rendering order
    pub z_level: f32,
    /// Nesting depth (0 = direct child of main mob)
    pub depth: usize,
    /// Decorations for this jointed mob: (sprite_path, offset)
    pub decorations: Vec<(String, Vec2)>,
}

/// Resolve a mob_ref path to an actual file path
fn resolve_mob_ref(mob_ref: &str, config: &crate::plugin::EditorConfig) -> Option<PathBuf> {
    config.resolve_mob_ref(mob_ref)
}

/// Parse a Vec2 from a TOML value (array of two floats)
fn parse_vec2(value: Option<&toml::Value>) -> Option<Vec2> {
    let arr = value?.as_array()?;
    if arr.len() < 2 {
        return None;
    }
    let x = arr[0]
        .as_float()
        .or_else(|| arr[0].as_integer().map(|i| i as f64))? as f32;
    let y = arr[1]
        .as_float()
        .or_else(|| arr[1].as_integer().map(|i| i as f64))? as f32;
    Some(Vec2::new(x, y))
}

/// Extract decorations from a mob TOML value
fn extract_decorations(mob: &toml::Value) -> Vec<(String, Vec2)> {
    let mut result = Vec::new();

    let Some(decorations) = mob.get("decorations").and_then(|v| v.as_array()) else {
        return result;
    };

    for decoration in decorations {
        let Some(arr) = decoration.as_array() else {
            continue;
        };
        if arr.len() < 2 {
            continue;
        }

        let Some(sprite_path) = arr[0].as_str() else {
            continue;
        };

        let position = if let Some(pos_arr) = arr[1].as_array() {
            let x = pos_arr
                .first()
                .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
                .unwrap_or(0.0) as f32;
            let y = pos_arr
                .get(1)
                .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
                .unwrap_or(0.0) as f32;
            Vec2::new(x, y)
        } else {
            Vec2::ZERO
        };

        result.push((sprite_path.to_string(), position));
    }

    result
}

/// Recursively resolve all jointed mobs for a given mob
fn resolve_jointed_mobs(
    mob: &toml::Value,
    parent_offset: Vec2,
    depth: usize,
    results: &mut Vec<ResolvedJointedMob>,
    config: &crate::plugin::EditorConfig,
) {
    if depth >= MAX_RECURSION_DEPTH {
        warn!(
            "Max recursion depth {} reached while resolving jointed mobs",
            MAX_RECURSION_DEPTH
        );
        return;
    }

    let Some(jointed_mobs) = mob.get("jointed_mobs").and_then(|v| v.as_array()) else {
        return;
    };

    for jointed in jointed_mobs {
        let Some(table) = jointed.as_table() else {
            continue;
        };

        let mob_ref = table.get("mob_ref").and_then(|v| v.as_str());
        let offset_pos = parse_vec2(table.get("offset_pos")).unwrap_or(Vec2::ZERO);

        // Check for chain configuration
        if let Some(chain) = table.get("chain").and_then(|v| v.as_table()) {
            let length = chain
                .get("length")
                .and_then(|v| v.as_integer())
                .unwrap_or(1) as usize;
            let pos_offset = parse_vec2(chain.get("pos_offset")).unwrap_or(Vec2::ZERO);

            // Spawn chain segments
            for i in 0..length {
                let chain_offset = offset_pos + pos_offset * i as f32;

                // Load the referenced mob
                if let Some(ref_path) = mob_ref.and_then(|r| resolve_mob_ref(r, config))
                    && let Ok(ref_mob) = FileOperations::load_file(&ref_path)
                {
                    let sprite = ref_mob
                        .get("sprite")
                        .and_then(|v: &toml::Value| v.as_str())
                        .map(String::from);
                    let z_level = ref_mob
                        .get("z_level")
                        .and_then(|v: &toml::Value| v.as_float())
                        .unwrap_or(0.0) as f32;
                    let decorations = extract_decorations(&ref_mob);

                    results.push(ResolvedJointedMob {
                        sprite_path: sprite,
                        offset_pos: parent_offset + chain_offset,
                        z_level,
                        depth,
                        decorations,
                    });

                    // Recursively resolve this segment's jointed mobs (only for last segment)
                    if i == length - 1 {
                        resolve_jointed_mobs(
                            &ref_mob,
                            parent_offset + chain_offset,
                            depth + 1,
                            results,
                            config,
                        );
                    }
                }
            }
        } else {
            // Non-chain jointed mob
            if let Some(ref_path) = mob_ref.and_then(|r| resolve_mob_ref(r, config))
                && let Ok(ref_mob) = FileOperations::load_file(&ref_path)
            {
                let sprite = ref_mob
                    .get("sprite")
                    .and_then(|v: &toml::Value| v.as_str())
                    .map(String::from);
                let z_level = ref_mob
                    .get("z_level")
                    .and_then(|v: &toml::Value| v.as_float())
                    .unwrap_or(0.0) as f32;
                let decorations = extract_decorations(&ref_mob);

                results.push(ResolvedJointedMob {
                    sprite_path: sprite,
                    offset_pos: parent_offset + offset_pos,
                    z_level,
                    depth,
                    decorations,
                });

                // Recursively resolve nested jointed mobs
                resolve_jointed_mobs(
                    &ref_mob,
                    parent_offset + offset_pos,
                    depth + 1,
                    results,
                    config,
                );
            }
        }
    }
}

/// System to rebuild the jointed mob cache when needed
pub fn rebuild_jointed_mob_cache(
    session: Res<EditorSession>,
    preview_state: Res<PreviewState>,
    mut cache: ResMut<JointedMobCache>,
    state: Res<State<EditorState>>,
    config: Res<crate::plugin::EditorConfig>,
) {
    // Only rebuild when editing
    if *state.get() != EditorState::Editing {
        return;
    }

    // Check if we need to rebuild
    let path_changed = cache.current_path != session.current_path;
    let needs_rebuild = preview_state.needs_rebuild || cache.needs_rebuild || path_changed;

    if !needs_rebuild {
        return;
    }

    // Clear and rebuild
    cache.resolved_mobs.clear();
    cache.needs_rebuild = false;
    cache.current_path = session.current_path.clone();

    // Get current mob data
    let Some(mob) = session.mob_for_preview() else {
        return;
    };

    // Resolve jointed mobs hierarchy
    resolve_jointed_mobs(mob, Vec2::ZERO, 0, &mut cache.resolved_mobs, &config);

    let max_depth = cache
        .resolved_mobs
        .iter()
        .map(|m| m.depth)
        .max()
        .unwrap_or(0);

    info!(
        "Rebuilt jointed mob cache: {} resolved mobs, max depth {}",
        cache.resolved_mobs.len(),
        max_depth
    );
}
