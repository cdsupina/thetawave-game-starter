use std::path::PathBuf;

use bevy::{ecs::message::{MessageReader, MessageWriter}, prelude::*};
use bevy_aseprite_ultra::AsepriteUltraPlugin;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};

use crate::{
    data::{EditorSession, RegisteredSprite, SpriteRegistry, SpriteSource},
    file::{
        parse_assets_ron, DeleteMobEvent, FileOperations, FileTreeState, LoadMobEvent,
        NewMobEvent, ReloadMobEvent, SaveMobEvent,
    },
    preview::{
        check_preview_update, draw_collider_gizmos, draw_grid, draw_spawner_gizmos,
        handle_camera_input, setup_preview_camera, update_preview_camera, update_preview_mob,
        update_preview_settings, PreviewSettings, PreviewState,
    },
    states::{DialogState, EditingMode, EditorState},
    ui::{main_ui_system, DeleteDialogState, FileDialogState},
};

/// Main plugin for the mob editor
pub struct MobEditorPlugin {
    /// Base assets directory (where the main game's mobs are)
    pub base_assets_dir: PathBuf,
    /// Extended assets directory (for game-specific overrides)
    pub extended_assets_dir: Option<PathBuf>,
}

impl Default for MobEditorPlugin {
    fn default() -> Self {
        Self {
            base_assets_dir: PathBuf::from("assets/mobs"),
            extended_assets_dir: Some(PathBuf::from("thetawave-test-game/assets/mobs")),
        }
    }
}

impl Plugin for MobEditorPlugin {
    fn build(&self, app: &mut App) {
        // Add EguiPlugin if not already added
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin::default());
        }

        // Add Aseprite plugin for sprite loading
        if !app.is_plugin_added::<AsepriteUltraPlugin>() {
            app.add_plugins(AsepriteUltraPlugin);
        }

        // States
        app.init_state::<EditorState>()
            .init_state::<EditingMode>()
            .init_state::<DialogState>();

        // Resources
        app.init_resource::<EditorSession>()
            .init_resource::<FileTreeState>()
            .init_resource::<PreviewSettings>()
            .init_resource::<PreviewState>()
            .init_resource::<FileDialogState>()
            .init_resource::<DeleteDialogState>()
            .init_resource::<SpriteRegistry>()
            .init_resource::<SpriteRegistrationDialog>()
            .init_resource::<SpriteSelectionDialog>();

        // Store config
        app.insert_resource(EditorConfig {
            base_assets_dir: self.base_assets_dir.clone(),
            extended_assets_dir: self.extended_assets_dir.clone(),
        });

        // Messages
        app.add_message::<LoadMobEvent>()
            .add_message::<SaveMobEvent>()
            .add_message::<ReloadMobEvent>()
            .add_message::<NewMobEvent>()
            .add_message::<DeleteMobEvent>();

        // Startup systems
        app.add_systems(
            Startup,
            (setup_preview_camera, initial_scan_system, initial_sprite_scan),
        );

        // UI system runs in EguiPrimaryContextPass schedule (required for egui input handling)
        // Note: run_if removed due to system parameter limit; state check done inside main_ui_system
        app.add_systems(EguiPrimaryContextPass, main_ui_system);

        // Preview update systems (order matters)
        app.add_systems(
            Update,
            (
                update_preview_settings,
                handle_camera_input,
            ),
        );

        app.add_systems(
            Update,
            (
                check_preview_update,
                update_preview_mob.after(check_preview_update),
                update_preview_camera.after(update_preview_settings),
            ),
        );

        // Gizmo systems (run after preview update)
        app.add_systems(
            Update,
            (draw_grid, draw_collider_gizmos, draw_spawner_gizmos),
        );

        // Keyboard shortcuts and file operations run in Update
        app.add_systems(
            Update,
            (
                handle_keyboard_shortcuts,
                handle_load_mob,
                handle_save_mob,
                handle_reload_mob,
                handle_new_mob,
                handle_delete_mob,
                check_file_refresh,
                check_sprite_registry_refresh,
            ),
        );
    }
}

