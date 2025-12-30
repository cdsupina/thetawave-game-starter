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
        check_preview_update, draw_collider_gizmos, draw_grid, draw_joint_gizmos,
        draw_spawner_gizmos, handle_camera_input, rebuild_jointed_mob_cache,
        setup_preview_camera, update_decoration_positions, update_preview_camera,
        update_preview_mob, update_preview_settings, JointedMobCache, PreviewSettings,
        PreviewState,
    },
    states::{DialogState, EditingMode, EditorState},
    ui::{main_ui_system, DeleteDialogState, ErrorDialog, NewFolderDialog, NewMobDialog, UnsavedChangesDialog, ValidationDialog},
};

// =============================================================================
// Path constants used throughout the editor
// =============================================================================

/// Directory containing sprite files relative to assets directory.
pub const SPRITE_DIR: &str = "media/aseprite";

/// Directory containing mob files relative to assets directory.
pub const MOBS_DIR: &str = "mobs";

/// Prefix for extended asset paths in .mobpatch files.
pub const EXTENDED_PREFIX: &str = "extended://";

/// Main plugin for the mob editor
pub struct MobEditorPlugin {
    /// Base assets directory (where the main game's mobs are)
    pub base_assets_dir: PathBuf,
    /// Extended assets directory (for game-specific overrides)
    pub extended_assets_dir: Option<PathBuf>,
    /// Whether to show base mobs in the file tree.
    /// When false, only extended mobs are shown and users can only create patches.
    /// Set to false when using the editor as a library in an external game.
    pub show_base_mobs: bool,
}

impl Default for MobEditorPlugin {
    fn default() -> Self {
        Self {
            base_assets_dir: PathBuf::from("assets/mobs"),
            extended_assets_dir: Some(PathBuf::from("thetawave-test-game/assets/mobs")),
            show_base_mobs: true,
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
            .init_resource::<DeleteDialogState>()
            .init_resource::<UnsavedChangesDialog>()
            .init_resource::<ErrorDialog>()
            .init_resource::<ValidationDialog>()
            .init_resource::<NewFolderDialog>()
            .init_resource::<NewMobDialog>()
            .init_resource::<SpriteRegistry>()
            .init_resource::<SpriteRegistrationDialog>()
            .init_resource::<SpriteSelectionDialog>()
            .init_resource::<DecorationSelectionDialog>()
            .init_resource::<SpriteBrowserDialog>()
            .init_resource::<JointedMobCache>();

        // Store config
        app.insert_resource(EditorConfig {
            base_assets_dir: self.base_assets_dir.clone(),
            extended_assets_dir: self.extended_assets_dir.clone(),
            show_base_mobs: self.show_base_mobs,
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
                rebuild_jointed_mob_cache.after(check_preview_update),
                update_preview_mob.after(rebuild_jointed_mob_cache),
                update_decoration_positions.after(update_preview_mob),
                update_preview_camera.after(update_preview_settings),
            ),
        );

        // Gizmo systems (run after preview update)
        app.add_systems(
            Update,
            (draw_grid, draw_collider_gizmos, draw_spawner_gizmos, draw_joint_gizmos),
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
                handle_window_close,
            ),
        );
    }
}

/// Editor configuration resource
#[derive(Resource, Clone)]
pub struct EditorConfig {
    /// Base mobs directory (e.g., "assets/mobs")
    pub base_assets_dir: PathBuf,
    /// Extended mobs directory (e.g., "my-game/assets/mobs")
    pub extended_assets_dir: Option<PathBuf>,
    /// Whether to show base mobs in the file tree.
    /// When false, only extended mobs are shown and users can only create patches.
    pub show_base_mobs: bool,
}

impl EditorConfig {
    /// Get the base assets root directory (parent of mobs dir)
    /// e.g., "assets/mobs" -> "assets"
    pub fn base_assets_root(&self) -> Option<PathBuf> {
        self.base_assets_dir.parent().map(|p| p.to_path_buf())
    }

    /// Get the extended assets root directory (parent of mobs dir)
    /// e.g., "my-game/assets/mobs" -> "my-game/assets"
    pub fn extended_assets_root(&self) -> Option<PathBuf> {
        self.extended_assets_dir.as_ref().and_then(|p| p.parent()).map(|p| p.to_path_buf())
    }

