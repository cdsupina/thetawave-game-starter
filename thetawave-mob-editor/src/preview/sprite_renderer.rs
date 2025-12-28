use std::path::PathBuf;

use bevy::prelude::*;
use bevy_aseprite_ultra::prelude::{Animation, AseAnimation, Aseprite};

use crate::{data::EditorSession, plugin::EditorConfig, states::EditorState};

/// Marker component for the preview mob entity
#[derive(Component)]
pub struct PreviewMob;

/// Marker component for preview decoration entities
#[derive(Component)]
pub struct PreviewDecoration;

/// Resource tracking which mob is currently being previewed
#[derive(Resource, Default)]
pub struct PreviewState {
    /// Path of the currently previewed mob (to detect changes)
    pub current_path: Option<PathBuf>,
    /// Sprite key of the currently loaded sprite
    pub current_sprite_key: Option<String>,
    /// Whether the preview needs to be rebuilt
    pub needs_rebuild: bool,
}

/// Check if the preview needs to be updated
pub fn check_preview_update(
    session: Res<EditorSession>,
    mut preview_state: ResMut<PreviewState>,
    state: Res<State<EditorState>>,
) {
    // Only care about preview when editing
    if *state.get() != EditorState::Editing {
        preview_state.needs_rebuild = false;
        return;
    }

    // Check if we switched files
    if preview_state.current_path != session.current_path {
        info!("File changed, triggering preview rebuild. Path: {:?}", session.current_path);
        preview_state.needs_rebuild = true;
        preview_state.current_path = session.current_path.clone();
        return;
    }

    // Check if sprite_key changed (use merged data for preview)
    if let Some(mob) = session.mob_for_preview() {
        let sprite_key = get_sprite_key(mob, session.current_path.as_ref());

        if sprite_key != preview_state.current_sprite_key {
            info!("Sprite key changed: {:?} -> {:?}", preview_state.current_sprite_key, sprite_key);
            preview_state.needs_rebuild = true;
        }
    }
}

/// Get the sprite key from mob data, deriving it from the path if not specified
fn get_sprite_key(mob: &toml::Value, path: Option<&PathBuf>) -> Option<String> {
    // First check if sprite_key is explicitly set
    if let Some(key) = mob.get("sprite_key").and_then(|v| v.as_str()) {
        return Some(key.to_string());
    }

    // Otherwise derive from the file path
    // e.g., "assets/mobs/xhitara/grunt.mob" -> "xhitara_grunt_mob"
    if let Some(path) = path {
        // Try to extract the mob reference from the path
        let path_str = path.to_string_lossy();

        // Find "mobs/" in the path and extract everything after it
        if let Some(mobs_idx) = path_str.find("mobs/") {
            let after_mobs = &path_str[mobs_idx + 5..]; // Skip "mobs/"

            // Remove the extension
            let without_ext = after_mobs
                .strip_suffix(".mob")
                .or_else(|| after_mobs.strip_suffix(".mobpatch"))
                .unwrap_or(after_mobs);

            // Replace / with _ and append _mob
            let derived = format!("{}_mob", without_ext.replace('/', "_"));
            return Some(derived);
        }
    }

    None
}

/// Spawn or update the preview mob entity
pub fn update_preview_mob(
    mut commands: Commands,
    session: Res<EditorSession>,
    mut preview_state: ResMut<PreviewState>,
    config: Res<EditorConfig>,
    asset_server: Res<AssetServer>,
    existing_mobs: Query<Entity, With<PreviewMob>>,
    existing_decorations: Query<Entity, With<PreviewDecoration>>,
    state: Res<State<EditorState>>,
) {
    // Despawn preview when not editing
    if *state.get() != EditorState::Editing {
        for entity in &existing_mobs {
            commands.entity(entity).despawn();
        }
        for entity in &existing_decorations {
            commands.entity(entity).despawn();
        }
        preview_state.current_sprite_key = None;
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

    // Use merged data for preview (falls back to current_mob for .mob files)
    let Some(mob) = session.mob_for_preview() else {
        return;
    };

    // Get sprite key (explicit or derived)
    let sprite_key = get_sprite_key(mob, session.current_path.as_ref());
    preview_state.current_sprite_key = sprite_key.clone();

    // Get z_level
    let z_level = mob
        .get("z_level")
        .and_then(|v| v.as_float())
        .unwrap_or(0.0) as f32;

    // Try to load the sprite
    if let Some(ref key) = sprite_key {
        if let Some(aseprite_handle) = try_load_sprite(key, &config, &asset_server) {
            info!("Loading sprite for mob preview: {}", key);

            // Spawn the main mob preview entity
            let mob_entity = commands
                .spawn((
                    PreviewMob,
                    AseAnimation {
                        animation: Animation::tag("idle"),
                        aseprite: aseprite_handle,
                    },
                    Sprite::default(),
                    Transform::from_xyz(0.0, 0.0, z_level),
                ))
                .id();

            // Spawn decorations
            spawn_decorations(
                &mut commands,
                mob,
                mob_entity,
                &config,
                &asset_server,
            );
        } else {
            warn!("Could not find sprite: {}", key);
        }
    } else {
        warn!("No sprite_key found for mob");
    }
}

/// Try to load a sprite by key from various asset directories
fn try_load_sprite(
    sprite_key: &str,
    _config: &EditorConfig,
    asset_server: &AssetServer,
) -> Option<Handle<Aseprite>> {
    let filename = format!("{}.aseprite", sprite_key);
    let relative_path = format!("media/aseprite/{}", filename);

    // Get current working directory for building absolute paths
    let cwd = std::env::current_dir().unwrap_or_default();

    // Build list of filesystem paths to check
    let search_paths: Vec<PathBuf> = vec![
        // Base assets directory
        cwd.join("assets").join(&relative_path),
        // Extended assets (thetawave-test-game)
        cwd.join("thetawave-test-game/assets").join(&relative_path),
    ];

    // Check each location
    for fs_path in &search_paths {
        if fs_path.exists() {
            // Use absolute path for loading to handle both base and extended assets
            let abs_path = fs_path.to_string_lossy().to_string();
            info!("Found sprite at: {:?}, loading with absolute path", fs_path);
            return Some(asset_server.load(abs_path));
        }
    }

    // Log what we tried
    warn!(
        "Sprite '{}' not found. Searched: {:?}",
        sprite_key,
        search_paths
    );

    None
}

/// Spawn decoration sprites as separate entities
fn spawn_decorations(
    commands: &mut Commands,
    mob: &toml::Value,
    _parent: Entity,
    config: &EditorConfig,
    asset_server: &AssetServer,
) {
    let Some(decorations) = mob.get("decorations").and_then(|v| v.as_array()) else {
        return;
    };

    for decoration in decorations {
        let Some(arr) = decoration.as_array() else {
            continue;
        };
        if arr.len() < 2 {
            continue;
        }

        // First element is the sprite key
        let Some(sprite_key) = arr[0].as_str() else {
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

        // Try to load the decoration sprite
        if let Some(handle) = try_load_sprite(sprite_key, config, asset_server) {
            info!("Loading decoration sprite: {} at {:?}", sprite_key, position);
            commands.spawn((
                PreviewDecoration,
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
