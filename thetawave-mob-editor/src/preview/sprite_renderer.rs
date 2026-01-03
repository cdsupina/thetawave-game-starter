//! Mob sprite rendering for the preview panel.
//!
//! Handles loading and displaying mob sprites, decorations, and
//! jointed mob previews in the central preview area.

use std::path::PathBuf;

use bevy::{
    asset::Handle,
    log::{debug, info, warn},
    prelude::{
        AssetServer, Color, Commands, Component, Entity, Quat, Query, Res, ResMut, Resource, Sprite,
        State, Transform, Vec2, With, default,
    },
};
use bevy_aseprite_ultra::prelude::{Animation, AseAnimation, Aseprite};

use super::{JointedMobCache, PreviewSettings};
use crate::{data::EditorSession, states::EditorState};

/// Marker component for the preview mob entity
#[derive(Component)]
pub struct PreviewMob;

/// Marker component for preview decoration entities
#[derive(Component)]
pub struct PreviewDecoration {
    /// Index of this decoration in the mob's decorations array
    pub index: usize,
}

/// Marker component for jointed mob preview entities
#[derive(Component)]
pub struct PreviewJointedMob;

/// Result of attempting to load a sprite
#[derive(Debug, Clone, Default)]
pub struct SpriteLoadInfo {
    /// The sprite key being loaded
    pub sprite_key: Option<String>,
    /// Path where the sprite was found and loaded from (None if not found)
    pub loaded_from: Option<PathBuf>,
    /// Paths that were searched
    pub searched_paths: Vec<PathBuf>,
    /// Error message if sprite couldn't be loaded
    pub error: Option<String>,
}

/// Resource tracking which mob is currently being previewed
#[derive(Resource, Default)]
pub struct PreviewState {
    /// Path of the currently previewed mob (to detect changes)
    pub current_path: Option<PathBuf>,
    /// Sprite key of the currently loaded sprite
    pub current_sprite_key: Option<String>,
    /// Whether the preview needs to be rebuilt
    pub needs_rebuild: bool,
    /// Information about the current sprite load attempt
    pub sprite_info: SpriteLoadInfo,
}

/// Check if the preview needs to be updated
pub fn check_preview_update(
    mut session: ResMut<EditorSession>,
    mut preview_state: ResMut<PreviewState>,
    state: Res<State<EditorState>>,
) {
    // Only care about preview when editing
    if *state.get() != EditorState::Editing {
        preview_state.needs_rebuild = false;
        session.preview_needs_rebuild = false;
        return;
    }

    // Check if session flagged a rebuild needed (from properties panel changes)
    if session.preview_needs_rebuild {
        preview_state.needs_rebuild = true;
        session.preview_needs_rebuild = false;
    }

    // Check if we switched files
    if preview_state.current_path != session.current_path {
        info!(
            "File changed, triggering preview rebuild. Path: {:?}",
            session.current_path
        );
        preview_state.needs_rebuild = true;
        preview_state.current_path = session.current_path.clone();
        return;
    }

    // Check if sprite changed (use merged data for preview)
    if let Some(mob) = session.mob_for_preview() {
        let sprite_path = get_sprite_path(mob);
        let sprite_key = sprite_path.as_ref().map(|p| sprite_path_to_key(p));

        if sprite_key != preview_state.current_sprite_key {
            debug!(
                "Sprite changed: {:?} -> {:?}",
                preview_state.current_sprite_key, sprite_key
            );
            preview_state.needs_rebuild = true;
        }
    }
}

/// Get the sprite path from mob data
fn get_sprite_path(mob: &toml::Value) -> Option<String> {
    mob.get("sprite").and_then(|v| v.as_str()).map(String::from)
}

/// Extract asset key from sprite path
/// "media/aseprite/xhitara_grunt_mob.aseprite" → "xhitara_grunt_mob"
fn sprite_path_to_key(path: &str) -> String {
    std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(path)
        .to_string()
}