    /// Get the base game.assets.ron path
    pub fn base_assets_ron(&self) -> Option<PathBuf> {
        self.base_assets_root().map(|p| p.join("game.assets.ron"))
    }

    /// Get the extended game.assets.ron path
    pub fn extended_assets_ron(&self) -> Option<PathBuf> {
        self.extended_assets_root().map(|p| p.join("game.assets.ron"))
    }

    /// Check if a path is within the extended assets directory.
    ///
    /// Uses `Path::starts_with` for correct path prefix checking,
    /// avoiding false positives that could occur with string contains.
    pub fn is_extended_path(&self, path: &std::path::Path) -> bool {
        if let Some(extended) = &self.extended_assets_dir {
            // Use Path::starts_with for proper path prefix checking
            // This handles path normalization correctly
            path.starts_with(extended)
        } else {
            false
        }
    }

    /// Resolve a sprite path to a filesystem path, checking extended first then base
    pub fn resolve_sprite_path(&self, relative_path: &str) -> Option<PathBuf> {
        let relative_path = relative_path.strip_prefix("extended://").unwrap_or(relative_path);
        let cwd = std::env::current_dir().ok()?;

        // Try extended first if available
        if let Some(extended_root) = self.extended_assets_root() {
            let extended_path = cwd.join(&extended_root).join(relative_path);
            if extended_path.exists() {
                return Some(extended_path);
            }
        }

        // Try base
        if let Some(base_root) = self.base_assets_root() {
            let base_path = cwd.join(&base_root).join(relative_path);
            if base_path.exists() {
                return Some(base_path);
            }
        }

        None
    }

    /// Resolve a mob_ref path (e.g., "mobs/enemy.mob") to a filesystem path
    pub fn resolve_mob_ref(&self, mob_ref: &str) -> Option<PathBuf> {
        let cwd = std::env::current_dir().ok()?;

        // Try base assets first
        if let Some(base_root) = self.base_assets_root() {
            let base_path = cwd.join(&base_root).join(mob_ref);
            if base_path.exists() {
                return Some(base_path);
            }
        }

        // Try extended assets
        if let Some(extended_root) = self.extended_assets_root() {
            let extended_path = cwd.join(&extended_root).join(mob_ref);
            if extended_path.exists() {
                return Some(extended_path);
            }
        }

        None
    }
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

/// State for the decoration sprite selection confirmation dialog
#[derive(Resource, Default)]
pub struct DecorationSelectionDialog {
    /// Whether the dialog is showing
    pub show: bool,
    /// The decoration index to update
    pub decoration_index: usize,
    /// The path to use in the mob file
    pub mob_path: String,
    /// Display name for the sprite
    pub display_name: String,
}

/// Target for sprite browser selection
#[derive(Clone, Default)]
pub enum SpriteBrowserTarget {
    #[default]
    MainSprite,
    Decoration(usize),
}

/// Entry in the sprite browser file list
#[derive(Clone)]
pub struct SpriteBrowserEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_directory: bool,
}

/// State for the sprite browser dialog
#[derive(Resource, Default)]
pub struct SpriteBrowserDialog {
    /// Whether the dialog is showing
    pub is_open: bool,
    /// What we're selecting a sprite for
    pub target: SpriteBrowserTarget,
    /// Currently browsing base (false) or extended (true) assets
    pub browsing_extended: bool,
    /// Current directory path relative to assets root
    pub current_path: Vec<String>,
    /// Entries in current directory
    pub entries: Vec<SpriteBrowserEntry>,
    /// Currently selected file (if any)
    pub selected: Option<PathBuf>,
    /// Whether extended assets are allowed (based on file type)
    pub allow_extended: bool,
}

impl SpriteBrowserDialog {
    /// Open the browser for a specific target
    fn open(&mut self, target: SpriteBrowserTarget, allow_extended: bool, config: &EditorConfig) {
        self.is_open = true;
        self.target = target;
        self.allow_extended = allow_extended;
        self.browsing_extended = false;
        // Start in media/aseprite where sprite files are located
        self.current_path = vec!["media".to_string(), "aseprite".to_string()];
        self.selected = None;
        self.scan_current_directory(config);
    }

