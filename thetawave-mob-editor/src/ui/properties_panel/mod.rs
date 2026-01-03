//! Properties panel module for the mob editor.
//!
//! This module provides the main properties panel UI and re-exports all
//! submodules for editing different aspects of mob data:
//!
//! - [`fields`] - Reusable field rendering helpers with patch awareness
//! - [`colliders`] - Collider shape and position editing
//! - [`spawners`] - Projectile and mob spawner configuration
//! - [`jointed`] - Jointed mob relationships and chains
//! - [`decorations`] - Sprite and decoration editing
//! - [`behavior`] - Behavior tree editing

pub mod behavior;
pub mod colliders;
pub mod decorations;
pub mod fields;
pub mod jointed;
pub mod spawners;

use bevy::ecs::message::MessageWriter;
use bevy_egui::egui;

use super::truncate_filename;

/// Maximum length for displayed filenames in the properties panel header
const MAX_FILENAME_DISPLAY_LEN: usize = 24;

use crate::data::{EditorSession, FileType, MobAssetRegistry, SpriteRegistry};
use crate::file::{FileTreeState, ReloadMobEvent, SaveMobEvent};
use crate::plugin::EditorConfig;

use fields::FieldResult;

/// Result from rendering the properties panel
#[derive(Default)]
pub struct PropertiesPanelResult {
    /// Whether the main sprite browser should be opened
    pub open_sprite_browser: bool,
    /// If Some, the decoration index that needs a sprite browser
    pub open_decoration_browser: Option<usize>,
    /// Whether mob registration was triggered
    pub register_mob: bool,
}

/// Render the complete properties panel
///
/// This is the main entry point for rendering all mob properties. It handles:
/// - Save/Reload action buttons
/// - General properties (name, spawnable, faction, etc.)
/// - Combat properties (health, damage, speed)
/// - Sprite and decorations
/// - Colliders
/// - Spawners (projectile and mob)
/// - Jointed mobs
/// - Behavior tree
pub fn properties_panel_ui(
    ui: &mut egui::Ui,
    session: &mut EditorSession,
    sprite_registry: &SpriteRegistry,
    mob_registry: &MobAssetRegistry,
    file_tree: &FileTreeState,
    config: &EditorConfig,
    save_events: &mut MessageWriter<SaveMobEvent>,
    reload_events: &mut MessageWriter<ReloadMobEvent>,
) -> PropertiesPanelResult {
    let mut result = PropertiesPanelResult::default();
    let mut modified = false;

    // File info header with registration status
    if render_file_info(ui, session, mob_registry, config) {
        result.register_mob = true;
    }

    // Action buttons below the title
    render_action_buttons(ui, session, save_events, reload_events);

    ui.separator();

    // Get the merged display table and patch-only table
    let (display_table, patch_table) = match get_display_tables(session) {
        Some(tables) => tables,
        None => {
            ui.label("No mob loaded");
            return result;
        }
    };

    let is_patch = session.file_type == FileType::MobPatch;

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            // General Properties
            render_general_properties(ui, &display_table, &patch_table, session, is_patch, &mut modified);

            ui.separator();

            // Combat Properties
            render_combat_properties(ui, &display_table, &patch_table, session, is_patch, &mut modified);

            ui.separator();

            // Sprite picker
            if decorations::render_sprite_picker(
                ui,
                &display_table,
                &patch_table,
                session,
                sprite_registry,
                is_patch,
                &mut modified,
                config,
            ) {
                result.open_sprite_browser = true;
            }

            ui.separator();

            // Decorations
            if let Some(idx) = decorations::render_decorations_section(
                ui,
                &display_table,
                &patch_table,
                session,
                sprite_registry,
                is_patch,
                &mut modified,
                config,
            ) {
                result.open_decoration_browser = Some(idx);
            }

            ui.separator();

            // Colliders
            colliders::render_colliders_section(
                ui,
                &display_table,
                &patch_table,
                session,
                is_patch,
                &mut modified,
            );

            ui.separator();

            // Projectile Spawners
            spawners::render_projectile_spawners_section(
                ui,
                &display_table,
                &patch_table,
                session,
                is_patch,
                &mut modified,
            );

            ui.separator();

            // Mob Spawners
            spawners::render_mob_spawners_section(
                ui,
                &display_table,
                &patch_table,
                session,
                is_patch,
                &mut modified,
                file_tree,
            );

            ui.separator();

            // Jointed Mobs
            jointed::render_jointed_mobs_section(
                ui,
                &display_table,
                &patch_table,
                session,
                is_patch,
                &mut modified,
                file_tree,
            );

            ui.separator();

            // Behavior Tree
            behavior::render_behavior_section(
                ui,
                &display_table,
                &patch_table,
                session,
                is_patch,
                &mut modified,
            );
        });

    // Mark session modified if any section made changes
    if modified {
        session.check_modified();
        // Update merged preview data for patches so preview reflects changes
        session.update_merged_for_preview();
    }

    result
}