/// Editor configuration resource
#[derive(Resource)]
pub struct EditorConfig {
    pub base_assets_dir: PathBuf,
    pub extended_assets_dir: Option<PathBuf>,
}

/// State for the sprite registration dialog
#[derive(Resource, Default)]
pub struct SpriteRegistrationDialog {
    /// Whether the dialog is showing
    pub show: bool,
    /// Unregistered sprites found in the mob
    pub unregistered_sprites: Vec<String>,
    /// Path to save to after handling
    pub pending_save_path: Option<PathBuf>,
}

/// State for the sprite selection confirmation dialog
#[derive(Resource, Default)]
pub struct SpriteSelectionDialog {
    /// Whether the dialog is showing
    pub show: bool,
    /// The asset path of the registered sprite
    pub asset_path: String,
    /// The path to use in the mob file
    pub mob_path: String,
    /// Display name for the sprite
    pub display_name: String,
}

/// Initial scan of the mobs directories
fn initial_scan_system(
    mut file_tree: ResMut<FileTreeState>,
    config: Res<EditorConfig>,
    mut next_state: ResMut<NextState<EditorState>>,
) {
    file_tree.scan_directories(&config.base_assets_dir, config.extended_assets_dir.as_ref());
    next_state.set(EditorState::Browsing);
}

/// Initial scan of sprites from .assets.ron files
fn initial_sprite_scan(mut sprite_registry: ResMut<SpriteRegistry>) {
    scan_sprite_registry(&mut sprite_registry);
}

/// Scan .assets.ron files and populate the sprite registry
fn scan_sprite_registry(registry: &mut SpriteRegistry) {
    registry.sprites.clear();
    registry.parse_errors.clear();

    let cwd = std::env::current_dir().unwrap_or_default();

    // Scan base assets
    let base_assets_ron = cwd.join("assets/game.assets.ron");
    if base_assets_ron.exists() {
        match parse_assets_ron(&base_assets_ron) {
            Ok(paths) => {
                for path in paths {
                    let display_name = extract_sprite_display_name(&path);
                    registry.sprites.push(RegisteredSprite {
                        asset_path: path,
                        display_name,
                        source: SpriteSource::Base,
                    });
                }
            }
            Err(e) => registry.parse_errors.push(e),
        }
    }

    // Scan extended assets
    let extended_assets_ron = cwd.join("thetawave-test-game/assets/game.assets.ron");
    if extended_assets_ron.exists() {
        match parse_assets_ron(&extended_assets_ron) {
            Ok(paths) => {
                for path in paths {
                    let display_name = extract_sprite_display_name(&path);
                    registry.sprites.push(RegisteredSprite {
                        asset_path: path,
                        display_name,
                        source: SpriteSource::Extended,
                    });
                }
            }
            Err(e) => registry.parse_errors.push(e),
        }
    }

    // Sort by display name
    registry.sprites.sort_by(|a, b| a.display_name.cmp(&b.display_name));
    registry.needs_refresh = false;

    info!(
        "Loaded {} sprites ({} base, {} extended)",
        registry.sprites.len(),
        registry.base_sprites().count(),
        registry.extended_sprites().count()
    );
}

/// Extract display name (file stem) from a sprite path
fn extract_sprite_display_name(path: &str) -> String {
    std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(path)
        .to_string()
}

/// Check if file tree needs refresh
fn check_file_refresh(mut file_tree: ResMut<FileTreeState>, config: Res<EditorConfig>) {
    if file_tree.needs_refresh {
        file_tree.scan_directories(&config.base_assets_dir, config.extended_assets_dir.as_ref());
    }
}

/// Check if sprite registry needs refresh
fn check_sprite_registry_refresh(mut sprite_registry: ResMut<SpriteRegistry>) {
    if sprite_registry.needs_refresh {
        scan_sprite_registry(&mut sprite_registry);
    }
}

