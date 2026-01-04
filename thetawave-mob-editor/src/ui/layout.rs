//! Main UI layout and dialog rendering.
//!
//! Contains the main UI system that orchestrates all panels,
//! status bar, and modal dialogs.

use bevy::{
    ecs::{message::MessageWriter, system::SystemParam},
    log::warn,
    prelude::{Local, Res, ResMut, State, Time},
};
use bevy_egui::{EguiContexts, egui};

use crate::{
    data::{AssetSource, EditorSession, MobAssetRegistry, SpriteRegistry},
    file::{
        DeleteDirectoryEvent, DeleteMobEvent, FileTreeState, LoadMobEvent, NewMobEvent,
        ReloadMobEvent, SaveMobEvent, append_sprite_to_assets_ron,
    },
    plugin::{
        DecorationSelectionDialog, EditorConfig, SpriteBrowserDialog, SpriteBrowserTarget,
        SpriteRegistrationDialog, SpriteSelectionDialog,
    },
    preview::{PreviewSettings, PreviewState},
    states::EditorState,
};

// =============================================================================
// UI Layout Constants
// =============================================================================

/// Number of frames to skip at startup to allow egui to initialize
const STARTUP_FRAMES_TO_SKIP: u8 = 2;

/// Default width of the properties panel in pixels
const PROPERTIES_PANEL_DEFAULT_WIDTH: f32 = 300.0;

/// Convert a filename like "my_new_mob" to Title Case "My New Mob"
fn filename_to_display_name(filename: &str) -> String {
    filename
        .split(['_', '-'])
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Minimum width of the properties panel in pixels
const PROPERTIES_PANEL_MIN_WIDTH: f32 = 250.0;

/// Height of the status log panel when expanded
const STATUS_LOG_EXPANDED_HEIGHT: f32 = 120.0;

/// Height of the status log panel when collapsed
const STATUS_LOG_COLLAPSED_HEIGHT: f32 = 24.0;

use super::{
    DeleteDialogState, DeleteDirectoryDialogState, NewFolderDialog, NewMobDialog,
    UnsavedChangesDialog, file_panel_ui, properties_panel_ui, render_delete_dialog,
    render_delete_directory_dialog, update_decoration_sprite,
};

/// Grouped message writers for the main UI system
#[derive(SystemParam)]
pub struct UiMessageWriters<'w> {
    load: MessageWriter<'w, LoadMobEvent>,
    save: MessageWriter<'w, SaveMobEvent>,
    reload: MessageWriter<'w, ReloadMobEvent>,
    new: MessageWriter<'w, NewMobEvent>,
    delete: MessageWriter<'w, DeleteMobEvent>,
    delete_dir: MessageWriter<'w, DeleteDirectoryEvent>,
    exit: MessageWriter<'w, bevy::app::AppExit>,
}

/// Grouped dialog resources for the main UI system
#[derive(SystemParam)]
pub struct DialogResources<'w> {
    pub delete_dialog: ResMut<'w, DeleteDialogState>,
    pub delete_dir_dialog: ResMut<'w, DeleteDirectoryDialogState>,
    pub unsaved_dialog: ResMut<'w, UnsavedChangesDialog>,
    pub registration_dialog: ResMut<'w, SpriteRegistrationDialog>,
    pub selection_dialog: ResMut<'w, SpriteSelectionDialog>,
    pub decoration_selection_dialog: ResMut<'w, DecorationSelectionDialog>,
    pub new_folder_dialog: ResMut<'w, NewFolderDialog>,
    pub new_mob_dialog: ResMut<'w, NewMobDialog>,
    pub sprite_browser: ResMut<'w, SpriteBrowserDialog>,
}