/// Get the display tables for rendering
///
/// For regular .mob files, returns the mob data for both
/// For .mobpatch files, returns merged data for display and patch-only data for checking
fn get_display_tables(session: &EditorSession) -> Option<(toml::value::Table, toml::value::Table)> {
    let current_mob = session.current_mob.as_ref()?.as_table()?;

    if session.file_type == FileType::MobPatch {
        // For patches, compute merged data on-the-fly from base + current patch
        // This ensures display is always in sync with edits
        if let Some(base) = session.base_mob.as_ref() {
            let mut merged = base.clone();
            crate::file::merge_toml_values(&mut merged, session.current_mob.clone().unwrap());
            let merged_table = merged.as_table()?.clone();
            Some((merged_table, current_mob.clone()))
        } else {
            // Fallback to just current if no base data
            let cloned = current_mob.clone();
            Some((cloned.clone(), cloned))
        }
    } else {
        // For regular mobs, display = current = patch
        let cloned = current_mob.clone();
        Some((cloned.clone(), cloned))
    }
}

/// Render the save and reload action buttons
fn render_action_buttons(
    ui: &mut egui::Ui,
    session: &EditorSession,
    save_events: &mut MessageWriter<SaveMobEvent>,
    reload_events: &mut MessageWriter<ReloadMobEvent>,
) {
    ui.horizontal(|ui| {
        let save_enabled = session.is_modified && session.current_path.is_some();
        if ui
            .add_enabled(save_enabled, egui::Button::new("💾 Save"))
            .clicked()
        {
            save_events.write(SaveMobEvent {
                path: None,
                skip_registration_check: false,
            });
        }

        let reload_enabled = session.current_path.is_some();
        if ui
            .add_enabled(reload_enabled, egui::Button::new("🔄 Reload"))
            .clicked()
        {
            reload_events.write(ReloadMobEvent);
        }
    });
}

/// Render file information header
///
/// Returns true if the "Register" button was clicked
fn render_file_info(
    ui: &mut egui::Ui,
    session: &EditorSession,
    mob_registry: &MobAssetRegistry,
    config: &EditorConfig,
) -> bool {
    let mut register_clicked = false;

    if let Some(path) = &session.current_path {
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown");

        // Truncate long filenames
        let display_name = truncate_filename(filename, MAX_FILENAME_DISPLAY_LEN);

        ui.horizontal(|ui| {
            let label_response = ui.label(
                egui::RichText::new(&display_name)
                    .heading()
                    .color(egui::Color32::WHITE),
            );

            // Show full filename on hover if truncated
            if display_name != filename {
                label_response.on_hover_text(filename);
            }

            if session.is_modified {
                ui.label(egui::RichText::new("*").color(egui::Color32::YELLOW));
            }
        });

        // Show registration status
        let is_registered = mob_registry.is_registered(path, config);
        ui.horizontal(|ui| {
            if is_registered {
                ui.label(
                    egui::RichText::new("✔ Registered")
                        .small()
                        .color(egui::Color32::from_rgb(100, 200, 100)),
                );
            } else {
                ui.label(
                    egui::RichText::new("⚠ Not registered")
                        .small()
                        .color(egui::Color32::YELLOW),
                );
                if ui.small_button("Register").clicked() {
                    register_clicked = true;
                }
            }
        });

        // Show patch info if applicable
        if session.file_type == FileType::MobPatch {
            if session.base_mob.is_some() {
                // Base mob was found - show which mob this patches
                if let Some(base_path) = &session.expected_base_path {
                    ui.label(
                        egui::RichText::new(format!("Patches: {}", base_path))
                            .small()
                            .color(egui::Color32::from_rgb(100, 200, 100)),
                    );
                }
            } else {
                // Base mob not found - show warning with advice
                ui.label(
                    egui::RichText::new("Base mob not found")
                        .small()
                        .color(egui::Color32::from_rgb(255, 180, 50)),
                );
                if let Some(expected) = &session.expected_base_path {
                    ui.label(
                        egui::RichText::new(format!("Expected: assets/mobs/{}", expected))
                            .small()
                            .color(egui::Color32::GRAY),
                    );
                }
            }
        }
    }

    register_clicked
}

