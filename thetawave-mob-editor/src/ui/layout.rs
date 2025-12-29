use bevy::{
    ecs::{message::MessageWriter, system::SystemParam},
    prelude::*,
};
use bevy_egui::{EguiContexts, egui};

use crate::{
    data::{EditorSession, SpriteRegistry},
    file::{
        append_sprite_to_assets_ron, DeleteMobEvent, FileTreeState, LoadMobEvent, NewMobEvent,
        ReloadMobEvent, SaveMobEvent,
    },
    plugin::{EditorConfig, SpriteRegistrationDialog, SpriteSelectionDialog, DecorationSelectionDialog},
    preview::{JointedMobCache, PreviewSettings, PreviewState},
    states::EditorState,
};

use super::{
    file_panel_ui, properties_panel_ui, render_delete_dialog, toolbar_ui,
    DeleteDialogState, ErrorDialog, FileDialogState, FileDialogResult,
    DecorationBrowseResult, BrowseRegistrationRequest, UnsavedChangesDialog,
    ValidationDialog, update_decoration_sprite,
};

/// Grouped message writers for the main UI system
#[derive(SystemParam)]
pub struct UiMessageWriters<'w> {
    load: MessageWriter<'w, LoadMobEvent>,
    save: MessageWriter<'w, SaveMobEvent>,
    reload: MessageWriter<'w, ReloadMobEvent>,
    new: MessageWriter<'w, NewMobEvent>,
    delete: MessageWriter<'w, DeleteMobEvent>,
    exit: MessageWriter<'w, bevy::app::AppExit>,
}

/// Grouped dialog resources for the main UI system
#[derive(SystemParam)]
pub struct DialogResources<'w> {
    pub file_dialog: ResMut<'w, FileDialogState>,
    pub delete_dialog: ResMut<'w, DeleteDialogState>,
    pub unsaved_dialog: ResMut<'w, UnsavedChangesDialog>,
    pub error_dialog: ResMut<'w, ErrorDialog>,
    pub validation_dialog: ResMut<'w, ValidationDialog>,
    pub registration_dialog: ResMut<'w, SpriteRegistrationDialog>,
    pub selection_dialog: ResMut<'w, SpriteSelectionDialog>,
    pub decoration_selection_dialog: ResMut<'w, DecorationSelectionDialog>,
}