/// Handle loading a mob file
fn handle_load_mob(
    mut events: MessageReader<LoadMobEvent>,
    mut session: ResMut<EditorSession>,
    mut next_state: ResMut<NextState<EditorState>>,
    time: Res<Time>,
) {
    for event in events.read() {
        // Determine file type
        let is_patch = event.path.extension().is_some_and(|e| e == "mobpatch");
        let file_type = if is_patch {
            crate::data::FileType::MobPatch
        } else {
            crate::data::FileType::Mob
        };

        // Load file (with merging for patches)
        let load_result = if is_patch {
            FileOperations::load_patch_with_base(&event.path)
                .map(|(patch, base, merged)| (patch, base, merged))
        } else {
            FileOperations::load_file(&event.path).map(|v| (v, None, None))
        };

        match load_result {
            Ok((value, base, merged)) => {
                session.current_mob = Some(value.clone());
                session.original_mob = Some(value);
                session.base_mob = base;
                session.merged_for_preview = merged;
                session.current_path = Some(event.path.clone());
                session.file_type = file_type;
                session.is_modified = false;
                session.history.clear();
                session.selected_collider = None;
                session.selected_behavior_node = None;

                let status = if is_patch && session.merged_for_preview.is_some() {
                    format!("Loaded patch (merged with base): {}", event.path.display())
                } else if is_patch {
                    format!("Loaded patch (no base found): {}", event.path.display())
                } else {
                    format!("Loaded: {}", event.path.display())
                };
                session.set_status(status, &time);

                next_state.set(EditorState::Editing);
            }
            Err(e) => {
                session.set_status(format!("Error loading file: {}", e), &time);
            }
        }
    }
}

/// Handle saving the current mob
fn handle_save_mob(
    mut events: MessageReader<SaveMobEvent>,
    mut session: ResMut<EditorSession>,
    sprite_registry: Res<SpriteRegistry>,
    mut registration_dialog: ResMut<SpriteRegistrationDialog>,
    time: Res<Time>,
) {
    for event in events.read() {
        // Clone the path to avoid borrow checker issues
        let path = event
            .path
            .clone()
            .or_else(|| session.current_path.clone());

        let Some(path) = path else {
            session.set_status("No file path specified", &time);
            continue;
        };

        let Some(mob) = session.current_mob.clone() else {
            session.set_status("No mob to save", &time);
            continue;
        };

        // Check for unregistered sprites (only if dialog isn't already showing)
        if !registration_dialog.show {
            let unregistered = find_unregistered_sprites(&mob, &sprite_registry);
            if !unregistered.is_empty() {
                // Show dialog instead of saving
                registration_dialog.show = true;
                registration_dialog.unregistered_sprites = unregistered;
                registration_dialog.pending_save_path = Some(path);
                continue;
            }
        }

        match FileOperations::save_file(&path, &mob) {
            Ok(()) => {
                // Update original_mob to match current after save
                session.original_mob = session.current_mob.clone();
                session.is_modified = false;
                if event.path.is_some() {
                    session.current_path = event.path.clone();
                }
                session.set_status(format!("Saved: {}", path.display()), &time);
            }
            Err(e) => {
                session.set_status(format!("Error saving: {}", e), &time);
            }
        }
    }
}

/// Find all unregistered sprites in a mob
fn find_unregistered_sprites(mob: &toml::Value, registry: &SpriteRegistry) -> Vec<String> {
    let mut unregistered = Vec::new();

    // Check main sprite
    if let Some(sprite) = mob.get("sprite").and_then(|v| v.as_str()) {
        if !sprite.is_empty() && !registry.is_registered(sprite) {
            unregistered.push(sprite.to_string());
        }
    }

    // Check decorations
    if let Some(decorations) = mob.get("decorations").and_then(|v| v.as_array()) {
        for dec in decorations {
            if let Some(arr) = dec.as_array() {
                if let Some(sprite) = arr.first().and_then(|v| v.as_str()) {
                    if !sprite.is_empty()
                        && !registry.is_registered(sprite)
                        && !unregistered.contains(&sprite.to_string())
                    {
                        unregistered.push(sprite.to_string());
                    }
                }
            }
        }
    }

    unregistered
}