/// Render general properties section
fn render_general_properties(
    ui: &mut egui::Ui,
    display_table: &toml::value::Table,
    patch_table: &toml::value::Table,
    session: &mut EditorSession,
    is_patch: bool,
    modified: &mut bool,
) {
    // Check if any General field is modified
    let section_modified = session.is_field_modified("name")
        || session.is_field_modified("spawnable")
        || session.is_field_modified("z_level");

    let header_text =
        egui::RichText::new("General").color(fields::header_color(ui, section_modified));
    egui::CollapsingHeader::new(header_text)
        .default_open(true)
        .show(ui, |ui| {
            // Name
            let name_patched = is_patch && patch_table.contains_key("name");
            let name_modified = session.is_field_modified("name");
            let name = display_table
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            match fields::render_string_field(ui, "Name:", name, name_patched, is_patch, name_modified) {
                FieldResult::Changed(new_val) => {
                    set_field(session, "name", toml::Value::String(new_val));
                    *modified = true;
                }
                FieldResult::Reset => {
                    remove_field(session, "name");
                    *modified = true;
                }
                FieldResult::NoChange => {}
            }

            // Spawnable (default: true)
            const SPAWNABLE_DEFAULT: bool = true;
            let spawnable_patched = is_patch && patch_table.contains_key("spawnable");
            let spawnable_modified = session.is_field_modified("spawnable");
            let spawnable = display_table
                .get("spawnable")
                .and_then(|v| v.as_bool())
                .unwrap_or(SPAWNABLE_DEFAULT);
            match fields::render_bool_field(
                ui,
                "Spawnable:",
                spawnable,
                spawnable_patched,
                is_patch,
                spawnable_modified,
            ) {
                FieldResult::Changed(new_val) => {
                    set_field_with_default(
                        session,
                        "spawnable",
                        toml::Value::Boolean(new_val),
                        toml::Value::Boolean(SPAWNABLE_DEFAULT),
                    );
                    *modified = true;
                }
                FieldResult::Reset => {
                    remove_field(session, "spawnable");
                    *modified = true;
                }
                FieldResult::NoChange => {}
            }

            // Z-Level (default: 0.0)
            const Z_LEVEL_DEFAULT: f64 = 0.0;
            let z_patched = is_patch && patch_table.contains_key("z_level");
            let z_modified = session.is_field_modified("z_level");
            let z_level = display_table
                .get("z_level")
                .and_then(|v| v.as_float())
                .unwrap_or(Z_LEVEL_DEFAULT) as f32;
            match fields::render_float_field(
                ui,
                "Z-Level:",
                z_level,
                -100.0..=100.0,
                Some(0.1),
                z_patched,
                is_patch,
                z_modified,
            ) {
                FieldResult::Changed(new_val) => {
                    set_field_with_default(
                        session,
                        "z_level",
                        toml::Value::Float(new_val as f64),
                        toml::Value::Float(Z_LEVEL_DEFAULT),
                    );
                    *modified = true;
                }
                FieldResult::Reset => {
                    remove_field(session, "z_level");
                    *modified = true;
                }
                FieldResult::NoChange => {}
            }
        });
}