/// Main UI layout system that renders all egui panels
pub fn main_ui_system(
    mut contexts: EguiContexts,
    mut session: ResMut<EditorSession>,
    mut file_tree: ResMut<FileTreeState>,
    state: Res<State<EditorState>>,
    config: Res<EditorConfig>,
    time: Res<Time>,
    mut events: UiMessageWriters,
    mut dialogs: DialogResources,
    mut preview_settings: ResMut<PreviewSettings>,
    mut preview_state: ResMut<PreviewState>,
    mut sprite_registry: ResMut<SpriteRegistry>,
    jointed_cache: Res<JointedMobCache>,
    mut frames_passed: Local<u8>,
) {
    // Skip first two frames to let egui initialize properly
    if *frames_passed < 2 {
        *frames_passed += 1;
        return;
    }

    // Skip during loading state
    if *state.get() == EditorState::Loading {
        return;
    }

    // Poll for file dialog results
    if let Some(result) = dialogs.file_dialog.poll_result() {
        match result {
            FileDialogResult::NewFile { path, is_patch } => {
                // Extract name from filename (without extension)
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unnamed")
                    .to_string();

                events.new.write(NewMobEvent {
                    path,
                    name,
                    is_patch,
                });
            }
            FileDialogResult::OpenFile { path } => {
                // Open the selected file
                events.load.write(LoadMobEvent { path });
            }
            FileDialogResult::Cancelled => {
                // User cancelled, nothing to do
            }
        }
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    // Top toolbar
    toolbar_ui(
        ctx,
        &mut session,
        &mut events.save,
        &mut events.reload,
        &mut *dialogs.file_dialog,
        &config,
    );

    // Left panel - File browser
    egui::SidePanel::left("file_panel")
        .default_width(250.0)
        .min_width(200.0)
        .resizable(true)
        .show(ctx, |ui| {
            file_panel_ui(
                ui,
                &mut file_tree,
                &session,
                &mut events.load,
                &mut *dialogs.file_dialog,
                &mut *dialogs.delete_dialog,
                &mut *dialogs.unsaved_dialog,
                &config,
            );
        });

    // Right panel - Properties (only when editing)
    let mut panel_result = super::PropertiesPanelResult::default();
    if *state.get() == EditorState::Editing {
        egui::SidePanel::right("properties_panel")
            .default_width(300.0)
            .min_width(250.0)
            .resizable(true)
            .show(ctx, |ui| {
                panel_result = properties_panel_ui(ui, &mut session, &sprite_registry, &jointed_cache);
            });
    }

    // Handle main sprite browse & register result
    if let Some(result) = panel_result.sprite_browse {
        handle_sprite_browse_result(
            result,
            &mut sprite_registry,
            &mut *dialogs.selection_dialog,
            &mut session,
            &time,
        );
    }

    // Handle decoration sprite browse & register result
    if let Some(result) = panel_result.decoration_browse {
        handle_decoration_browse_result(
            result,
            &mut sprite_registry,
            &mut *dialogs.decoration_selection_dialog,
            &mut session,
            &time,
        );
    }

    // Bottom status bar / log panel
    let panel_height = if session.log.expanded { 120.0 } else { 24.0 };
    egui::TopBottomPanel::bottom("status_bar")
        .exact_height(panel_height)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // File path
                if let Some(path) = &session.current_path {
                    ui.label(format!("File: {}", path.display()));
                } else {
                    ui.label("No file loaded");
                }

                ui.separator();

                // Modified indicator
                if session.is_modified {
                    ui.colored_label(egui::Color32::YELLOW, "Modified");
                } else {
                    ui.label("Saved");
                }

                // When collapsed, show last message
                if !session.log.expanded {
                    if let Some(entry) = session.log.last() {
                        ui.separator();
                        ui.colored_label(entry.level.color(), &entry.text);
                    }
                }

                // Right-aligned expand/collapse and clear buttons
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Clear button (only when there are entries)
                    if !session.log.is_empty() {
                        if ui.small_button("Clear").clicked() {
                            session.log.clear();
                        }
                    }

                    // Expand/collapse toggle
                    let toggle_text = if session.log.expanded { "Collapse" } else { "Log" };
                    if ui.small_button(toggle_text).clicked() {
                        session.log.expanded = !session.log.expanded;
                    }
                });
            });

            // When expanded, show scrollable log
            if session.log.expanded {
                ui.separator();
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for entry in session.log.entries() {
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
                                ui.label(egui::RichText::new(time_str).small().color(egui::Color32::GRAY));
                                ui.colored_label(entry.level.color(), &entry.text);
                            });
                        }
                    });
            }
        });

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
                            ui.label("Select a .mob or .mobpatch file from the file browser");
                            ui.label("or use File → New Mob/Patch to create one.");
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
                        if ui.selectable_label(preview_settings.show_grid, "Grid").clicked() {
                            preview_settings.show_grid = !preview_settings.show_grid;
                        }
                        if ui.selectable_label(preview_settings.show_colliders, "Colliders").clicked() {
                            preview_settings.show_colliders = !preview_settings.show_colliders;
                        }
                        if ui.selectable_label(preview_settings.show_jointed_mobs, "Jointed").clicked() {
                            preview_settings.show_jointed_mobs = !preview_settings.show_jointed_mobs;
                            preview_state.needs_rebuild = true;
                        }
                        if ui.selectable_label(preview_settings.show_joint_gizmos, "Joints").clicked() {
                            preview_settings.show_joint_gizmos = !preview_settings.show_joint_gizmos;
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

    // Render delete dialog window
    render_delete_dialog(ctx, &mut *dialogs.delete_dialog, &mut events.delete);

    // Render sprite registration dialog
    render_registration_dialog(
        ctx,
        &mut *dialogs.registration_dialog,
        &mut *sprite_registry,
        &mut events.save,
        &mut session,
        &time,
    );

    // Render sprite selection confirmation dialog
    render_selection_dialog(
        ctx,
        &mut *dialogs.selection_dialog,
        &mut session,
        &time,
    );

    // Render decoration sprite selection confirmation dialog
    render_decoration_selection_dialog(
        ctx,
        &mut *dialogs.decoration_selection_dialog,
        &mut session,
        &time,
    );

    // Render unsaved changes dialog
    render_unsaved_changes_dialog(
        ctx,
        &mut *dialogs.unsaved_dialog,
        &session,
        &mut events.save,
        &mut events.load,
        &mut events.exit,
    );

    // Render error dialog
    render_error_dialog(ctx, &mut *dialogs.error_dialog);

    // Render validation dialog
    render_validation_dialog(ctx, &mut *dialogs.validation_dialog);
}

