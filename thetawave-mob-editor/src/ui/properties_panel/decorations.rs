//! Decoration editing functionality for the properties panel.
//!
//! This module handles rendering and editing of mob decorations (visual
//! elements attached to the mob sprite).

use bevy_egui::egui;

use crate::data::{EditorSession, SpriteRegistry};
use crate::plugin::EditorConfig;

use super::fields::{
    INDENT_SPACING, INHERITED_COLOR, PATCHED_COLOR, header_color, render_patch_indicator,
    render_reset_button,
};
use super::update_decoration_sprite;

/// Render a sprite picker dropdown
///
/// Returns true if the sprite browser should be opened
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
    let is_modified = session.is_field_modified("sprite");
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

        // Use modification color (yellow) if modified, otherwise patch-aware color
        let text_color = if is_modified {
            super::fields::MODIFIED_COLOR
        } else if is_patch && !is_patched {
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
                // Only selected if current sprite has NO extended:// prefix
                let current_is_extended = current_sprite.starts_with("extended://");
                let base_sprites: Vec<_> = sprite_registry.base_sprites().collect();
                if !base_sprites.is_empty() {
                    ui.label(
                        egui::RichText::new("Base Sprites")
                            .small()
                            .color(egui::Color32::GRAY),
                    );
                    for sprite in base_sprites {
                        let is_selected =
                            !current_is_extended && normalized_current == sprite.asset_path;
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
                        // Extended sprites need extended:// prefix for patches AND extended mobs
                        let needs_extended_prefix =
                            is_patch || session.is_extended_mob(config);
                        for sprite in extended_sprites {
                            // Only selected if current sprite HAS extended:// prefix
                            let is_selected =
                                current_is_extended && normalized_current == sprite.asset_path;
                            if ui
                                .selectable_label(is_selected, &sprite.display_name)
                                .clicked()
                            {
                                selected_path = if needs_extended_prefix {
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
                    let _ =
                        ui.selectable_label(true, sprite_registry.display_name_for(current_sprite));
                }
            });

        // Apply change if different from current displayed value
        if selected_path != current_sprite
            && let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut())
        {
            if selected_path.is_empty() {
                // User selected "(none)" - remove the field
                mob.remove("sprite");
            } else {
                // Set the new sprite value
                // The is_field_modified comparison will correctly detect if it equals original
                mob.insert("sprite".to_string(), toml::Value::String(selected_path.clone()));
            }
            *modified = true;
        }

        // Reset button for patches
        if render_reset_button(ui, is_patched, is_patch)
            && let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut())
        {
            mob.remove("sprite");
            *modified = true;
        }
    });

    // Browse & Register button row
    ui.horizontal(|ui| {
        ui.add_space(INDENT_SPACING);

        if ui
            .small_button("Browse...")
            .on_hover_text(
                "Browse for an aseprite file and register it in game.assets.ron",
            )
            .clicked()
        {
            open_browser = true;
        }
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

/// Render the decorations section with sprite pickers
///
/// Returns Some(decoration_index) if the sprite browser should be opened for a decoration
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
    let section_modified = session.is_field_modified("decorations");
    // Only allow editing if not a patch OR if decorations are in the patch
    let can_edit = !is_patch || is_patched;

    let header_text =
        egui::RichText::new("Decorations").color(header_color(ui, section_modified));
    egui::CollapsingHeader::new(header_text)
        .default_open(false)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                render_patch_indicator(ui, is_patched, is_patch);
                if is_patch && !is_patched {
                    let has_base_decorations = display_table.contains_key("decorations");
                    ui.label(
                        egui::RichText::new(if has_base_decorations {
                            "(inherited from base)"
                        } else {
                            "(no decorations in base)"
                        })
                        .small()
                        .color(INHERITED_COLOR),
                    );
                    // Add "Override"/"Add" button to create decorations in patch
                    let button_label = if has_base_decorations {
                        "Override"
                    } else {
                        "Add Decorations"
                    };
                    if ui.button(button_label).clicked()
                        && let Some(mob) =
                            session.current_mob.as_mut().and_then(|v| v.as_table_mut())
                    {
                        // Copy from base if available, otherwise create empty array
                        let decorations = display_table
                            .get("decorations")
                            .cloned()
                            .unwrap_or_else(|| toml::Value::Array(vec![]));
                        mob.insert("decorations".to_string(), decorations);
                        *modified = true;
                    }
                } else if is_patch && is_patched {
                    ui.label(
                        egui::RichText::new("(overriding base)")
                            .small()
                            .color(PATCHED_COLOR),
                    );
                    // Add "Reset" button to remove decorations from patch
                    if ui.button("Reset to base").clicked()
                        && let Some(mob) =
                            session.current_mob.as_mut().and_then(|v| v.as_table_mut())
                    {
                        mob.remove("decorations");
                        *modified = true;
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
                    let x = pos_arr.first().and_then(|v| v.as_float()).unwrap_or(0.0) as f32;
                    let y = pos_arr.get(1).and_then(|v| v.as_float()).unwrap_or(0.0) as f32;
                    (x, y)
                } else {
                    (0.0, 0.0)
                };

                let item_modified = session.is_array_item_modified("decorations", i);

                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("#{}", i + 1))
                                .color(header_color(ui, item_modified)),
                        );
                        if can_edit {
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .button(egui::RichText::new("🗑").color(egui::Color32::RED))
                                        .on_hover_text("Delete decoration")
                                        .clicked()
                                    {
                                        delete_index = Some(i);
                                    }
                                },
                            );
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