/// Render combat properties section
fn render_combat_properties(
    ui: &mut egui::Ui,
    display_table: &toml::value::Table,
    patch_table: &toml::value::Table,
    session: &mut EditorSession,
    is_patch: bool,
    modified: &mut bool,
) {
    // Check if any Combat field is modified
    let section_modified = session.is_field_modified("health")
        || session.is_field_modified("collision_damage")
        || session.is_field_modified("max_linear_speed")
        || session.is_field_modified("linear_acceleration")
        || session.is_field_modified("projectile_speed");

    let header_text =
        egui::RichText::new("Combat").color(fields::header_color(ui, section_modified));
    egui::CollapsingHeader::new(header_text)
        .default_open(true)
        .show(ui, |ui| {
            // Health (default: 100)
            const HEALTH_DEFAULT: i64 = 100;
            let health_patched = is_patch && patch_table.contains_key("health");
            let health_modified = session.is_field_modified("health");
            let health = display_table
                .get("health")
                .and_then(|v| v.as_integer())
                .unwrap_or(HEALTH_DEFAULT) as i32;
            match fields::render_int_field(
                ui,
                "Health:",
                health,
                1..=10000,
                health_patched,
                is_patch,
                health_modified,
            ) {
                FieldResult::Changed(new_val) => {
                    set_field_with_default(
                        session,
                        "health",
                        toml::Value::Integer(new_val as i64),
                        toml::Value::Integer(HEALTH_DEFAULT),
                    );
                    *modified = true;
                }
                FieldResult::Reset => {
                    remove_field(session, "health");
                    *modified = true;
                }
                FieldResult::NoChange => {}
            }

            // Collision Damage (default: 0)
            const DAMAGE_DEFAULT: i64 = 0;
            let damage_patched = is_patch && patch_table.contains_key("collision_damage");
            let damage_modified = session.is_field_modified("collision_damage");
            let damage = display_table
                .get("collision_damage")
                .and_then(|v| v.as_integer())
                .unwrap_or(DAMAGE_DEFAULT) as i32;
            match fields::render_int_field(
                ui,
                "Collision Damage:",
                damage,
                0..=1000,
                damage_patched,
                is_patch,
                damage_modified,
            ) {
                FieldResult::Changed(new_val) => {
                    set_field_with_default(
                        session,
                        "collision_damage",
                        toml::Value::Integer(new_val as i64),
                        toml::Value::Integer(DAMAGE_DEFAULT),
                    );
                    *modified = true;
                }
                FieldResult::Reset => {
                    remove_field(session, "collision_damage");
                    *modified = true;
                }
                FieldResult::NoChange => {}
            }

            // Max Linear Speed (default: [50.0, 50.0])
            const SPEED_DEFAULT: (f32, f32) = (50.0, 50.0);
            let speed_patched = is_patch && patch_table.contains_key("max_linear_speed");
            let speed_modified = session.is_field_modified("max_linear_speed");
            let (speed_x, speed_y) =
                fields::get_vec2_value(display_table, "max_linear_speed", SPEED_DEFAULT.0, SPEED_DEFAULT.1);
            match fields::render_vec2_field(
                ui,
                "Max Speed:",
                speed_x,
                speed_y,
                0.0..=1000.0,
                Some(1.0),
                speed_patched,
                is_patch,
                speed_modified,
            ) {
                FieldResult::Changed((x, y)) => {
                    set_vec2_field_with_default(session, "max_linear_speed", x, y, SPEED_DEFAULT.0, SPEED_DEFAULT.1);
                    *modified = true;
                }
                FieldResult::Reset => {
                    remove_field(session, "max_linear_speed");
                    *modified = true;
                }
                FieldResult::NoChange => {}
            }

            // Linear Acceleration (default: [100.0, 100.0])
            const ACCEL_DEFAULT: (f32, f32) = (100.0, 100.0);
            let accel_patched = is_patch && patch_table.contains_key("linear_acceleration");
            let accel_modified = session.is_field_modified("linear_acceleration");
            let (accel_x, accel_y) =
                fields::get_vec2_value(display_table, "linear_acceleration", ACCEL_DEFAULT.0, ACCEL_DEFAULT.1);
            match fields::render_vec2_field(
                ui,
                "Acceleration:",
                accel_x,
                accel_y,
                0.0..=2000.0,
                Some(1.0),
                accel_patched,
                is_patch,
                accel_modified,
            ) {
                FieldResult::Changed((x, y)) => {
                    set_vec2_field_with_default(session, "linear_acceleration", x, y, ACCEL_DEFAULT.0, ACCEL_DEFAULT.1);
                    *modified = true;
                }
                FieldResult::Reset => {
                    remove_field(session, "linear_acceleration");
                    *modified = true;
                }
                FieldResult::NoChange => {}
            }

            // Projectile Speed (default: 150.0)
            const PROJ_SPEED_DEFAULT: f64 = 150.0;
            let proj_speed_patched = is_patch && patch_table.contains_key("projectile_speed");
            let proj_speed_modified = session.is_field_modified("projectile_speed");
            let proj_speed = display_table
                .get("projectile_speed")
                .and_then(|v| v.as_float())
                .unwrap_or(PROJ_SPEED_DEFAULT) as f32;
            match fields::render_float_field(
                ui,
                "Projectile Speed:",
                proj_speed,
                0.0..=1000.0,
                Some(1.0),
                proj_speed_patched,
                is_patch,
                proj_speed_modified,
            ) {
                FieldResult::Changed(new_val) => {
                    set_field_with_default(
                        session,
                        "projectile_speed",
                        toml::Value::Float(new_val as f64),
                        toml::Value::Float(PROJ_SPEED_DEFAULT),
                    );
                    *modified = true;
                }
                FieldResult::Reset => {
                    remove_field(session, "projectile_speed");
                    *modified = true;
                }
                FieldResult::NoChange => {}
            }
        });
}