/// Render the validation errors dialog
fn render_validation_dialog(ctx: &egui::Context, dialog: &mut ValidationDialog) {
    if !dialog.is_open {
        return;
    }

    egui::Window::new("Validation Errors")
        .collapsible(false)
        .resizable(true)
        .default_width(400.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label("The following issues were found:");
            ui.add_space(8.0);

            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for error in &dialog.errors {
                        ui.horizontal(|ui| {
                            ui.colored_label(egui::Color32::RED, ">");
                            ui.label(egui::RichText::new(&error.field_path).monospace().strong());
                            ui.label(&error.message);
                        });
                    }
                });

            ui.add_space(12.0);
            ui.label(egui::RichText::new("Please fix these issues before saving.").small().color(egui::Color32::GRAY));

            ui.add_space(8.0);
            if ui.button("OK").clicked() {
                dialog.is_open = false;
                dialog.errors.clear();
            }
        });
}

/// Render the unsaved changes dialog
fn render_unsaved_changes_dialog(
    ctx: &egui::Context,
    dialog: &mut UnsavedChangesDialog,
    session: &EditorSession,
    save_events: &mut MessageWriter<SaveMobEvent>,
    load_events: &mut MessageWriter<LoadMobEvent>,
    exit_events: &mut MessageWriter<bevy::app::AppExit>,
) {
    use super::UnsavedAction;

    if !dialog.is_open {
        return;
    }

    let mut action_to_take: Option<UnsavedAction> = None;
    let mut close_dialog = false;

    // Determine dialog message based on action
    let is_exit = matches!(&dialog.pending_action, Some(UnsavedAction::Exit));
    let title = if is_exit { "Exit with Unsaved Changes?" } else { "Unsaved Changes" };

    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label("You have unsaved changes. What would you like to do?");

            if let Some(path) = &session.current_path {
                ui.add_space(4.0);
                ui.label(egui::RichText::new(format!("File: {}", path.display())).small().color(egui::Color32::GRAY));
            }

            ui.add_space(12.0);

            ui.horizontal(|ui| {
                let continue_text = if is_exit { "Save & Exit" } else { "Save & Continue" };
                if ui.button(continue_text).clicked() {
                    // Save first, then perform action
                    save_events.write(crate::file::SaveMobEvent { path: None });
                    action_to_take = dialog.pending_action.clone();
                    close_dialog = true;
                }

                let discard_text = if is_exit { "Exit Without Saving" } else { "Discard Changes" };
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

/// Render the error dialog
fn render_error_dialog(ctx: &egui::Context, dialog: &mut ErrorDialog) {
    if !dialog.is_open {
        return;
    }

    egui::Window::new(&dialog.title)
        .collapsible(false)
        .resizable(true)
        .default_width(400.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::RED, "Error:");
                ui.label(&dialog.message);
            });

            if let Some(details) = &dialog.details {
                ui.add_space(8.0);
                egui::CollapsingHeader::new("Details")
                    .default_open(false)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(200.0)
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut details.as_str())
                                        .font(egui::TextStyle::Monospace)
                                        .desired_width(f32::INFINITY)
                                );
                            });
                    });
            }

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button("OK").clicked() {
                    dialog.close();
                }
            });
        });
}