/// Render sprite picker for a decoration
///
/// Returns Some(decoration_index) if the sprite browser should be opened
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
                    // Only selected if current sprite has NO extended:// prefix
                    let current_is_extended = current_sprite.starts_with("extended://");
                    let base_sprites: Vec<_> = sprite_registry.base_sprites().collect();
                    if !base_sprites.is_empty() {
                        ui.label(
                            egui::RichText::new("Base Sprites")
                                .small()
                                .color(egui::Color32::GRAY),
                        );
                        for sprite in base_sprites {
                            let is_selected =
                                !current_is_extended && normalized_current == sprite.asset_path;
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
                            // Extended sprites need extended:// prefix for patches AND extended mobs
                            let needs_extended_prefix =
                                is_patch || session.is_extended_mob(config);
                            for sprite in extended_sprites {
                                // Only selected if current sprite HAS extended:// prefix
                                let is_selected =
                                    current_is_extended && normalized_current == sprite.asset_path;
                                if ui
                                    .selectable_label(is_selected, &sprite.display_name)
                                    .clicked()
                                {
                                    selected_path = if needs_extended_prefix {
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
                            sprite_registry.display_name_for(current_sprite),
                        );
                    }
                });

            // Apply change if different
            if selected_path != current_sprite {
                update_decoration_sprite(session, index, &selected_path);
                *modified = true;
            }

            // Browse for sprite button
            if ui
                .small_button("📁")
                .on_hover_text("Browse for sprite")
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

/// Update a decoration's position
fn update_decoration_position(session: &mut EditorSession, index: usize, x: f32, y: f32) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut())
        && let Some(decorations) = mob.get_mut("decorations").and_then(|v| v.as_array_mut())
        && let Some(decoration) = decorations.get_mut(index).and_then(|v| v.as_array_mut())
        && decoration.len() >= 2
    {
        decoration[1] = toml::Value::Array(vec![
            toml::Value::Float(x as f64),
            toml::Value::Float(y as f64),
        ]);
    }
}

/// Delete a decoration by index
fn delete_decoration(session: &mut EditorSession, index: usize) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut())
        && let Some(decorations) = mob.get_mut("decorations").and_then(|v| v.as_array_mut())
    {
        if index < decorations.len() {
            decorations.remove(index);
        }
        // Clean up empty array
        if decorations.is_empty() {
            mob.remove("decorations");
        }
    }
}

/// Add a new decoration with defaults
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
