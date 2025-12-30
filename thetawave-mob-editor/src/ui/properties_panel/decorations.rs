//! Decoration editing functionality for the properties panel.
//!
//! This module handles rendering and editing of mob decorations (visual
//! elements attached to the mob sprite).

use bevy_egui::egui;

use crate::data::{EditorSession, SpriteRegistry, SpriteSource};
use crate::plugin::EditorConfig;

use super::fields::{render_patch_indicator, ICON_BUTTON_MIN_SIZE, INDENT_SPACING, INHERITED_COLOR, PATCHED_COLOR};

/// Render a sprite picker dropdown.
///
/// Returns true if the sprite browser should be opened.
pub fn render_sprite_picker(
    ui: &mut egui::Ui,
    display_table: &toml::value::Table,
    patch_table: &toml::value::Table,
    session: &mut EditorSession,
    sprite_registry: &SpriteRegistry,
    is_patch: bool,
    modified: &mut bool,
    config: &EditorConfig,
) -> bool {
    let mut open_browser = false;

    let is_patched = is_patch && patch_table.contains_key("sprite");
    let current_sprite = display_table
        .get("sprite")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Normalize for comparison (strip extended:// prefix)
    let normalized_current = current_sprite
        .strip_prefix("extended://")
        .unwrap_or(current_sprite);

    ui.horizontal(|ui| {
        render_patch_indicator(ui, is_patched, is_patch);

        let text_color = if is_patch && !is_patched {
            INHERITED_COLOR
        } else {
            ui.style().visuals.text_color()
        };

        ui.label(egui::RichText::new("Sprite:").color(text_color));

        // Determine if current sprite is registered
        let is_registered = sprite_registry.is_registered(current_sprite);
        let display_text = if is_registered {
            sprite_registry.display_name_for(current_sprite)
        } else if current_sprite.is_empty() {
            "(none)".to_string()
        } else {
            format!("{} ⚠", sprite_registry.display_name_for(current_sprite))
        };

        let mut selected_path = current_sprite.to_string();

        egui::ComboBox::from_id_salt("sprite_picker")
            .selected_text(&display_text)
            .width(160.0)
            .show_ui(ui, |ui| {
                // Option for no sprite
                if ui
                    .selectable_label(current_sprite.is_empty(), "(none)")
                    .clicked()
                {
                    selected_path = String::new();
                }

                ui.separator();

                // Base sprites section
                let base_sprites: Vec<_> = sprite_registry.base_sprites().collect();
                if !base_sprites.is_empty() {
                    ui.label(
                        egui::RichText::new("Base Sprites")
                            .small()
                            .color(egui::Color32::GRAY),
                    );
                    for sprite in base_sprites {
                        let is_selected = normalized_current == sprite.asset_path;
                        if ui
                            .selectable_label(is_selected, &sprite.display_name)
                            .clicked()
                        {
                            selected_path = sprite.mob_path();
                        }
                    }
                }

                // Extended sprites section (only for extended mobs or mobpatches)
                if session.can_use_extended_sprites(config) {
                    let extended_sprites: Vec<_> = sprite_registry.extended_sprites().collect();
                    if !extended_sprites.is_empty() {
                        ui.separator();
                        ui.label(
                            egui::RichText::new("Extended Sprites")
                                .small()
                                .color(PATCHED_COLOR),
                        );
                        for sprite in extended_sprites {
                            let is_selected = normalized_current == sprite.asset_path;
                            if ui
                                .selectable_label(is_selected, &sprite.display_name)
                                .clicked()
                            {
                                // For patches, use extended:// prefix for extended sprites
                                selected_path =
                                    if is_patch && sprite.source == SpriteSource::Extended {
                                        sprite.mobpatch_path()
                                    } else {
                                        sprite.mob_path()
                                    };
                            }
                        }
                    }
                }

                // Show current unregistered sprite at bottom if applicable
                if !is_registered && !current_sprite.is_empty() {
                    ui.separator();
                    ui.label(
                        egui::RichText::new("Current (Unregistered)")
                            .small()
                            .color(egui::Color32::YELLOW),
                    );
                    let _ = ui.selectable_label(true, &sprite_registry.display_name_for(current_sprite));
                }
            });

        // Apply change if different
        if selected_path != current_sprite {
            if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                if selected_path.is_empty() {
                    mob.remove("sprite");
                } else {
                    mob.insert("sprite".to_string(), toml::Value::String(selected_path));
                }
                *modified = true;
            }
        }

        // Reset button for patches
        if render_reset_button(ui, is_patched, is_patch) {
            if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                mob.remove("sprite");
                *modified = true;
            }
        }
    });

    // Browse & Register button row
    ui.horizontal(|ui| {
        ui.add_space(INDENT_SPACING);

        if ui
            .small_button("➕ Register New Sprite...")
            .on_hover_text("Find an aseprite file not yet in the sprite list and add it to game.assets.ron")
            .clicked()
        {
            open_browser = true;
        }

        ui.label(
            egui::RichText::new("Add unregistered aseprite to game")
                .small()
                .color(egui::Color32::GRAY),
        );
    });

    // Show warning if unregistered
    if !sprite_registry.is_registered(current_sprite) && !current_sprite.is_empty() {
        ui.horizontal(|ui| {
            ui.add_space(INDENT_SPACING);
            ui.label(
                egui::RichText::new("⚠ Not in game.assets.ron")
                    .small()
                    .color(egui::Color32::YELLOW),
            );
        });
    }

    open_browser
}

