use std::path::PathBuf;

use bevy::prelude::*;

use crate::data::EditorSession;
use crate::file::{FileNode, FileOperations, FileTreeState};
use crate::states::EditorState;

use super::PreviewState;

/// Maximum recursion depth for jointed mob resolution
const MAX_RECURSION_DEPTH: usize = 10;

/// Cached jointed mob data for preview rendering
#[derive(Resource, Default)]
pub struct JointedMobCache {
    /// Resolved jointed mob hierarchy for the current mob
    pub resolved_mobs: Vec<ResolvedJointedMob>,
    /// List of parent mobs that reference the current mob
    pub parent_mobs: Vec<ParentMobRef>,
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

/// Reference to a parent mob that uses this mob
#[derive(Debug, Clone)]
pub struct ParentMobRef {
    /// Path to the parent mob file
    pub path: PathBuf,
    /// Name of the parent mob
    pub name: String,
    /// The key used in the parent's jointed_mobs array
    pub jointed_key: String,
}

/// Resolve a mob_ref path to an actual file path
fn resolve_mob_ref(mob_ref: &str) -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;

    // Try base assets first
    let base_path = cwd.join("assets").join(mob_ref);
    if base_path.exists() {
        return Some(base_path);
    }

    // Try extended assets
    let extended_path = cwd.join("thetawave-test-game/assets").join(mob_ref);
    if extended_path.exists() {
        return Some(extended_path);
    }

    None
}

/// Parse a Vec2 from a TOML value (array of two floats)
fn parse_vec2(value: Option<&toml::Value>) -> Option<Vec2> {
    let arr = value?.as_array()?;
    if arr.len() < 2 {
        return None;
    }
    let x = arr[0].as_float().or_else(|| arr[0].as_integer().map(|i| i as f64))? as f32;
    let y = arr[1].as_float().or_else(|| arr[1].as_integer().map(|i| i as f64))? as f32;
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
                if let Some(ref_path) = mob_ref.and_then(resolve_mob_ref) {
                    if let Ok(ref_mob) = FileOperations::load_file(&ref_path) {
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
                            );
                        }
                    }
                }
            }
        } else {
            // Non-chain jointed mob
            if let Some(ref_path) = mob_ref.and_then(resolve_mob_ref) {
                if let Ok(ref_mob) = FileOperations::load_file(&ref_path) {
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
                    );
                }
            }
        }
    }
}

/// Extract the relative mob reference path from a full file path
fn extract_relative_mob_path(full_path: &PathBuf) -> Option<String> {
    let path_str = full_path.to_string_lossy();

    // Find "mobs/" in the path
    let mobs_idx = path_str.find("mobs/")?;
    let relative = &path_str[mobs_idx..];

    Some(relative.to_string())
}

/// Recursively scan file nodes for parent references
fn scan_for_parent_refs(node: &FileNode, target_ref: &str, results: &mut Vec<ParentMobRef>) {
    if node.is_directory {
        for child in &node.children {
            scan_for_parent_refs(child, target_ref, results);
        }
    } else if node
        .path
        .extension()
        .map(|e| e == "mob")
        .unwrap_or(false)
    {
        // Load and check this mob file
        if let Ok(mob) = FileOperations::load_file(&node.path) {
            if let Some(jointed_mobs) = mob
                .get("jointed_mobs")
                .and_then(|v: &toml::Value| v.as_array())
            {
                for jointed in jointed_mobs {
                    if let Some(table) = jointed.as_table() {
                        if let Some(mob_ref) = table
                            .get("mob_ref")
                            .and_then(|v: &toml::Value| v.as_str())
                        {
                            if mob_ref == target_ref {
                                let name = mob
                                    .get("name")
                                    .and_then(|v: &toml::Value| v.as_str())
                                    .unwrap_or(&node.name)
                                    .to_string();
                                let key = table
                                    .get("key")
                                    .and_then(|v: &toml::Value| v.as_str())
                                    .unwrap_or("unknown")
                                    .to_string();

                                results.push(ParentMobRef {
                                    path: node.path.clone(),
                                    name,
                                    jointed_key: key,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Scan all mob files to find which ones reference the given mob
fn find_parent_mobs(current_mob_path: &PathBuf, file_tree: &FileTreeState) -> Vec<ParentMobRef> {
    let mut parents = Vec::new();

    // Get the relative path for the current mob
    let Some(current_relative) = extract_relative_mob_path(current_mob_path) else {
        return parents;
    };

    // Scan all mob files in the file tree
    for root in &file_tree.roots {
        scan_for_parent_refs(root, &current_relative, &mut parents);
    }

    parents
}

/// System to rebuild the jointed mob cache when needed
pub fn rebuild_jointed_mob_cache(
    session: Res<EditorSession>,
    preview_state: Res<PreviewState>,
    file_tree: Res<FileTreeState>,
    mut cache: ResMut<JointedMobCache>,
    state: Res<State<EditorState>>,
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
    cache.parent_mobs.clear();
    cache.needs_rebuild = false;
    cache.current_path = session.current_path.clone();

    // Get current mob data
    let Some(mob) = session.mob_for_preview() else {
        return;
    };

    // Resolve jointed mobs hierarchy
    resolve_jointed_mobs(mob, Vec2::ZERO, 0, &mut cache.resolved_mobs);

    // Find parent mobs if we have a current path
    if let Some(path) = &session.current_path {
        cache.parent_mobs = find_parent_mobs(path, &file_tree);
    }

    let max_depth = cache
        .resolved_mobs
        .iter()
        .map(|m| m.depth)
        .max()
        .unwrap_or(0);

    info!(
        "Rebuilt jointed mob cache: {} resolved mobs, {} parent refs, max depth {}",
        cache.resolved_mobs.len(),
        cache.parent_mobs.len(),
        max_depth
    );
}