/// Main UI layout system that renders all egui panels
pub fn main_ui_system(
    mut contexts: EguiContexts,
    mut session: ResMut<EditorSession>,
    mut file_tree: ResMut<FileTreeState>,
    state: Res<State<EditorState>>,
    time: Res<Time>,
    mut events: UiMessageWriters,
    mut dialogs: DialogResources,
    mut preview_settings: ResMut<PreviewSettings>,
    mut preview_state: ResMut<PreviewState>,
    mut sprite_registry: ResMut<SpriteRegistry>,
    mut mob_registry: ResMut<MobAssetRegistry>,
    config: Res<EditorConfig>,
    mut frames_passed: Local<u8>,
) {
    // Skip initial frames to let egui initialize properly
    if *frames_passed < STARTUP_FRAMES_TO_SKIP {
        *frames_passed += 1;
        return;
    }

    // Skip during loading state
    if *state.get() == EditorState::Loading {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    // Reset scroll area tracking each frame (prevents zoom when scrolling in UI panels)
    preview_settings.pointer_over_ui_scroll_area = false;

    // Left panel - File browser
    let file_panel_response = egui::SidePanel::left("file_panel")
        .default_width(250.0)
        .min_width(200.0)
        .resizable(true)
        .show(ctx, |ui| {
            file_panel_ui(
                ui,
                &mut file_tree,
                &mut sprite_registry,
                &mut mob_registry,
                &session,
                &config,
                &mut events.load,
                &mut dialogs.delete_dialog,
                &mut dialogs.delete_dir_dialog,
                &mut dialogs.unsaved_dialog,
                &mut dialogs.new_folder_dialog,
                &mut dialogs.new_mob_dialog,
            );
        });
    if file_panel_response.response.contains_pointer() {
        preview_settings.pointer_over_ui_scroll_area = true;
    }

    // Right panel - Properties (only when editing)
    let mut panel_result = super::PropertiesPanelResult::default();
    if *state.get() == EditorState::Editing {
        let props_panel_response = egui::SidePanel::right("properties_panel")
            .default_width(PROPERTIES_PANEL_DEFAULT_WIDTH)
            .min_width(PROPERTIES_PANEL_MIN_WIDTH)
            .resizable(true)
            .show(ctx, |ui| {
                panel_result = properties_panel_ui(
                    ui,
                    &mut session,
                    &sprite_registry,
                    &mob_registry,
                    &file_tree,
                    &config,
                    &mut events.save,
                    &mut events.reload,
                );
            });
        if props_panel_response.response.contains_pointer() {
            preview_settings.pointer_over_ui_scroll_area = true;
        }
    }

    // Open sprite browser for main sprite
    if panel_result.open_sprite_browser {
        let allow_extended = session.can_use_extended_sprites(&config);
        dialogs
            .sprite_browser
            .open_for_sprite(allow_extended, &config);
    }

    // Open sprite browser for decoration
    if let Some(decoration_index) = panel_result.open_decoration_browser {
        let allow_extended = session.can_use_extended_sprites(&config);
        dialogs
            .sprite_browser
            .open_for_decoration(decoration_index, allow_extended, &config);
    }

    // Handle mob registration
    if panel_result.register_mob
        && let Some(path) = &session.current_path
    {
        let is_extended = config.is_extended_path(path);
        let is_patch = path.extension().is_some_and(|e| e == "mobpatch");

        // Get the mobs.assets.ron path
        let mobs_assets_path = if is_extended {
            config.extended_mobs_assets_ron()
        } else {
            config.base_mobs_assets_ron()
        };

        if let Some(mobs_assets_path) = mobs_assets_path {
            // Calculate relative path for the assets.ron file
            let assets_root = if is_extended {
                config.extended_assets_root()
            } else {
                config.base_assets_root()
            };

            if let Some(assets_root) = assets_root {
                if let Ok(relative) = path.strip_prefix(&assets_root) {
                    let relative_str = relative.to_string_lossy().to_string();
                    match crate::file::append_to_mobs_assets_ron(
                        &mobs_assets_path,
                        &relative_str,
                        is_patch,
                        is_extended,
                    ) {
                        Ok(()) => {
                            mob_registry.needs_refresh = true;
                            session.log_success(
                                format!("Registered {} in mobs.assets.ron", relative_str),
                                &time,
                            );
                        }
                        Err(e) => {
                            session.log_error(format!("Failed to register: {}", e), &time);
                        }
                    }
                } else {
                    session.log_error("Could not calculate relative path".to_string(), &time);
                }
            }
        } else {
            session.log_error(
                "Could not determine mobs.assets.ron path".to_string(),
                &time,
            );
        }
    }

    // Handle sprite registration from properties panel
    for sprite_path in panel_result.register_sprites {
        // Determine if this is an extended sprite and extract asset path
        let is_extended = sprite_path.starts_with("extended://");
        let asset_path = sprite_path
            .strip_prefix("extended://")
            .unwrap_or(&sprite_path);

        // Get the appropriate assets.ron path
        let cwd = match std::env::current_dir() {
            Ok(dir) => dir,
            Err(e) => {
                session.log_error(format!("Failed to get working directory: {}", e), &time);
                continue;
            }
        };
        let assets_ron = if is_extended {
            config.extended_assets_ron().map(|p| cwd.join(p))
        } else {
            config.base_assets_ron().map(|p| cwd.join(p))
        };

        if let Some(assets_ron) = assets_ron {
            match append_sprite_to_assets_ron(&assets_ron, asset_path, is_extended) {
                Ok(()) => {
                    sprite_registry.needs_refresh = true;
                    session.log_success(format!("Registered sprite: {}", asset_path), &time);
                }
                Err(e) => {
                    session.log_error(format!("Failed to register sprite: {}", e), &time);
                }
            }
        } else {
            session.log_error(
                format!(
                    "Could not determine assets.ron path for {}",
                    if is_extended { "extended" } else { "base" }
                ),
                &time,
            );
        }
    }

    // Bottom status bar / log panel
    let panel_height = if session.log.expanded {
        STATUS_LOG_EXPANDED_HEIGHT
    } else {
        STATUS_LOG_COLLAPSED_HEIGHT
    };
    let status_bar_response = egui::TopBottomPanel::bottom("status_bar")
        .exact_height(panel_height)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Right-aligned buttons FIRST (so they're always visible)
                // Reserve space by adding them with right-to-left layout
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Clear button (only when there are entries)
                    if !session.log.is_empty() && ui.small_button("Clear").clicked() {
                        session.log.clear();
                    }

                    // Expand/collapse toggle
                    let toggle_text = if session.log.expanded {
                        "Collapse"
                    } else {
                        "Log"
                    };
                    if ui.small_button(toggle_text).clicked() {
                        session.log.expanded = !session.log.expanded;
                    }

                    ui.separator();

                    // When collapsed, show last message (truncated if needed)
                    if !session.log.expanded
                        && let Some(entry) = session.log.last()
                    {
                        // Truncate long messages
                        let text = if entry.text.len() > 60 {
                            format!("{}...", &entry.text[..57])
                        } else {
                            entry.text.clone()
                        };
                        ui.colored_label(entry.level.color(), text);
                        ui.separator();
                    }

                    // Modified indicator
                    if session.is_modified {
                        ui.colored_label(egui::Color32::YELLOW, "Modified");
                    } else {
                        ui.label("Saved");
                    }

                    ui.separator();

                    // File path (truncated if needed)
                    if let Some(path) = &session.current_path {
                        let path_str = path.display().to_string();
                        let display_path = if path_str.len() > 40 {
                            format!("...{}", &path_str[path_str.len() - 37..])
                        } else {
                            path_str
                        };
                        ui.label(format!("File: {}", display_path));
                    } else {
                        ui.label("No file loaded");
                    }
                });
            });

            // When expanded, show scrollable log
            if session.log.expanded {
                ui.separator();
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for entry in session.log.iter() {
                            ui.horizontal(|ui| {
                                // Timestamp (relative seconds)
                                let elapsed = time.elapsed_secs_f64() - entry.timestamp;
                                let time_str = if elapsed < 60.0 {
                                    format!("{:.0}s ago", elapsed)
                                } else if elapsed < 3600.0 {
                                    format!("{:.0}m ago", elapsed / 60.0)
                                } else {
                                    format!("{:.0}h ago", elapsed / 3600.0)
                                };
                                ui.label(
                                    egui::RichText::new(time_str)
                                        .small()
                                        .color(egui::Color32::GRAY),
                                );
                                ui.colored_label(entry.level.color(), &entry.text);
                            });
                        }
                    });
            }
        });
    // Only block scroll for status bar when log is expanded (has scrollable content)
    if session.log.expanded && status_bar_response.response.contains_pointer() {
        preview_settings.pointer_over_ui_scroll_area = true;
    }

    // Central panel - Preview area (transparent to show 2D camera view)
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            match *state.get() {
                EditorState::Loading => {
                    ui.centered_and_justified(|ui| {
                        ui.heading("Loading...");
                    });
                }
                EditorState::Browsing => {
                    ui.centered_and_justified(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.heading("Mob Editor");
                            ui.add_space(20.0);
                            ui.label("Select a .mob or .mobpatch file from the file browser,");
                            ui.label("or right-click a folder to create a new one.");
                        });
                    });
                }
                EditorState::Editing => {
                    // Preview controls at the top
                    ui.horizontal_wrapped(|ui| {
                        ui.label("Preview:");

                        // Zoom controls
                        if ui.small_button("-").clicked() {
                            preview_settings.adjust_zoom(-0.2);
                        }
                        ui.label(format!("{:.0}%", preview_settings.zoom * 100.0));
                        if ui.small_button("+").clicked() {
                            preview_settings.adjust_zoom(0.2);
                        }

                        ui.separator();

                        // Toggle buttons
                        if ui
                            .selectable_label(preview_settings.show_grid, "Grid")
                            .clicked()
                        {
                            preview_settings.show_grid = !preview_settings.show_grid;
                        }
                        if ui
                            .selectable_label(preview_settings.show_colliders, "Colliders")
                            .clicked()
                        {
                            preview_settings.show_colliders = !preview_settings.show_colliders;
                        }
                        if ui
                            .selectable_label(preview_settings.show_jointed_mobs, "Jointed")
                            .clicked()
                        {
                            preview_settings.show_jointed_mobs =
                                !preview_settings.show_jointed_mobs;
                            preview_state.needs_rebuild = true;
                        }
                        if ui
                            .selectable_label(preview_settings.show_joint_gizmos, "Joints")
                            .clicked()
                        {
                            preview_settings.show_joint_gizmos =
                                !preview_settings.show_joint_gizmos;
                        }

                        ui.separator();

                        if ui.small_button("Reset").clicked() {
                            preview_settings.reset_view();
                        }
                    });

                    // Help text on separate line
                    ui.label(
                        egui::RichText::new("Scroll: zoom | Right-drag: pan | Home: reset")
                            .small()
                            .color(egui::Color32::GRAY),
                    );

                    // Sprite info collapsible section
                    egui::CollapsingHeader::new("Sprite Source")
                        .default_open(false)
                        .show(ui, |ui| {
                            render_sprite_info(ui, &mut preview_state);
                        });

                    // The rest of the central area is transparent to show the 2D camera view
                    // Just allocate the space so egui doesn't try to fill it
                    let available = ui.available_size();
                    ui.allocate_space(available);
                }
            }
        });

    // Render delete dialog windows
    render_delete_dialog(ctx, &mut dialogs.delete_dialog, &mut events.delete);
    render_delete_directory_dialog(ctx, &mut dialogs.delete_dir_dialog, &mut events.delete_dir);

    // Render sprite registration dialog
    render_registration_dialog(
        ctx,
        &mut dialogs.registration_dialog,
        &mut sprite_registry,
        &mut events.save,
        &mut session,
        &time,
        &config,
    );

    // Render sprite selection confirmation dialog
    render_selection_dialog(
        ctx,
        &mut dialogs.selection_dialog,
        &mut session,
        &time,
        &mut preview_state,
    );

    // Render decoration sprite selection confirmation dialog
    render_decoration_selection_dialog(
        ctx,
        &mut dialogs.decoration_selection_dialog,
        &mut session,
        &time,
        &mut preview_state,
    );

    // Render unsaved changes dialog
    render_unsaved_changes_dialog(
        ctx,
        &mut dialogs.unsaved_dialog,
        &mut session,
        &mut events.save,
        &mut events.load,
        &mut events.exit,
    );

    // Render new folder dialog
    render_new_folder_dialog(ctx, &mut dialogs.new_folder_dialog, &mut file_tree);

    // Render new mob dialog
    render_new_mob_dialog(
        ctx,
        &mut dialogs.new_mob_dialog,
        &mut events.new,
        &mut file_tree,
        &config,
    );

    // Render sprite browser dialog
    let browser_result =
        render_sprite_browser_dialog(ctx, &mut dialogs.sprite_browser, &config, &sprite_registry);

    // Handle sprite browser selection result
    if let Some((asset_path, is_extended)) = browser_result {
        let cwd = match std::env::current_dir() {
            Ok(dir) => dir,
            Err(e) => {
                session.log_error(format!("Failed to get working directory: {}", e), &time);
                return;
            }
        };
        let assets_ron = if is_extended {
            config.extended_assets_ron().map(|p| cwd.join(p))
        } else {
            config.base_assets_ron().map(|p| cwd.join(p))
        };
        let Some(assets_ron) = assets_ron else {
            warn!("Could not determine assets.ron path");
            return;
        };

        // Determine the mob path (with extended:// prefix if needed)
        // Extended sprites need the prefix for both patches AND extended mobs
        let is_extended_mob = session
            .current_path
            .as_ref()
            .map(|p| config.is_extended_path(p))
            .unwrap_or(false);
        let needs_extended_prefix = is_extended
            && (session.file_type == crate::data::FileType::MobPatch || is_extended_mob);
        let mob_path = if needs_extended_prefix {
            format!("extended://{}", asset_path)
        } else {
            asset_path.clone()
        };

        // Extract display name from asset path
        let display_name = std::path::Path::new(&asset_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&asset_path)
            .to_string();

        // Register sprite if needed
        // When checking extended sprites, use the extended:// prefix for proper matching
        let check_path = if is_extended {
            format!("extended://{}", asset_path)
        } else {
            asset_path.clone()
        };
        if !sprite_registry.is_registered(&check_path) {
            if let Err(e) = append_sprite_to_assets_ron(&assets_ron, &asset_path, is_extended) {
                session.log_error(format!("Failed to register sprite: {}", e), &time);
                // Don't open dialog if registration failed
                return;
            } else {
                sprite_registry.needs_refresh = true;
            }
        }

        // Open confirmation dialog to ask if user wants to set the sprite
        match dialogs.sprite_browser.target.clone() {
            SpriteBrowserTarget::MainSprite => {
                dialogs.selection_dialog.show = true;
                dialogs.selection_dialog.asset_path = asset_path;
                dialogs.selection_dialog.mob_path = mob_path;
                dialogs.selection_dialog.display_name = display_name;
            }
            SpriteBrowserTarget::Decoration(index) => {
                dialogs.decoration_selection_dialog.show = true;
                dialogs.decoration_selection_dialog.decoration_index = index;
                dialogs.decoration_selection_dialog.mob_path = mob_path;
                dialogs.decoration_selection_dialog.display_name = display_name;
            }
        }
    }

    // Block scroll when sprite browser (with scroll area) is open and pointer is over it
    if dialogs.sprite_browser.is_open && ctx.is_pointer_over_area() {
        preview_settings.pointer_over_ui_scroll_area = true;
    }
}