/// Render the decorations section with sprite pickers.
///
/// Returns Some(decoration_index) if the sprite browser should be opened for a decoration.
pub fn render_decorations_section(
    ui: &mut egui::Ui,
    display_table: &toml::value::Table,
    patch_table: &toml::value::Table,
    session: &mut EditorSession,
    sprite_registry: &SpriteRegistry,
    is_patch: bool,
    modified: &mut bool,
    config: &EditorConfig,
) -> Option<usize> {
    let mut open_decoration_browser: Option<usize> = None;
    let is_patched = is_patch && patch_table.contains_key("decorations");
    // Only allow editing if not a patch OR if decorations are in the patch
    let can_edit = !is_patch || is_patched;

    egui::CollapsingHeader::new("Decorations")
        .default_open(false)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                render_patch_indicator(ui, is_patched, is_patch);
                if is_patch && !is_patched {
                    ui.label(
                        egui::RichText::new("(inherited from base)")
                            .small()
                            .color(INHERITED_COLOR),
                    );
                    // Add "Override" button to copy decorations to patch
                    if ui.button("Override").clicked() {
                        if let Some(decorations) = display_table.get("decorations").cloned() {
                            if let Some(mob) =
                                session.current_mob.as_mut().and_then(|v| v.as_table_mut())
                            {
                                mob.insert("decorations".to_string(), decorations);
                                *modified = true;
                            }
                        }
                    }
                } else if is_patch && is_patched {
                    ui.label(
                        egui::RichText::new("(overriding base)")
                            .small()
                            .color(PATCHED_COLOR),
                    );
                    // Add "Reset" button to remove decorations from patch
                    if ui.button("Reset to base").clicked() {
                        if let Some(mob) =
                            session.current_mob.as_mut().and_then(|v| v.as_table_mut())
                        {
                            mob.remove("decorations");
                            *modified = true;
                        }
                    }
                }
            });

            // Get decorations array
            let decorations = display_table
                .get("decorations")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();

            if decorations.is_empty() {
                ui.label(
                    egui::RichText::new("No decorations")
                        .small()
                        .color(egui::Color32::GRAY),
                );
            }

            let mut delete_index: Option<usize> = None;

            for (i, decoration) in decorations.iter().enumerate() {
                let Some(arr) = decoration.as_array() else {
                    continue;
                };
                if arr.len() < 2 {
                    continue;
                }

                let sprite_path = arr[0].as_str().unwrap_or("");
                let position = if let Some(pos_arr) = arr[1].as_array() {
                    let x = pos_arr
                        .first()
                        .and_then(|v| v.as_float())
                        .unwrap_or(0.0) as f32;
                    let y = pos_arr.get(1).and_then(|v| v.as_float()).unwrap_or(0.0) as f32;
                    (x, y)
                } else {
                    (0.0, 0.0)
                };

                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(format!("#{}", i + 1));
                        if can_edit {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui
                                    .button(egui::RichText::new("🗑").color(egui::Color32::RED))
                                    .on_hover_text("Delete decoration")
                                    .clicked()
                                {
                                    delete_index = Some(i);
                                }
                            });
                        }
                    });

                    // Sprite picker for this decoration
                    let open_browser_for = render_decoration_sprite_picker(
                        ui,
                        i,
                        sprite_path,
                        sprite_registry,
                        session,
                        is_patch,
                        can_edit,
                        modified,
                        config,
                    );
                    if let Some(idx) = open_browser_for {
                        open_decoration_browser = Some(idx);
                    }

                    // Position editors
                    ui.horizontal(|ui| {
                        ui.label("Position:");
                        let mut x = position.0;
                        let mut y = position.1;

                        if can_edit {
                            let x_changed = ui
                                .add(
                                    egui::DragValue::new(&mut x)
                                        .prefix("X: ")
                                        .range(-500.0..=500.0)
                                        .speed(0.5),
                                )
                                .changed();
                            let y_changed = ui
                                .add(
                                    egui::DragValue::new(&mut y)
                                        .prefix("Y: ")
                                        .range(-500.0..=500.0)
                                        .speed(0.5),
                                )
                                .changed();

                            if x_changed || y_changed {
                                update_decoration_position(session, i, x, y);
                                *modified = true;
                            }
                        } else {
                            ui.label(format!("X: {:.1}  Y: {:.1}", x, y));
                        }
                    });
                });
            }

            // Handle deletion
            if let Some(idx) = delete_index {
                delete_decoration(session, idx);
                *modified = true;
            }

            if can_edit {
                ui.separator();

                // Add new decoration button
                if ui.button("+ Add Decoration").clicked() {
                    add_new_decoration(session, sprite_registry);
                    *modified = true;
                }
            }
        });

    open_decoration_browser
}