/// Render the sprite registration dialog
fn render_registration_dialog(
    ctx: &egui::Context,
    dialog: &mut SpriteRegistrationDialog,
    sprite_registry: &mut SpriteRegistry,
    save_events: &mut MessageWriter<SaveMobEvent>,
    session: &mut EditorSession,
    time: &Time,
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
                    let cwd = std::env::current_dir().unwrap_or_default();

                    for sprite in &dialog.unregistered_sprites {
                        let is_extended = sprite.starts_with("extended://");
                        let clean_path = sprite.strip_prefix("extended://").unwrap_or(sprite);

                        let assets_ron = if is_extended {
                            cwd.join("thetawave-test-game/assets/game.assets.ron")
                        } else {
                            cwd.join("assets/game.assets.ron")
                        };

                        if let Err(e) = append_sprite_to_assets_ron(&assets_ron, clean_path, is_extended)
                        {
                            session.log_error(format!("Error registering sprite: {}", e), time);
                        }
                    }

                    // Mark registry for refresh
                    sprite_registry.needs_refresh = true;

                    // Proceed with save
                    if let Some(path) = dialog.pending_save_path.take() {
                        save_events.write(SaveMobEvent { path: Some(path) });
                    }

                    dialog.show = false;
                    dialog.unregistered_sprites.clear();
                }

                if ui.button("Save Without Registering").clicked() {
                    // Proceed with save anyway
                    if let Some(path) = dialog.pending_save_path.take() {
                        save_events.write(SaveMobEvent { path: Some(path) });
                    }

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
) {
    if !dialog.show {
        return;
    }

    egui::Window::new("Set Sprite?")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label("Sprite registered successfully!");
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("Sprite:");
                ui.label(egui::RichText::new(&dialog.display_name).monospace().strong());
            });

            ui.add_space(8.0);
            ui.label("Would you like to set this as the sprite for the current mob?");
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui.button("Yes, set sprite").clicked() {
                    // Set the sprite path in the mob
                    let success = if let Some(mob) = session
                        .current_mob
                        .as_mut()
                        .and_then(|v| v.as_table_mut())
                    {
                        mob.insert(
                            "sprite".to_string(),
                            toml::Value::String(dialog.mob_path.clone()),
                        );
                        session.check_modified();

                        // Also update merged preview for patches
                        if let (Some(base), Some(patch)) = (&session.base_mob, &session.current_mob) {
                            let mut merged = base.clone();
                            crate::file::merge_toml_values(&mut merged, patch.clone());
                            session.merged_for_preview = Some(merged);
                        }

                        true
                    } else {
                        false
                    };

                    if success {
                        session.log_success(
                            format!("Sprite set to: {}", dialog.display_name),
                            time,
                        );
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
                    session.log_success(
                        format!("Sprite registered: {}", dialog.display_name),
                        time,
                    );

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
            ui.label(egui::RichText::new("(none)").italics().color(egui::Color32::GRAY));
        }
    });

    // Show load status
    ui.horizontal(|ui| {
        ui.label("Status:");
        if sprite_info.loaded_from.is_some() {
            ui.label(egui::RichText::new("Loaded").color(egui::Color32::GREEN));
            if preview_state.sprite_override_path.is_some() {
                ui.label(egui::RichText::new("(override)").small().color(egui::Color32::YELLOW));
            }
        } else if let Some(error) = &sprite_info.error {
            ui.label(egui::RichText::new(error).color(egui::Color32::RED));
        }
    });

    // Show source path
    if let Some(path) = &sprite_info.loaded_from {
        ui.horizontal(|ui| {
            ui.label("Source:");
            ui.label(egui::RichText::new(path.display().to_string()).small().monospace());
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
                ui.label(egui::RichText::new(path.display().to_string()).small().monospace().color(color));
            }
        });
    }

    ui.separator();

    // Sprite override section
    ui.horizontal(|ui| {
        ui.label("Override:");

        if let Some(override_path) = &preview_state.sprite_override_path {
            // Show current override path
            let path_str = override_path.display().to_string();
            let short_path = if path_str.len() > 40 {
                format!("...{}", &path_str[path_str.len() - 37..])
            } else {
                path_str
            };
            ui.label(egui::RichText::new(short_path).small().monospace().color(egui::Color32::YELLOW));

            // Clear button
            if ui.small_button("Clear").clicked() {
                preview_state.sprite_override_path = None;
                preview_state.needs_rebuild = true;
            }
        } else {
            ui.label(egui::RichText::new("(none)").italics().color(egui::Color32::GRAY));
        }
    });

    // Browse button
    if ui.button("Browse for sprite...").clicked() {
        // Open file dialog for sprite selection
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Aseprite Files", &["aseprite", "ase"])
            .add_filter("All Files", &["*"])
            .set_title("Select Sprite File")
            .pick_file()
        {
            preview_state.sprite_override_path = Some(path);
            preview_state.needs_rebuild = true;
        }
    }

    ui.label(
        egui::RichText::new("Use this to preview a sprite file that's not in the game's asset directories.")
            .small()
            .color(egui::Color32::GRAY),
    );
}