    /// Open the browser for main sprite selection
    pub fn open_for_sprite(&mut self, allow_extended: bool, config: &EditorConfig) {
        self.open(SpriteBrowserTarget::MainSprite, allow_extended, config);
    }

    /// Open the browser for decoration sprite selection
    pub fn open_for_decoration(&mut self, index: usize, allow_extended: bool, config: &EditorConfig) {
        self.open(SpriteBrowserTarget::Decoration(index), allow_extended, config);
    }

    /// Close the browser
    pub fn close(&mut self) {
        self.is_open = false;
        self.selected = None;
        self.entries.clear();
    }

    /// Get the current assets directory being browsed
    /// Note: config stores mobs directories (assets/mobs), but we need the parent assets directory
    fn get_assets_dir(&self, config: &EditorConfig) -> Option<PathBuf> {
        if self.browsing_extended {
            // extended_assets_dir is like "thetawave-test-game/assets/mobs"
            // We need "thetawave-test-game/assets"
            config.extended_assets_dir.as_ref().and_then(|p| p.parent()).map(|p| p.to_path_buf())
        } else {
            // base_assets_dir is like "assets/mobs"
            // We need "assets"
            config.base_assets_dir.parent().map(|p| p.to_path_buf())
        }
    }

    /// Scan the current directory for sprite files
    pub fn scan_current_directory(&mut self, config: &EditorConfig) {
        self.entries.clear();
        self.selected = None;

        let Some(assets_dir) = self.get_assets_dir(config) else {
            return;
        };

        // Build full path from current_path components
        let mut full_path = assets_dir;
        for component in &self.current_path {
            full_path = full_path.join(component);
        }

        let Ok(entries) = std::fs::read_dir(&full_path) else {
            return;
        };

        let mut dirs = Vec::new();
        let mut files = Vec::new();

        for entry in entries.flatten() {
            let entry_path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden files
            if name.starts_with('.') {
                continue;
            }

            if entry_path.is_dir() {
                dirs.push(SpriteBrowserEntry {
                    name,
                    path: entry_path,
                    is_directory: true,
                });
            } else if let Some(ext) = entry_path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if ext_str == "aseprite" || ext_str == "ase" {
                    files.push(SpriteBrowserEntry {
                        name,
                        path: entry_path,
                        is_directory: false,
                    });
                }
            }
        }

        // Sort alphabetically
        dirs.sort_by(|a, b| a.name.cmp(&b.name));
        files.sort_by(|a, b| a.name.cmp(&b.name));

        // Directories first, then files
        self.entries.extend(dirs);
        self.entries.extend(files);
    }

    /// Navigate into a subdirectory
    pub fn navigate_into(&mut self, dir_name: &str, config: &EditorConfig) {
        self.current_path.push(dir_name.to_string());
        self.scan_current_directory(config);
    }

    /// Navigate up to parent directory
    pub fn navigate_up(&mut self, config: &EditorConfig) {
        if !self.current_path.is_empty() {
            self.current_path.pop();
            self.scan_current_directory(config);
        }
    }

    /// Switch between base and extended assets
    pub fn switch_assets_source(&mut self, use_extended: bool, config: &EditorConfig) {
        if use_extended && !self.allow_extended {
            return;
        }
        self.browsing_extended = use_extended;
        // Reset to media/aseprite when switching sources
        self.current_path = vec!["media".to_string(), "aseprite".to_string()];
        self.scan_current_directory(config);
    }

    /// Get the asset path for a selected file (relative to assets root)
    pub fn get_asset_path(&self, file_path: &PathBuf, config: &EditorConfig) -> Option<String> {
        let assets_dir = self.get_assets_dir(config)?;
        let relative = file_path.strip_prefix(&assets_dir).ok()?;
        Some(relative.to_string_lossy().to_string())
    }
}

/// Initial scan of the mobs directories
fn initial_scan_system(
    mut file_tree: ResMut<FileTreeState>,
    config: Res<EditorConfig>,
    mut next_state: ResMut<NextState<EditorState>>,
) {
    file_tree.scan_directories(&config.base_assets_dir, config.extended_assets_dir.as_ref(), config.show_base_mobs);
    next_state.set(EditorState::Browsing);
}