/// Spawn or update the preview mob entity
pub fn update_preview_mob(
    mut commands: Commands,
    session: Res<EditorSession>,
    mut preview_state: ResMut<PreviewState>,
    asset_server: Res<AssetServer>,
    existing_mobs: Query<Entity, With<PreviewMob>>,
    existing_decorations: Query<Entity, With<PreviewDecoration>>,
    existing_jointed: Query<Entity, With<PreviewJointedMob>>,
    state: Res<State<EditorState>>,
    preview_settings: Res<PreviewSettings>,
    jointed_cache: Res<JointedMobCache>,
    config: Res<crate::plugin::EditorConfig>,
) {
    // Despawn preview when not editing
    if *state.get() != EditorState::Editing {
        for entity in &existing_mobs {
            commands.entity(entity).despawn();
        }
        for entity in &existing_decorations {
            commands.entity(entity).despawn();
        }
        for entity in &existing_jointed {
            commands.entity(entity).despawn();
        }
        preview_state.current_sprite_key = None;
        preview_state.sprite_info = SpriteLoadInfo::default();
        return;
    }

    if !preview_state.needs_rebuild {
        return;
    }
    preview_state.needs_rebuild = false;

    // Despawn existing preview entities
    for entity in &existing_mobs {
        commands.entity(entity).despawn();
    }
    for entity in &existing_decorations {
        commands.entity(entity).despawn();
    }
    for entity in &existing_jointed {
        commands.entity(entity).despawn();
    }

    // Use merged data for preview (falls back to current_mob for .mob files)
    let Some(mob) = session.mob_for_preview() else {
        preview_state.sprite_info = SpriteLoadInfo {
            sprite_key: None,
            loaded_from: None,
            searched_paths: vec![],
            error: Some("No mob data loaded".to_string()),
        };
        return;
    };

    // Get sprite path from mob data
    let sprite_path = get_sprite_path(mob);
    // For change detection, use the path with extended:// stripped
    let sprite_key = sprite_path
        .as_ref()
        .map(|p| sprite_path_to_key(strip_extended_prefix(p)));
    preview_state.current_sprite_key = sprite_key.clone();

    // Get z_level
    let z_level = mob.get("z_level").and_then(|v| v.as_float()).unwrap_or(0.0) as f32;

    // Try to load the sprite
    if let Some(ref path) = sprite_path {
        let load_result = try_load_sprite_from_path(path, &asset_server, &config);

        // Update sprite info for UI display
        preview_state.sprite_info = SpriteLoadInfo {
            sprite_key: sprite_path.clone(),
            loaded_from: load_result.loaded_from.clone(),
            searched_paths: load_result.searched_paths,
            error: if load_result.handle.is_none() {
                Some(format!("Sprite '{}' not found", path))
            } else {
                None
            },
        };

        if let Some(aseprite_handle) = load_result.handle {
            debug!("Loading sprite for mob preview: {}", path);

            // Spawn the main mob preview entity
            commands.spawn((
                PreviewMob,
                AseAnimation {
                    animation: Animation::tag("idle"),
                    aseprite: aseprite_handle,
                },
                Sprite::default(),
                Transform::from_xyz(0.0, 0.0, z_level),
            ));

            // Spawn decorations
            spawn_decorations(&mut commands, mob, &asset_server, &config);

            // Spawn jointed mobs if toggle is enabled
            if preview_settings.show_jointed_mobs {
                spawn_jointed_mob_previews(&mut commands, &jointed_cache, &asset_server, &config);
            }
        } else {
            warn!("Could not find sprite: {}", path);
        }
    } else {
        // No sprite field - this is valid, sprites are optional
        preview_state.sprite_info = SpriteLoadInfo {
            sprite_key: None,
            loaded_from: None,
            searched_paths: vec![],
            error: None,
        };
    }
}