// =============================================================================
// Helper functions for field manipulation
// =============================================================================

/// Set a field on the current mob with smart modification tracking.
///
/// - If original had this field: always set the value (so comparison works correctly)
/// - If original didn't have this field and new value equals default: remove from current
///   (user reverted to default, so don't add the field)
/// - If original didn't have this field and new value differs from default: add the field
fn set_field_with_default(
    session: &mut EditorSession,
    key: &str,
    value: toml::Value,
    default: toml::Value,
) {
    let original_has_field = session
        .original_mob
        .as_ref()
        .and_then(|m| m.get(key))
        .is_some();

    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        if original_has_field {
            // Original had this field - always set the new value
            mob.insert(key.to_string(), value);
        } else {
            // Original didn't have this field
            if EditorSession::values_equal(&value, &default) {
                // Setting to default value - don't add field (remove if exists)
                mob.remove(key);
            } else {
                // Setting to non-default value - add the field
                mob.insert(key.to_string(), value);
            }
        }
    }
}

/// Simple set_field for fields that always exist (no default handling needed)
fn set_field(session: &mut EditorSession, key: &str, value: toml::Value) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        mob.insert(key.to_string(), value);
    }
}

/// Remove a field from the current mob
fn remove_field(session: &mut EditorSession, key: &str) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        mob.remove(key);
    }
}

/// Set a Vec2 field on the current mob with smart modification tracking.
fn set_vec2_field_with_default(
    session: &mut EditorSession,
    key: &str,
    x: f32,
    y: f32,
    default_x: f32,
    default_y: f32,
) {
    let new_value = toml::Value::Array(vec![
        toml::Value::Float(x as f64),
        toml::Value::Float(y as f64),
    ]);
    let default_value = toml::Value::Array(vec![
        toml::Value::Float(default_x as f64),
        toml::Value::Float(default_y as f64),
    ]);

    let original_has_field = session
        .original_mob
        .as_ref()
        .and_then(|m| m.get(key))
        .is_some();

    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        if original_has_field {
            // Original had this field - always set the new value
            fields::set_vec2_value(mob, key, x, y);
        } else {
            // Original didn't have this field
            if EditorSession::values_equal(&new_value, &default_value) {
                // Setting to default value - don't add field
                mob.remove(key);
            } else {
                // Setting to non-default value - add the field
                fields::set_vec2_value(mob, key, x, y);
            }
        }
    }
}

/// Update a decoration's sprite path
///
/// This is exported for use by the sprite browser dialog
pub fn update_decoration_sprite(session: &mut EditorSession, index: usize, sprite_path: &str) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut())
        && let Some(decorations) = mob.get_mut("decorations").and_then(|v| v.as_array_mut())
        && let Some(decoration) = decorations.get_mut(index).and_then(|v| v.as_array_mut())
        && !decoration.is_empty()
    {
        decoration[0] = toml::Value::String(sprite_path.to_string());
    }
    session.check_modified();
    session.update_merged_for_preview();
}