/// Render the unsaved changes dialog
fn render_unsaved_changes_dialog(
    ctx: &egui::Context,
    dialog: &mut UnsavedChangesDialog,
    session: &mut EditorSession,
    save_events: &mut MessageWriter<SaveMobEvent>,
    load_events: &mut MessageWriter<LoadMobEvent>,
    exit_events: &mut MessageWriter<bevy::app::AppExit>,
) {
    use super::UnsavedAction;

    if !dialog.is_open {
        return;
    }

    let mut action_to_take: Option<UnsavedAction> = None;
    let mut save_then_exit = false;
    let mut close_dialog = false;

    // Determine dialog message based on action
    let is_exit = matches!(&dialog.pending_action, Some(UnsavedAction::Exit));
    let title = if is_exit {
        "Exit with Unsaved Changes?"
    } else {
        "Unsaved Changes"
    };

    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label("You have unsaved changes. What would you like to do?");

            if let Some(path) = &session.current_path {
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(format!("File: {}", path.display()))
                        .small()
                        .color(egui::Color32::GRAY),
                );
            }

            ui.add_space(12.0);

            ui.horizontal(|ui| {
                let continue_text = if is_exit {
                    "Save & Exit"
                } else {
                    "Save & Continue"
                };
                if ui.button(continue_text).clicked() {
                    // Save first, then perform action
                    save_events.write(crate::file::SaveMobEvent {
                        path: None,
                        skip_registration_check: false,
                    });
                    if is_exit {
                        // For exit: set flag so save handler will exit after save completes
                        save_then_exit = true;
                    } else {
                        // For load: proceed with load after save
                        action_to_take = dialog.pending_action.clone();
                    }
                    close_dialog = true;
                }

                let discard_text = if is_exit {
                    "Exit Without Saving"
                } else {
                    "Discard Changes"
                };
                if ui.button(discard_text).clicked() {
                    // Perform action without saving
                    action_to_take = dialog.pending_action.clone();
                    close_dialog = true;
                }

                if ui.button("Cancel").clicked() {
                    close_dialog = true;
                }
            });
        });

    // Set the pending exit flag if "Save & Exit" was clicked
    if save_then_exit {
        session.pending_exit_after_save = true;
    }

    if let Some(action) = action_to_take {
        match action {
            UnsavedAction::LoadFile(path) => {
                load_events.write(LoadMobEvent { path });
            }
            UnsavedAction::Exit => {
                exit_events.write(bevy::app::AppExit::Success);
            }
        }
    }

    if close_dialog {
        dialog.close();
    }
}