/// Handle a browse result for main sprite registration
fn handle_sprite_browse_result(
    result: super::BrowseResult,
    sprite_registry: &mut SpriteRegistry,
    selection_dialog: &mut SpriteSelectionDialog,
    session: &mut EditorSession,
    time: &Time,
) {
    match result {
        super::BrowseResult::Register(request) => {
            register_and_show_dialog(
                &request,
                sprite_registry,
                session,
                time,
                |display_name, mob_path| {
                    selection_dialog.show = true;
                    selection_dialog.asset_path = request.asset_path.clone();
                    selection_dialog.mob_path = mob_path;
                    selection_dialog.display_name = display_name;
                },
            );
        }
        super::BrowseResult::InvalidLocation(error_msg) => {
            session.log_error(error_msg, time);
        }
    }
}

/// Handle a browse result for decoration sprite registration
fn handle_decoration_browse_result(
    result: DecorationBrowseResult,
    sprite_registry: &mut SpriteRegistry,
    decoration_dialog: &mut DecorationSelectionDialog,
    session: &mut EditorSession,
    time: &Time,
) {
    match result {
        DecorationBrowseResult::Register { index, request } => {
            register_and_show_dialog(
                &request,
                sprite_registry,
                session,
                time,
                |display_name, mob_path| {
                    decoration_dialog.show = true;
                    decoration_dialog.decoration_index = index;
                    decoration_dialog.mob_path = mob_path;
                    decoration_dialog.display_name = display_name;
                },
            );
        }
        DecorationBrowseResult::InvalidLocation(message) => {
            session.log_error(message, time);
        }
    }
}

/// Register a sprite and call the success callback
fn register_and_show_dialog<F>(
    request: &BrowseRegistrationRequest,
    sprite_registry: &mut SpriteRegistry,
    session: &mut EditorSession,
    time: &Time,
    on_success: F,
) where
    F: FnOnce(String, String),
{
    let cwd = std::env::current_dir().unwrap_or_default();
    let assets_ron = if request.is_extended {
        cwd.join("thetawave-test-game/assets/game.assets.ron")
    } else {
        cwd.join("assets/game.assets.ron")
    };

    // Register the sprite
    match append_sprite_to_assets_ron(&assets_ron, &request.asset_path, request.is_extended) {
        Ok(()) => {
            // Mark registry for refresh
            sprite_registry.needs_refresh = true;

            // Extract display name from asset path
            let display_name = std::path::Path::new(&request.asset_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(&request.asset_path)
                .to_string();

            session.log_success(format!("Registered: {}", request.asset_path), time);

            // Call the success callback
            on_success(display_name, request.mob_path.clone());
        }
        Err(e) => {
            session.log_error(format!("Failed to register sprite: {}", e), time);
        }
    }
}

/// Render the decoration sprite selection confirmation dialog
fn render_decoration_selection_dialog(
    ctx: &egui::Context,
    dialog: &mut DecorationSelectionDialog,
    session: &mut EditorSession,
    time: &Time,
) {
    if !dialog.show {
        return;
    }

    egui::Window::new("Set Decoration Sprite?")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label("Sprite registered successfully!");
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("Sprite:");
                ui.label(egui::RichText::new(&dialog.display_name).monospace().strong());
            });

            ui.horizontal(|ui| {
                ui.label("Decoration:");
                ui.label(format!("#{}", dialog.decoration_index + 1));
            });

            ui.add_space(8.0);
            ui.label("Would you like to set this as the sprite for this decoration?");
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui.button("Yes, set sprite").clicked() {
                    // Set the sprite path in the decoration
                    update_decoration_sprite(session, dialog.decoration_index, &dialog.mob_path);
                    session.check_modified();

                    // Also update merged preview for patches
                    if let (Some(base), Some(patch)) = (&session.base_mob, &session.current_mob) {
                        let mut merged = base.clone();
                        crate::file::merge_toml_values(&mut merged, patch.clone());
                        session.merged_for_preview = Some(merged);
                    }

                    session.log_success(
                        format!("Decoration {} sprite set to: {}", dialog.decoration_index + 1, dialog.display_name),
                        time,
                    );

                    dialog.show = false;
                    dialog.mob_path.clear();
                    dialog.display_name.clear();
                }

                if ui.button("No, just register").clicked() {
                    session.log_success(
                        format!("Sprite registered: {}", dialog.display_name),
                        time,
                    );

                    dialog.show = false;
                    dialog.mob_path.clear();
                    dialog.display_name.clear();
                }
            });
        });
}
