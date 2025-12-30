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

use bevy_egui::egui;

use crate::data::{EditorSession, FileType, SpriteRegistry};
use crate::file::FileTreeState;
use crate::plugin::EditorConfig;
use crate::preview::JointedMobCache;

// Re-export commonly used items
pub use fields::{FieldResult, INHERITED_COLOR, PATCHED_COLOR};

/// Result from rendering the properties panel
#[derive(Default)]
pub struct PropertiesPanelResult {
    /// Whether the main sprite browser should be opened
    pub open_sprite_browser: bool,
    /// If Some, the decoration index that needs a sprite browser
    pub open_decoration_browser: Option<usize>,
}

/// Render the complete properties panel.
///
/// This is the main entry point for rendering all mob properties. It handles:
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
    _jointed_cache: &JointedMobCache,
    file_tree: &FileTreeState,
    config: &EditorConfig,
) -> PropertiesPanelResult {
    let mut result = PropertiesPanelResult::default();
    let mut modified = false;

    // Get the merged display table and patch-only table
    let (display_table, patch_table) = match get_display_tables(session, config) {
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
            // File info header
            render_file_info(ui, session);

            ui.separator();

            // General Properties
            render_general_properties(ui, &display_table, &patch_table, session, is_patch);

            ui.separator();

            // Combat Properties
            render_combat_properties(ui, &display_table, &patch_table, session, is_patch);

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
        session.is_modified = true;
    }

    result
}

/// Get the display tables for rendering.
///
/// For regular .mob files, returns the mob data for both.
/// For .mobpatch files, returns merged data for display and patch-only data for checking.
fn get_display_tables(
    session: &EditorSession,
    _config: &EditorConfig,
) -> Option<(toml::value::Table, toml::value::Table)> {
    let current_mob = session.current_mob.as_ref()?.as_table()?;

    if session.file_type == FileType::MobPatch {
        // For patches, use merged_for_preview if available
        if let Some(merged) = session.merged_for_preview.as_ref() {
            let merged_table = merged.as_table()?.clone();
            Some((merged_table, current_mob.clone()))
        } else {
            // Fallback to just current if no merged data
            Some((current_mob.clone(), current_mob.clone()))
        }
    } else {
        // For regular mobs, display = current = patch
        Some((current_mob.clone(), current_mob.clone()))
    }
}

/// Render file information header.
fn render_file_info(ui: &mut egui::Ui, session: &EditorSession) {
    if let Some(path) = &session.current_path {
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown");

        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(filename)
                    .heading()
                    .color(egui::Color32::WHITE),
            );
            if session.is_modified {
                ui.label(egui::RichText::new("*").color(egui::Color32::YELLOW));
            }
        });

        // Show patch info if applicable
        if session.file_type == FileType::MobPatch {
            // For patches, show that it's patching something
            // The base mob reference is stored in the base_mob field
            if session.base_mob.is_some() {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("(patch file)")
                            .small()
                            .color(PATCHED_COLOR),
                    );
                });
            }
        }
    }
}

/// Render general properties section.
fn render_general_properties(
    ui: &mut egui::Ui,
    display_table: &toml::value::Table,
    patch_table: &toml::value::Table,
    session: &mut EditorSession,
    is_patch: bool,
) {
    egui::CollapsingHeader::new("General")
        .default_open(true)
        .show(ui, |ui| {
            // Name
            let name_patched = is_patch && patch_table.contains_key("name");
            let name = display_table
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            match fields::render_string_field(ui, "Name:", name, name_patched, is_patch) {
                FieldResult::Changed(new_val) => {
                    set_field(session, "name", toml::Value::String(new_val));
                    session.is_modified = true;
                }
                FieldResult::Reset => {
                    remove_field(session, "name");
                    session.is_modified = true;
                }
                FieldResult::NoChange => {}
            }

            // Spawnable
            let spawnable_patched = is_patch && patch_table.contains_key("spawnable");
            let spawnable = display_table
                .get("spawnable")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            match fields::render_bool_field(ui, "Spawnable:", spawnable, spawnable_patched, is_patch)
            {
                FieldResult::Changed(new_val) => {
                    set_field(session, "spawnable", toml::Value::Boolean(new_val));
                    session.is_modified = true;
                }
                FieldResult::Reset => {
                    remove_field(session, "spawnable");
                    session.is_modified = true;
                }
                FieldResult::NoChange => {}
            }

            // Z-Level
            let z_patched = is_patch && patch_table.contains_key("z_level");
            let z_level = display_table
                .get("z_level")
                .and_then(|v| v.as_float())
                .unwrap_or(0.0) as f32;
            match fields::render_float_field(
                ui,
                "Z-Level:",
                z_level,
                -100.0..=100.0,
                Some(0.1),
                z_patched,
                is_patch,
            ) {
                FieldResult::Changed(new_val) => {
                    set_field(session, "z_level", toml::Value::Float(new_val as f64));
                    session.is_modified = true;
                }
                FieldResult::Reset => {
                    remove_field(session, "z_level");
                    session.is_modified = true;
                }
                FieldResult::NoChange => {}
            }
        });
}