/// Render the new folder dialog
fn render_new_folder_dialog(
    ctx: &egui::Context,
    dialog: &mut NewFolderDialog,
    file_tree: &mut FileTreeState,
) {
    if !dialog.is_open {
        return;
    }

    let mut should_create = false;
    let mut should_close = false;

    egui::Window::new("New Folder")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Folder name:");
                let response = ui.text_edit_singleline(&mut dialog.folder_name);
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    should_create = true;
                }
                // Focus the text field when dialog opens
                response.request_focus();
            });

            ui.add_space(4.0);

            // Check for invalid characters and show warning
            let has_invalid_chars = dialog
                .folder_name
                .contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|']);
            if has_invalid_chars {
                ui.label(
                    egui::RichText::new("Invalid characters: / \\ : * ? \" < > |")
                        .small()
                        .color(egui::Color32::from_rgb(255, 180, 100)),
                );
            }

            // Show error message if present
            if let Some(error) = &dialog.error_message {
                ui.label(
                    egui::RichText::new(error)
                        .small()
                        .color(egui::Color32::from_rgb(255, 100, 100)),
                );
            }

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    should_close = true;
                }

                let valid_name = !dialog.folder_name.trim().is_empty() && !has_invalid_chars;

                if ui
                    .add_enabled(valid_name, egui::Button::new("Create"))
                    .clicked()
                {
                    should_create = true;
                }
            });
        });

    if should_create && !dialog.folder_name.trim().is_empty() {
        let new_path = dialog.parent_path.join(dialog.folder_name.trim());
        if let Err(e) = std::fs::create_dir_all(&new_path) {
            dialog.error_message = Some(format!("Failed to create folder: {}", e));
            bevy::log::error!("Failed to create folder {:?}: {}", new_path, e);
            return;
        }
        file_tree.needs_refresh = true;
        dialog.close();
    } else if should_close {
        dialog.close();
    }
}