/// Initial scan of sprites from .assets.ron files
fn initial_sprite_scan(mut sprite_registry: ResMut<SpriteRegistry>, config: Res<EditorConfig>) {
    scan_sprite_registry(&mut sprite_registry, &config);
}

/// Scan .assets.ron files and populate the sprite registry
fn scan_sprite_registry(registry: &mut SpriteRegistry, config: &EditorConfig) {
    registry.sprites.clear();
    registry.parse_errors.clear();

    let cwd = std::env::current_dir().unwrap_or_default();

    // Scan base assets
    if let Some(base_assets_ron) = config.base_assets_ron() {
        let base_assets_ron = cwd.join(&base_assets_ron);
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
    }

    // Scan extended assets
    if let Some(extended_assets_ron) = config.extended_assets_ron() {
        let extended_assets_ron = cwd.join(&extended_assets_ron);
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
        file_tree.scan_directories(&config.base_assets_dir, config.extended_assets_dir.as_ref(), config.show_base_mobs);
    }
}

/// Check if sprite registry needs refresh
fn check_sprite_registry_refresh(mut sprite_registry: ResMut<SpriteRegistry>, config: Res<EditorConfig>) {
    if sprite_registry.needs_refresh {
        scan_sprite_registry(&mut sprite_registry, &config);
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
                session.selected_collider = None;
                session.selected_behavior_node = None;

                let status = if is_patch && session.merged_for_preview.is_some() {
                    format!("Loaded patch (merged with base): {}", event.path.display())
                } else if is_patch {
                    format!("Loaded patch (no base found): {}", event.path.display())
                } else {
                    format!("Loaded: {}", event.path.display())
                };
                session.log_success(status, &time);

                next_state.set(EditorState::Editing);
            }
            Err(e) => {
                session.log_error(format!("Error loading file: {}", e), &time);
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
    mut validation_dialog: ResMut<ValidationDialog>,
    time: Res<Time>,
) {
    use crate::data::validate_mob;

    for event in events.read() {
        // Clone the path to avoid borrow checker issues
        let path = event
            .path
            .clone()
            .or_else(|| session.current_path.clone());

        let Some(path) = path else {
            session.log_warning("No file path specified", &time);
            continue;
        };

        let Some(mob) = session.current_mob.clone() else {
            session.log_warning("No mob to save", &time);
            continue;
        };

        // Run validation (skip if validation dialog is already open)
        if !validation_dialog.is_open {
            let is_patch = session.file_type == crate::data::FileType::MobPatch;
            let validation_result = validate_mob(&mob, is_patch);
            if validation_result.has_errors() {
                validation_dialog.is_open = true;
                validation_dialog.errors = validation_result.errors;
                session.log_warning("Validation errors found - please review", &time);
                continue;
            }
        }

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
                session.log_success(format!("Saved: {}", path.display()), &time);
            }
            Err(e) => {
                session.log_error(format!("Error saving: {}", e), &time);
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
                session.log_success(format!("Created: {}", event.path.display()), &time);

                file_tree.needs_refresh = true;
                next_state.set(EditorState::Editing);
            }
            Err(e) => {
                session.log_error(format!("Error creating file: {}", e), &time);
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
                session.log_success(format!("Deleted: {}", event.path.display()), &time);
            }
            Err(e) => {
                session.log_error(format!("Error deleting: {}", e), &time);
            }
        }
    }
}

/// Handle keyboard shortcuts
fn handle_keyboard_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    session: Res<EditorSession>,
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

    }
}

/// Handle window close request - check for unsaved changes
fn handle_window_close(
    mut close_events: MessageReader<bevy::window::WindowCloseRequested>,
    mut exit_writer: MessageWriter<bevy::app::AppExit>,
    session: Res<EditorSession>,
    mut unsaved_dialog: ResMut<UnsavedChangesDialog>,
) {
    use crate::ui::UnsavedAction;

    for _event in close_events.read() {
        if session.is_modified {
            // Show unsaved changes dialog instead of closing
            unsaved_dialog.open(UnsavedAction::Exit);
        } else {
            // No unsaved changes, exit immediately
            exit_writer.write(bevy::app::AppExit::Success);
        }
    }
}