/// Render sprite picker for a decoration.
///
/// Returns Some(decoration_index) if the sprite browser should be opened.
fn render_decoration_sprite_picker(
    ui: &mut egui::Ui,
    index: usize,
    current_sprite: &str,
    sprite_registry: &SpriteRegistry,
    session: &mut EditorSession,
    is_patch: bool,
    can_edit: bool,
    modified: &mut bool,
    config: &EditorConfig,
) -> Option<usize> {
    let mut open_browser_for: Option<usize> = None;

    // Normalize for comparison (strip extended:// prefix)
    let normalized_current = current_sprite
        .strip_prefix("extended://")
        .unwrap_or(current_sprite);

    ui.horizontal(|ui| {
        ui.label("Sprite:");

        // Determine if current sprite is registered
        let is_registered = sprite_registry.is_registered(current_sprite);
        let display_text = if is_registered {
            sprite_registry.display_name_for(current_sprite)
        } else if current_sprite.is_empty() {
            "(none)".to_string()
        } else {
            format!("{} ⚠", sprite_registry.display_name_for(current_sprite))
        };

        if can_edit {
            let mut selected_path = current_sprite.to_string();

            egui::ComboBox::from_id_salt(format!("decoration_sprite_{}", index))
                .selected_text(&display_text)
                .width(140.0)
                .show_ui(ui, |ui| {
                    // Option for no sprite
                    if ui
                        .selectable_label(current_sprite.is_empty(), "(none)")
                        .clicked()
                    {
                        selected_path = String::new();
                    }

                    ui.separator();

                    // Base sprites section
                    let base_sprites: Vec<_> = sprite_registry.base_sprites().collect();
                    if !base_sprites.is_empty() {
                        ui.label(
                            egui::RichText::new("Base Sprites")
                                .small()
                                .color(egui::Color32::GRAY),
                        );
                        for sprite in base_sprites {
                            let is_selected = normalized_current == sprite.asset_path;
                            if ui
                                .selectable_label(is_selected, &sprite.display_name)
                                .clicked()
                            {
                                selected_path = sprite.mob_path();
                            }
                        }
                    }

                    // Extended sprites section
                    if session.can_use_extended_sprites(config) {
                        let extended_sprites: Vec<_> = sprite_registry.extended_sprites().collect();
                        if !extended_sprites.is_empty() {
                            ui.separator();
                            ui.label(
                                egui::RichText::new("Extended Sprites")
                                    .small()
                                    .color(PATCHED_COLOR),
                            );
                            for sprite in extended_sprites {
                                let is_selected = normalized_current == sprite.asset_path;
                                if ui
                                    .selectable_label(is_selected, &sprite.display_name)
                                    .clicked()
                                {
                                    selected_path =
                                        if is_patch && sprite.source == SpriteSource::Extended {
                                            sprite.mobpatch_path()
                                        } else {
                                            sprite.mob_path()
                                        };
                                }
                            }
                        }
                    }

                    // Show current unregistered sprite
                    if !is_registered && !current_sprite.is_empty() {
                        ui.separator();
                        ui.label(
                            egui::RichText::new("Current (Unregistered)")
                                .small()
                                .color(egui::Color32::YELLOW),
                        );
                        let _ = ui.selectable_label(
                            true,
                            &sprite_registry.display_name_for(current_sprite),
                        );
                    }
                });

            // Apply change if different
            if selected_path != current_sprite {
                update_decoration_sprite(session, index, &selected_path);
                *modified = true;
            }

            // Register new sprite button
            if ui
                .small_button("➕")
                .on_hover_text("Register new sprite for this decoration")
                .clicked()
            {
                open_browser_for = Some(index);
            }
        } else {
            ui.label(&display_text);
        }
    });

    open_browser_for
}