/// Render the new mob/patch dialog
fn render_new_mob_dialog(
    ctx: &egui::Context,
    dialog: &mut NewMobDialog,
    new_events: &mut MessageWriter<NewMobEvent>,
    file_tree: &mut FileTreeState,
    config: &crate::plugin::EditorConfig,
) {
    if !dialog.is_open {
        return;
    }

    let mut should_create = false;
    let mut should_close = false;

    if dialog.is_patch {
        // Patch dialog with mob selector
        render_new_patch_dialog(
            ctx,
            dialog,
            new_events,
            file_tree,
            config,
            &mut should_create,
            &mut should_close,
        );
    } else {
        // Regular mob dialog with text input
        render_new_mob_text_dialog(
            ctx,
            dialog,
            new_events,
            file_tree,
            &mut should_create,
            &mut should_close,
        );
    }

    if should_close {
        dialog.close();
    }
}

/// Render the new mob dialog (text input for filename)
fn render_new_mob_text_dialog(
    ctx: &egui::Context,
    dialog: &mut NewMobDialog,
    new_events: &mut MessageWriter<NewMobEvent>,
    file_tree: &mut FileTreeState,
    should_create: &mut bool,
    should_close: &mut bool,
) {
    egui::Window::new("New Mob")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("File name:");
                let response = ui.text_edit_singleline(&mut dialog.file_name);
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    *should_create = true;
                }
                response.request_focus();
                ui.label(".mob");
            });

            ui.add_space(4.0);

            // Check for invalid characters and show warning
            let has_invalid_chars = dialog
                .file_name
                .contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|', '.']);
            if has_invalid_chars {
                ui.label(
                    egui::RichText::new("Invalid characters: / \\ : * ? \" < > | .")
                        .small()
                        .color(egui::Color32::from_rgb(255, 180, 100)),
                );
            }

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    *should_close = true;
                }

                let valid_name = !dialog.file_name.trim().is_empty() && !has_invalid_chars;

                if ui
                    .add_enabled(valid_name, egui::Button::new("Create"))
                    .clicked()
                {
                    *should_create = true;
                }
            });
        });

    if *should_create && !dialog.file_name.trim().is_empty() {
        let file_name = format!("{}.mob", dialog.file_name.trim());
        let path = dialog.parent_path.join(&file_name);

        new_events.write(NewMobEvent {
            path,
            name: filename_to_display_name(dialog.file_name.trim()),
            is_patch: false,
        });

        file_tree.needs_refresh = true;
        dialog.close();
        *should_create = false; // Prevent double close
    }
}