/// Spawn jointed mob preview entities with dimmed sprites
fn spawn_jointed_mob_previews(
    commands: &mut Commands,
    cache: &JointedMobCache,
    asset_server: &AssetServer,
    config: &crate::plugin::EditorConfig,
) {
    // Dimmed color (50% alpha)
    let dimmed_color = Color::srgba(1.0, 1.0, 1.0, 0.5);

    for resolved in &cache.resolved_mobs {
        let Some(sprite_path) = &resolved.sprite_path else {
            continue;
        };

        let load_result = try_load_sprite_from_path(sprite_path, asset_server, config);
        if let Some(aseprite_handle) = load_result.handle {
            // Spawn the jointed mob sprite with rotation
            let rotation = Quat::from_rotation_z(resolved.offset_rot.to_radians());
            commands.spawn((
                PreviewJointedMob,
                AseAnimation {
                    animation: Animation::tag("idle"),
                    aseprite: aseprite_handle,
                },
                Sprite {
                    color: dimmed_color,
                    ..default()
                },
                Transform::from_xyz(
                    resolved.offset_pos.x,
                    resolved.offset_pos.y,
                    // Layer behind main mob, with depth affecting z-order
                    resolved.z_level - 0.01 * resolved.depth as f32,
                )
                .with_rotation(rotation),
            ));

            // Spawn decorations for this jointed mob (also dimmed)
            for (dec_path, dec_pos) in &resolved.decorations {
                let dec_result = try_load_sprite_from_path(dec_path, asset_server, config);
                if let Some(dec_handle) = dec_result.handle {
                    commands.spawn((
                        PreviewJointedMob,
                        AseAnimation {
                            animation: Animation::tag("idle"),
                            aseprite: dec_handle,
                        },
                        Sprite {
                            color: dimmed_color,
                            ..default()
                        },
                        Transform::from_xyz(
                            resolved.offset_pos.x + dec_pos.x,
                            resolved.offset_pos.y + dec_pos.y,
                            resolved.z_level + 0.1 - 0.01 * resolved.depth as f32,
                        ),
                    ));
                }
            }
        }
    }
}

/// Result of trying to load a sprite
pub struct SpriteLoadResult {
    pub handle: Option<Handle<Aseprite>>,
    pub loaded_from: Option<PathBuf>,
    pub searched_paths: Vec<PathBuf>,
}

/// Check if a sprite path uses the extended:// prefix
fn is_extended_path(path: &str) -> bool {
    path.starts_with("extended://")
}

/// Strip the extended:// prefix from a path if present
fn strip_extended_prefix(path: &str) -> &str {
    path.strip_prefix("extended://").unwrap_or(path)
}

/// Try to load a sprite from a full path (supports extended:// prefix)
pub fn try_load_sprite_from_path(
    sprite_path: &str,
    asset_server: &AssetServer,
    config: &crate::plugin::EditorConfig,
) -> SpriteLoadResult {
    let cwd = std::env::current_dir().unwrap_or_default();

    // Check for extended:// prefix
    let (relative_path, search_extended_first) = if is_extended_path(sprite_path) {
        (strip_extended_prefix(sprite_path), true)
    } else {
        (sprite_path, false)
    };

    // Build list of filesystem paths to check based on prefix
    let mut search_paths: Vec<PathBuf> = Vec::new();

    if search_extended_first {
        // Extended assets first
        if let Some(extended_root) = config.extended_assets_root() {
            search_paths.push(cwd.join(&extended_root).join(relative_path));
        }
        // Fall back to base assets
        if let Some(base_root) = config.base_assets_root() {
            search_paths.push(cwd.join(&base_root).join(relative_path));
        }
    } else {
        // Base assets directory first
        if let Some(base_root) = config.base_assets_root() {
            search_paths.push(cwd.join(&base_root).join(relative_path));
        }
        // Extended assets
        if let Some(extended_root) = config.extended_assets_root() {
            search_paths.push(cwd.join(&extended_root).join(relative_path));
        }
    };

    // Check each location
    for fs_path in &search_paths {
        if fs_path.exists() {
            let abs_path = fs_path.to_string_lossy().to_string();
            debug!("Found sprite at: {:?}", fs_path);
            return SpriteLoadResult {
                handle: Some(asset_server.load(abs_path)),
                loaded_from: Some(fs_path.clone()),
                searched_paths: search_paths,
            };
        }
    }

    warn!(
        "Sprite '{}' not found. Searched: {:?}",
        sprite_path, search_paths
    );

    SpriteLoadResult {
        handle: None,
        loaded_from: None,
        searched_paths: search_paths,
    }
}