// =============================================================================
// Helper functions for decoration manipulation
// =============================================================================

/// Helper to render reset button for patches.
fn render_reset_button(ui: &mut egui::Ui, is_patched: bool, is_patch_file: bool) -> bool {
    if is_patch_file && is_patched {
        let response = ui.add(
            egui::Button::new(egui::RichText::new("×").color(egui::Color32::WHITE))
                .fill(egui::Color32::from_rgb(120, 60, 60))
                .min_size(ICON_BUTTON_MIN_SIZE),
        );
        if response
            .on_hover_text("Remove from patch (use base value)")
            .clicked()
        {
            return true;
        }
    }
    false
}

/// Update a decoration's sprite path.
fn update_decoration_sprite(session: &mut EditorSession, index: usize, sprite_path: &str) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        if let Some(decorations) = mob.get_mut("decorations").and_then(|v| v.as_array_mut()) {
            if let Some(decoration) = decorations.get_mut(index).and_then(|v| v.as_array_mut()) {
                if !decoration.is_empty() {
                    decoration[0] = toml::Value::String(sprite_path.to_string());
                }
            }
        }
    }
}

/// Update a decoration's position.
fn update_decoration_position(session: &mut EditorSession, index: usize, x: f32, y: f32) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        if let Some(decorations) = mob.get_mut("decorations").and_then(|v| v.as_array_mut()) {
            if let Some(decoration) = decorations.get_mut(index).and_then(|v| v.as_array_mut()) {
                if decoration.len() >= 2 {
                    decoration[1] = toml::Value::Array(vec![
                        toml::Value::Float(x as f64),
                        toml::Value::Float(y as f64),
                    ]);
                }
            }
        }
    }
}

/// Delete a decoration by index.
fn delete_decoration(session: &mut EditorSession, index: usize) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        if let Some(decorations) = mob.get_mut("decorations").and_then(|v| v.as_array_mut()) {
            if index < decorations.len() {
                decorations.remove(index);
            }
            // Clean up empty array
            if decorations.is_empty() {
                mob.remove("decorations");
            }
        }
    }
}

/// Add a new decoration with defaults.
fn add_new_decoration(session: &mut EditorSession, sprite_registry: &SpriteRegistry) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        // Ensure decorations array exists
        if !mob.contains_key("decorations") {
            mob.insert("decorations".to_string(), toml::Value::Array(vec![]));
        }

        if let Some(decorations) = mob.get_mut("decorations").and_then(|v| v.as_array_mut()) {
            // Pick a default sprite (first base sprite, or empty)
            let default_sprite = sprite_registry
                .base_sprites()
                .next()
                .map(|s| s.mob_path())
                .unwrap_or_default();

            // Decoration format: [sprite_path, [x, y]]
            let decoration = toml::Value::Array(vec![
                toml::Value::String(default_sprite),
                toml::Value::Array(vec![toml::Value::Float(0.0), toml::Value::Float(0.0)]),
            ]);

            decorations.push(decoration);
        }
    }
}