/// Render the new patch dialog (dropdown selector for base mobs)
fn render_new_patch_dialog(
    ctx: &egui::Context,
    dialog: &mut NewMobDialog,
    new_events: &mut MessageWriter<NewMobEvent>,
    file_tree: &mut FileTreeState,
    config: &crate::plugin::EditorConfig,
    should_create: &mut bool,
    should_close: &mut bool,
) {
    // Get available base mobs and existing patches
    let base_mobs = file_tree.get_base_mob_refs();
    let existing_patches = file_tree.get_existing_patch_refs();

    egui::Window::new("New Patch")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label("Select a base mob to patch:");
            ui.add_space(4.0);

            // Dropdown for selecting base mob
            let selected_display = dialog
                .selected_mob_ref
                .as_deref()
                .unwrap_or("(select a mob)");

            egui::ComboBox::from_label("")
                .selected_text(selected_display)
                .width(250.0)
                .show_ui(ui, |ui| {
                    for mob_ref in &base_mobs {
                        let has_patch = existing_patches.contains(mob_ref);
                        let display_text = if has_patch {
                            format!("{} (patch exists)", mob_ref)
                        } else {
                            mob_ref.clone()
                        };

                        // Greyed out if patch exists
                        let text = if has_patch {
                            egui::RichText::new(&display_text).color(egui::Color32::GRAY)
                        } else {
                            egui::RichText::new(&display_text)
                        };

                        let is_selected = dialog.selected_mob_ref.as_ref() == Some(mob_ref);
                        let response = ui.selectable_label(is_selected, text);

                        // Only allow selection if no patch exists
                        if response.clicked() && !has_patch {
                            dialog.selected_mob_ref = Some(mob_ref.clone());
                        }
                    }
                });

            ui.add_space(8.0);

            // Show error message if present
            if let Some(error) = &dialog.error_message {
                ui.label(
                    egui::RichText::new(error)
                        .small()
                        .color(egui::Color32::from_rgb(255, 100, 100)),
                );
                ui.add_space(4.0);
            }

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    *should_close = true;
                }

                // Check if selection is valid (has a selection and no existing patch)
                let valid_selection = dialog
                    .selected_mob_ref
                    .as_ref()
                    .is_some_and(|s| !existing_patches.contains(s));

                if ui
                    .add_enabled(valid_selection, egui::Button::new("Create"))
                    .clicked()
                {
                    *should_create = true;
                }
            });
        });

    if *should_create
        && let Some(mob_ref) = &dialog.selected_mob_ref
        && let Some(extended_dir) = &config.extended_assets_dir
    {
        // Construct path: extended_dir/mob_ref.mobpatch
        let patch_path = extended_dir.join(format!("{}.mobpatch", mob_ref));

        // Create parent directories if needed
        if let Some(parent) = patch_path.parent()
            && let Err(e) = std::fs::create_dir_all(parent)
        {
            dialog.error_message = Some(format!("Failed to create directory: {}", e));
            bevy::log::error!(
                "Failed to create parent directory for patch {:?}: {}",
                patch_path,
                e
            );
            *should_create = false;
            return;
        }

        // Extract the mob name from the ref (last component) and convert to Title Case
        let base_name = mob_ref.rsplit('/').next().unwrap_or(mob_ref);
        let name = filename_to_display_name(base_name);

        new_events.write(NewMobEvent {
            path: patch_path,
            name,
            is_patch: true,
        });

        file_tree.needs_refresh = true;
        dialog.close();
        *should_create = false;
    }
}

/// Render the sprite registration dialog
fn render_registration_dialog(
    ctx: &egui::Context,
    dialog: &mut SpriteRegistrationDialog,
    sprite_registry: &mut SpriteRegistry,
    save_events: &mut MessageWriter<SaveMobEvent>,
    session: &mut EditorSession,
    time: &Time,
    config: &crate::plugin::EditorConfig,
) {
    if !dialog.show {
        return;
    }

    egui::Window::new("Unregistered Sprites")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label("The following sprites are not registered in game.assets.ron:");

            ui.add_space(8.0);

            for sprite in &dialog.unregistered_sprites {
                let display_name = std::path::Path::new(sprite)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(sprite);
                ui.horizontal(|ui| {
                    ui.label("•");
                    ui.label(egui::RichText::new(display_name).monospace());
                });
            }

            ui.add_space(8.0);
            ui.separator();
            ui.label("Would you like to register them before saving?");
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui.button("Register & Save").clicked() {
                    // Register sprites to appropriate .assets.ron file
                    let cwd = match std::env::current_dir() {
                        Ok(dir) => dir,
                        Err(e) => {
                            session
                                .log_error(format!("Failed to get working directory: {}", e), time);
                            dialog.show = false;
                            return;
                        }
                    };

                    for sprite in &dialog.unregistered_sprites {
                        let is_extended = sprite.starts_with("extended://");
                        let clean_path = sprite.strip_prefix("extended://").unwrap_or(sprite);

                        let assets_ron = if is_extended {
                            config.extended_assets_ron().map(|p| cwd.join(p))
                        } else {
                            config.base_assets_ron().map(|p| cwd.join(p))
                        };

                        let Some(assets_ron) = assets_ron else {
                            session.log_error("Could not determine assets.ron path", time);
                            continue;
                        };

                        if let Err(e) =
                            append_sprite_to_assets_ron(&assets_ron, clean_path, is_extended)
                        {
                            session.log_error(format!("Error registering sprite: {}", e), time);
                        }
                    }

                    // Mark registry for refresh
                    sprite_registry.needs_refresh = true;

                    // Proceed with save (use pending path or current file)
                    let path = dialog.pending_save_path.take();
                    save_events.write(SaveMobEvent {
                        path,
                        skip_registration_check: true, // We just registered them
                    });

                    dialog.show = false;
                    dialog.unregistered_sprites.clear();
                }

                if ui.button("Save Without Registering").clicked() {
                    // Proceed with save anyway (use pending path or current file)
                    let path = dialog.pending_save_path.take();
                    save_events.write(SaveMobEvent {
                        path,
                        skip_registration_check: true,
                    });

                    dialog.show = false;
                    dialog.unregistered_sprites.clear();
                }

                if ui.button("Cancel").clicked() {
                    dialog.show = false;
                    dialog.unregistered_sprites.clear();
                    dialog.pending_save_path = None;
                }
            });
        });
}