/// Render combat properties section.
fn render_combat_properties(
    ui: &mut egui::Ui,
    display_table: &toml::value::Table,
    patch_table: &toml::value::Table,
    session: &mut EditorSession,
    is_patch: bool,
) {
    egui::CollapsingHeader::new("Combat")
        .default_open(true)
        .show(ui, |ui| {
            // Health
            let health_patched = is_patch && patch_table.contains_key("health");
            let health = display_table
                .get("health")
                .and_then(|v| v.as_integer())
                .unwrap_or(100) as i32;
            match fields::render_int_field(ui, "Health:", health, 1..=10000, health_patched, is_patch)
            {
                FieldResult::Changed(new_val) => {
                    set_field(session, "health", toml::Value::Integer(new_val as i64));
                    session.is_modified = true;
                }
                FieldResult::Reset => {
                    remove_field(session, "health");
                    session.is_modified = true;
                }
                FieldResult::NoChange => {}
            }

            // Collision Damage
            let damage_patched = is_patch && patch_table.contains_key("collision_damage");
            let damage = display_table
                .get("collision_damage")
                .and_then(|v| v.as_integer())
                .unwrap_or(0) as i32;
            match fields::render_int_field(
                ui,
                "Collision Damage:",
                damage,
                0..=1000,
                damage_patched,
                is_patch,
            ) {
                FieldResult::Changed(new_val) => {
                    set_field(
                        session,
                        "collision_damage",
                        toml::Value::Integer(new_val as i64),
                    );
                    session.is_modified = true;
                }
                FieldResult::Reset => {
                    remove_field(session, "collision_damage");
                    session.is_modified = true;
                }
                FieldResult::NoChange => {}
            }

            // Max Linear Speed
            let speed_patched = is_patch && patch_table.contains_key("max_linear_speed");
            let (speed_x, speed_y) =
                fields::get_vec2_value(display_table, "max_linear_speed", 50.0, 50.0);
            match fields::render_vec2_field(
                ui,
                "Max Speed:",
                speed_x,
                speed_y,
                0.0..=1000.0,
                Some(1.0),
                speed_patched,
                is_patch,
            ) {
                FieldResult::Changed((x, y)) => {
                    set_vec2_field(session, "max_linear_speed", x, y);
                    session.is_modified = true;
                }
                FieldResult::Reset => {
                    remove_field(session, "max_linear_speed");
                    session.is_modified = true;
                }
                FieldResult::NoChange => {}
            }

            // Linear Acceleration
            let accel_patched = is_patch && patch_table.contains_key("linear_acceleration");
            let (accel_x, accel_y) =
                fields::get_vec2_value(display_table, "linear_acceleration", 100.0, 100.0);
            match fields::render_vec2_field(
                ui,
                "Acceleration:",
                accel_x,
                accel_y,
                0.0..=2000.0,
                Some(1.0),
                accel_patched,
                is_patch,
            ) {
                FieldResult::Changed((x, y)) => {
                    set_vec2_field(session, "linear_acceleration", x, y);
                    session.is_modified = true;
                }
                FieldResult::Reset => {
                    remove_field(session, "linear_acceleration");
                    session.is_modified = true;
                }
                FieldResult::NoChange => {}
            }

            // Projectile Speed
            let proj_speed_patched = is_patch && patch_table.contains_key("projectile_speed");
            let proj_speed = display_table
                .get("projectile_speed")
                .and_then(|v| v.as_float())
                .unwrap_or(150.0) as f32;
            match fields::render_float_field(
                ui,
                "Projectile Speed:",
                proj_speed,
                0.0..=1000.0,
                Some(1.0),
                proj_speed_patched,
                is_patch,
            ) {
                FieldResult::Changed(new_val) => {
                    set_field(
                        session,
                        "projectile_speed",
                        toml::Value::Float(new_val as f64),
                    );
                    session.is_modified = true;
                }
                FieldResult::Reset => {
                    remove_field(session, "projectile_speed");
                    session.is_modified = true;
                }
                FieldResult::NoChange => {}
            }
        });
}

// =============================================================================
// Helper functions for field manipulation
// =============================================================================

/// Set a field on the current mob.
fn set_field(session: &mut EditorSession, key: &str, value: toml::Value) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        mob.insert(key.to_string(), value);
    }
}

/// Remove a field from the current mob.
fn remove_field(session: &mut EditorSession, key: &str) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        mob.remove(key);
    }
}

/// Set a Vec2 field on the current mob.
fn set_vec2_field(session: &mut EditorSession, key: &str, x: f32, y: f32) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        fields::set_vec2_value(mob, key, x, y);
    }
}

/// Update a decoration's sprite path.
///
/// This is exported for use by the sprite browser dialog.
pub fn update_decoration_sprite(session: &mut EditorSession, index: usize, sprite_path: &str) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        if let Some(decorations) = mob.get_mut("decorations").and_then(|v| v.as_array_mut()) {
            if let Some(decoration) = decorations.get_mut(index).and_then(|v| v.as_array_mut()) {
                if !decoration.is_empty() {
                    decoration[0] = toml::Value::String(sprite_path.to_string());
                }
            }
        }
    }
    session.is_modified = true;
}