/// Spawn decoration sprites as separate entities
fn spawn_decorations(
    commands: &mut Commands,
    mob: &toml::Value,
    asset_server: &AssetServer,
    config: &crate::plugin::EditorConfig,
) {
    let Some(decorations) = mob.get("decorations").and_then(|v| v.as_array()) else {
        return;
    };

    for (index, decoration) in decorations.iter().enumerate() {
        let Some(arr) = decoration.as_array() else {
            continue;
        };
        if arr.len() < 2 {
            continue;
        }

        // First element is the sprite path (e.g., "media/aseprite/xhitara_grunt_thrusters.aseprite")
        // May also use extended:// prefix for extended assets
        let Some(sprite_path) = arr[0].as_str() else {
            continue;
        };

        // Second element is the position [x, y]
        let position = if let Some(pos_arr) = arr[1].as_array() {
            let x = pos_arr.first().and_then(|v| v.as_float()).unwrap_or(0.0) as f32;
            let y = pos_arr.get(1).and_then(|v| v.as_float()).unwrap_or(0.0) as f32;
            Vec2::new(x, y)
        } else {
            Vec2::ZERO
        };

        // Try to load the decoration sprite using the full path (supports extended:// prefix)
        let load_result = try_load_sprite_from_path(sprite_path, asset_server, config);
        if let Some(handle) = load_result.handle {
            debug!("Loading decoration sprite: {} at {:?}", sprite_path, position);
            commands.spawn((
                PreviewDecoration { index },
                AseAnimation {
                    animation: Animation::tag("idle"),
                    aseprite: handle,
                },
                Sprite::default(),
                Transform::from_xyz(position.x, position.y, 0.1), // Slightly above main sprite
            ));
        }
    }
}

/// Update decoration positions based on current mob data
pub fn update_decoration_positions(
    session: Res<EditorSession>,
    mut decorations: Query<(&PreviewDecoration, &mut Transform)>,
    state: Res<State<EditorState>>,
) {
    // Only update when editing
    if *state.get() != EditorState::Editing {
        return;
    }

    // Get mob data (use merged for patches)
    let Some(mob) = session.mob_for_preview() else {
        return;
    };

    let Some(decorations_array) = mob.get("decorations").and_then(|v| v.as_array()) else {
        return;
    };

    for (decoration, mut transform) in &mut decorations {
        // Get the decoration data at this index
        let Some(decoration_data) = decorations_array.get(decoration.index) else {
            continue;
        };

        let Some(arr) = decoration_data.as_array() else {
            continue;
        };

        if arr.len() < 2 {
            continue;
        }

        // Get position from second element
        if let Some(pos_arr) = arr[1].as_array() {
            let x = pos_arr.first().and_then(|v| v.as_float()).unwrap_or(0.0) as f32;
            let y = pos_arr.get(1).and_then(|v| v.as_float()).unwrap_or(0.0) as f32;

            // Update transform if position changed
            if (transform.translation.x - x).abs() > 0.001
                || (transform.translation.y - y).abs() > 0.001
            {
                transform.translation.x = x;
                transform.translation.y = y;
            }
        }
    }
}