/// Render the sprite selection confirmation dialog
fn render_selection_dialog(
    ctx: &egui::Context,
    dialog: &mut SpriteSelectionDialog,
    session: &mut EditorSession,
    time: &Time,
    preview_state: &mut PreviewState,
) {
    if !dialog.show {
        return;
    }

    egui::Window::new("Set Sprite?")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Sprite:");
                ui.label(
                    egui::RichText::new(&dialog.display_name)
                        .monospace()
                        .strong(),
                );
            });

            ui.add_space(8.0);
            ui.label("Would you like to use this sprite for the current mob?");
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui.button("Yes, use sprite").clicked() {
                    // Set the sprite path in the mob
                    let success = if let Some(mob) =
                        session.current_mob.as_mut().and_then(|v| v.as_table_mut())
                    {
                        mob.insert(
                            "sprite".to_string(),
                            toml::Value::String(dialog.mob_path.clone()),
                        );
                        session.check_modified();
                        session.update_merged_for_preview();

                        // Trigger preview rebuild
                        preview_state.needs_rebuild = true;

                        true
                    } else {
                        false
                    };

                    if success {
                        session
                            .log_success(format!("Sprite set to: {}", dialog.display_name), time);
                    } else {
                        session.log_error(
                            "Error: Could not set sprite - no mob loaded".to_string(),
                            time,
                        );
                    }

                    dialog.show = false;
                    dialog.asset_path.clear();
                    dialog.mob_path.clear();
                    dialog.display_name.clear();
                }

                if ui.button("No, just register").clicked() {
                    session
                        .log_success(format!("Sprite registered: {}", dialog.display_name), time);

                    dialog.show = false;
                    dialog.asset_path.clear();
                    dialog.mob_path.clear();
                    dialog.display_name.clear();
                }

                if ui.button("Cancel").clicked() {
                    // Cancel - don't register or set sprite
                    dialog.show = false;
                    dialog.asset_path.clear();
                    dialog.mob_path.clear();
                    dialog.display_name.clear();
                }
            });
        });
}

/// Render the sprite info panel showing source location and override options
fn render_sprite_info(ui: &mut egui::Ui, preview_state: &mut PreviewState) {
    let sprite_info = &preview_state.sprite_info;

    // Show sprite key
    ui.horizontal(|ui| {
        ui.label("Sprite Key:");
        if let Some(key) = &sprite_info.sprite_key {
            ui.label(egui::RichText::new(key).monospace());
        } else {
            ui.label(
                egui::RichText::new("(none)")
                    .italics()
                    .color(egui::Color32::GRAY),
            );
        }
    });

    // Show load status
    ui.horizontal(|ui| {
        ui.label("Status:");
        if sprite_info.loaded_from.is_some() {
            ui.label(egui::RichText::new("Loaded").color(egui::Color32::GREEN));
        } else if let Some(error) = &sprite_info.error {
            ui.label(egui::RichText::new(error).color(egui::Color32::RED));
        }
    });

    // Show source path
    if let Some(path) = &sprite_info.loaded_from {
        ui.horizontal(|ui| {
            ui.label("Source:");
            ui.label(
                egui::RichText::new(path.display().to_string())
                    .small()
                    .monospace(),
            );
        });
    }

    // Show searched paths if not found
    if sprite_info.loaded_from.is_none() && !sprite_info.searched_paths.is_empty() {
        ui.collapsing("Searched Paths", |ui| {
            for path in &sprite_info.searched_paths {
                let exists = path.exists();
                let color = if exists {
                    egui::Color32::GREEN
                } else {
                    egui::Color32::GRAY
                };
                ui.label(
                    egui::RichText::new(path.display().to_string())
                        .small()
                        .monospace()
                        .color(color),
                );
            }
        });
    }
}

/// Render the decoration sprite selection confirmation dialog
fn render_decoration_selection_dialog(
    ctx: &egui::Context,
    dialog: &mut DecorationSelectionDialog,
    session: &mut EditorSession,
    time: &Time,
    preview_state: &mut PreviewState,
) {
    if !dialog.show {
        return;
    }

    egui::Window::new("Set Decoration Sprite?")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Sprite:");
                ui.label(
                    egui::RichText::new(&dialog.display_name)
                        .monospace()
                        .strong(),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Decoration:");
                ui.label(format!("#{}", dialog.decoration_index + 1));
            });

            ui.add_space(8.0);
            ui.label("Would you like to use this sprite for this decoration?");
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui.button("Yes, use sprite").clicked() {
                    // Set the sprite path in the decoration (also updates merged preview)
                    update_decoration_sprite(session, dialog.decoration_index, &dialog.mob_path);

                    // Trigger preview rebuild
                    preview_state.needs_rebuild = true;

                    session.log_success(
                        format!(
                            "Decoration {} sprite set to: {}",
                            dialog.decoration_index + 1,
                            dialog.display_name
                        ),
                        time,
                    );

                    dialog.show = false;
                    dialog.mob_path.clear();
                    dialog.display_name.clear();
                }

                if ui.button("No, just register").clicked() {
                    session
                        .log_success(format!("Sprite registered: {}", dialog.display_name), time);

                    dialog.show = false;
                    dialog.mob_path.clear();
                    dialog.display_name.clear();
                }

                if ui.button("Cancel").clicked() {
                    // Cancel - don't register or set sprite
                    dialog.show = false;
                    dialog.mob_path.clear();
                    dialog.display_name.clear();
                }
            });
        });
}