/// Handle reloading the current mob from disk
fn handle_reload_mob(
    mut events: MessageReader<ReloadMobEvent>,
    session: Res<EditorSession>,
    mut load_events: MessageWriter<LoadMobEvent>,
) {
    for _ in events.read() {
        if let Some(path) = session.current_path.clone() {
            load_events.write(LoadMobEvent { path });
        }
    }
}

/// Handle creating a new mob file
fn handle_new_mob(
    mut events: MessageReader<NewMobEvent>,
    mut session: ResMut<EditorSession>,
    mut file_tree: ResMut<FileTreeState>,
    mut next_state: ResMut<NextState<EditorState>>,
    time: Res<Time>,
) {
    for event in events.read() {
        match FileOperations::create_new_file(&event.path, &event.name, event.is_patch) {
            Ok(value) => {
                session.current_mob = Some(value.clone());
                session.original_mob = Some(value);
                session.current_path = Some(event.path.clone());
                session.file_type = if event.is_patch {
                    crate::data::FileType::MobPatch
                } else {
                    crate::data::FileType::Mob
                };
                session.is_modified = false;
                session.history.clear();
                session.set_status(format!("Created: {}", event.path.display()), &time);

                file_tree.needs_refresh = true;
                next_state.set(EditorState::Editing);
            }
            Err(e) => {
                session.set_status(format!("Error creating file: {}", e), &time);
            }
        }
    }
}

/// Handle deleting a mob file
fn handle_delete_mob(
    mut events: MessageReader<DeleteMobEvent>,
    mut session: ResMut<EditorSession>,
    mut file_tree: ResMut<FileTreeState>,
    mut next_state: ResMut<NextState<EditorState>>,
    time: Res<Time>,
) {
    for event in events.read() {
        match FileOperations::delete_file(&event.path) {
            Ok(()) => {
                // If we deleted the current file, clear session
                if session.current_path.as_ref() == Some(&event.path) {
                    session.current_mob = None;
                    session.current_path = None;
                    session.is_modified = false;
                    next_state.set(EditorState::Browsing);
                }

                file_tree.needs_refresh = true;
                session.set_status(format!("Deleted: {}", event.path.display()), &time);
            }
            Err(e) => {
                session.set_status(format!("Error deleting: {}", e), &time);
            }
        }
    }
}

/// Handle keyboard shortcuts
fn handle_keyboard_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    mut session: ResMut<EditorSession>,
    mut save_events: MessageWriter<SaveMobEvent>,
    mut reload_events: MessageWriter<ReloadMobEvent>,
) {
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);

    if ctrl {
        // Ctrl+S - Save
        if keys.just_pressed(KeyCode::KeyS) {
            if session.current_path.is_some() && session.is_modified {
                save_events.write(SaveMobEvent { path: None });
            }
        }

        // Ctrl+R - Reload
        if keys.just_pressed(KeyCode::KeyR) {
            if session.current_path.is_some() {
                reload_events.write(ReloadMobEvent);
            }
        }

        // Ctrl+Z - Undo
        if keys.just_pressed(KeyCode::KeyZ) && !keys.pressed(KeyCode::ShiftLeft) {
            if let Some(mob) = session.current_mob.clone() {
                if let Some(prev) = session.history.undo(&mob) {
                    session.current_mob = Some(prev);
                    session.check_modified();
                }
            }
        }

        // Ctrl+Shift+Z or Ctrl+Y - Redo
        if (keys.just_pressed(KeyCode::KeyZ) && keys.pressed(KeyCode::ShiftLeft))
            || keys.just_pressed(KeyCode::KeyY)
        {
            if let Some(mob) = session.current_mob.clone() {
                if let Some(next) = session.history.redo(&mob) {
                    session.current_mob = Some(next);
                    session.check_modified();
                }
            }
        }
    }
}