/// Render the sprite browser dialog
/// Returns Some((asset_path, is_extended)) if a sprite was selected and confirmed
fn render_sprite_browser_dialog(
    ctx: &egui::Context,
    dialog: &mut SpriteBrowserDialog,
    config: &EditorConfig,
    sprite_registry: &SpriteRegistry,
) -> Option<(String, bool)> {
    if !dialog.is_open {
        return None;
    }

    let mut result = None;

    egui::Window::new("Sprite Browser")
        .collapsible(false)
        .resizable(true)
        .default_width(500.0)
        .default_height(400.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(
                egui::RichText::new("Browse for an aseprite file to register in game.assets.ron")
                    .small()
                    .color(egui::Color32::GRAY),
            );
            ui.add_space(4.0);
            // Asset source tabs
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(!dialog.browsing_extended, "📦 Base Assets")
                    .clicked()
                {
                    dialog.switch_assets_source(false, config);
                }

                if dialog.allow_extended
                    && config.extended_assets_dir.is_some()
                    && ui
                        .selectable_label(dialog.browsing_extended, "📂 Extended Assets")
                        .clicked()
                {
                    dialog.switch_assets_source(true, config);
                }
            });

            ui.separator();

            // Breadcrumb navigation
            ui.horizontal(|ui| {
                let source_name = if dialog.browsing_extended {
                    "extended"
                } else {
                    "base"
                };
                if ui.small_button(source_name).clicked() {
                    dialog.current_path.clear();
                    dialog.scan_current_directory(config);
                }

                for (i, component) in dialog.current_path.clone().iter().enumerate() {
                    ui.label("/");
                    if ui.small_button(component).clicked() {
                        dialog.current_path.truncate(i + 1);
                        dialog.scan_current_directory(config);
                    }
                }
            });

            ui.separator();

            // File list with scroll area
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(280.0)
                .show(ui, |ui| {
                    // Parent directory button
                    if !dialog.current_path.is_empty()
                        && ui.selectable_label(false, "📁 ..").clicked()
                    {
                        dialog.navigate_up(config);
                    }

                    // List entries
                    let entries = dialog.entries.clone();
                    for entry in entries {
                        let is_selected = dialog.selected.as_ref() == Some(&entry.path);

                        if entry.is_directory {
                            // Directory - double click to navigate
                            let response =
                                ui.selectable_label(is_selected, format!("📁 {}", entry.name));
                            if response.double_clicked() {
                                dialog.navigate_into(&entry.name, config);
                            }
                        } else {
                            // Sprite file - click to select
                            // Build full asset path for registry check (e.g., "media/aseprite/sprite.aseprite")
                            let asset_path = if dialog.current_path.is_empty() {
                                entry.name.clone()
                            } else {
                                format!("{}/{}", dialog.current_path.join("/"), entry.name)
                            };

                            // When browsing extended, check with extended:// prefix
                            let check_path = if dialog.browsing_extended {
                                format!("extended://{}", asset_path)
                            } else {
                                asset_path.clone()
                            };

                            let label = if sprite_registry.is_registered(&check_path) {
                                format!("{} ✔", entry.name)
                            } else {
                                entry.name.clone()
                            };

                            if ui.selectable_label(is_selected, label).clicked() {
                                dialog.selected = Some(entry.path.clone());
                            }
                        }
                    }

                    if dialog.entries.is_empty() {
                        ui.colored_label(egui::Color32::GRAY, "No sprite files in this directory.");
                    }
                });

            ui.separator();

            // Selected file info and registration status
            let selected_is_registered = if let Some(selected) = &dialog.selected {
                let file_name = selected
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                // Build asset path to check registration
                // When browsing extended, add extended:// prefix for proper matching
                let asset_path = dialog.get_asset_path(selected, config);
                let check_path = asset_path.as_ref().map(|p| {
                    if dialog.browsing_extended {
                        format!("extended://{}", p)
                    } else {
                        p.clone()
                    }
                });
                let registration_info = check_path
                    .as_ref()
                    .and_then(|p| sprite_registry.find_by_path(p));

                ui.horizontal(|ui| {
                    ui.label("Selected:");
                    ui.label(egui::RichText::new(&file_name).monospace().strong());
                });

                // Show registration status
                if let Some(sprite) = registration_info {
                    let source_name = match sprite.source {
                        AssetSource::Base => "base game.assets.ron",
                        AssetSource::Extended => "extended game.assets.ron",
                    };
                    ui.label(
                        egui::RichText::new(format!("✔ Already registered in {}", source_name))
                            .small()
                            .color(egui::Color32::GREEN),
                    );
                    true
                } else {
                    ui.label(
                        egui::RichText::new("Will be registered on use")
                            .small()
                            .color(egui::Color32::GRAY),
                    );
                    false
                }
            } else {
                ui.label(
                    egui::RichText::new("Click a sprite file to select it")
                        .italics()
                        .color(egui::Color32::GRAY),
                );
                false
            };

            ui.separator();

            // Action buttons
            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    dialog.close();
                }

                let can_select = dialog.selected.is_some();
                let button_label = if selected_is_registered {
                    "Use"
                } else {
                    "Register & Use"
                };
                if ui
                    .add_enabled(can_select, egui::Button::new(button_label))
                    .clicked()
                    && let Some(selected_path) = &dialog.selected
                    && let Some(asset_path) = dialog.get_asset_path(selected_path, config)
                {
                    result = Some((asset_path, dialog.browsing_extended));
                    dialog.close();
                }
            });
        });

    result
}
