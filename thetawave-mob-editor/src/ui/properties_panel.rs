use std::path::PathBuf;

use bevy_egui::egui;

use crate::data::{EditorSession, FileType, SpriteRegistry, SpriteSource};

/// Color for patched/overridden values
const PATCHED_COLOR: egui::Color32 = egui::Color32::from_rgb(100, 200, 255);
/// Color for inherited values (from base mob)
const INHERITED_COLOR: egui::Color32 = egui::Color32::from_rgb(140, 140, 140);

/// Valid projectile types
const PROJECTILE_TYPES: &[&str] = &["Bullet", "Blast"];
/// Valid faction types
const FACTIONS: &[&str] = &["Enemy", "Ally"];

/// Result of browsing for a sprite to register
#[derive(Debug, Clone)]
pub enum BrowseResult {
    /// Successfully identified a sprite to register
    Register(BrowseRegistrationRequest),
    /// File was selected but is not in a valid assets directory
    InvalidLocation(String),
}

/// Request to browse for and register a new sprite
#[derive(Debug, Clone)]
pub struct BrowseRegistrationRequest {
    /// The asset path (relative to assets directory)
    pub asset_path: String,
    /// Whether this is an extended sprite
    pub is_extended: bool,
    /// The path to use in the mob file
    pub mob_path: String,
}

/// Result of browsing for a decoration sprite to register
#[derive(Debug, Clone)]
pub enum DecorationBrowseResult {
    /// Successfully identified a sprite to register
    Register {
        /// The decoration index
        index: usize,
        /// The registration request
        request: BrowseRegistrationRequest,
    },
    /// File was selected but is not in a valid assets directory
    InvalidLocation(String),
}

/// Results from the properties panel for browse & register actions
#[derive(Debug, Clone, Default)]
pub struct PropertiesPanelResult {
    /// Browse result for main sprite
    pub sprite_browse: Option<BrowseResult>,
    /// Browse result for decoration sprite
    pub decoration_browse: Option<DecorationBrowseResult>,
}

/// Render the properties editing panel
/// Returns browse results if the user clicked browse to add a new sprite
pub fn properties_panel_ui(
    ui: &mut egui::Ui,
    session: &mut EditorSession,
    sprite_registry: &SpriteRegistry,
) -> PropertiesPanelResult {
    let mut result = PropertiesPanelResult::default();

    ui.heading("Properties");

    // Show patch indicator if applicable
    if session.file_type == FileType::MobPatch {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("📋 PATCH FILE").color(PATCHED_COLOR));
            if session.base_mob.is_some() {
                ui.label(egui::RichText::new("(base found)").small().color(egui::Color32::GREEN));
            } else {
                ui.label(egui::RichText::new("(no base)").small().color(egui::Color32::RED));
            }
        });

        // Legend
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("●").color(PATCHED_COLOR));
            ui.label(egui::RichText::new("= patched").small());
            ui.add_space(10.0);
            ui.label(egui::RichText::new("○").color(INHERITED_COLOR));
            ui.label(egui::RichText::new("= inherited").small());
        });
    }

    ui.separator();

    // Track if any value was modified during this frame
    let mut modified = false;

    // For patches, we display merged data but edit the patch
    let is_patch = session.file_type == FileType::MobPatch;

    // Get display data (merged for patches, current for regular mobs)
    let display_mob = if is_patch {
        session.merged_for_preview.clone().or_else(|| session.current_mob.clone())
    } else {
        session.current_mob.clone()
    };

    let Some(display_mob) = display_mob else {
        ui.label("No mob loaded");
        return result;
    };

    let Some(display_table) = display_mob.as_table().cloned() else {
        ui.colored_label(egui::Color32::RED, "Invalid mob data");
        return result;
    };

    // Get patch table for checking what's overridden
    let patch_table = session.current_mob
        .as_ref()
        .and_then(|v| v.as_table())
        .cloned()
        .unwrap_or_default();

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // General section
            egui::CollapsingHeader::new("General")
                .default_open(true)
                .show(ui, |ui| {
                    // Name
                    let is_patched = is_patch && patch_table.contains_key("name");
                    match render_string_field(
                        ui, "Name:", "name",
                        display_table.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                        is_patched, is_patch,
                    ) {
                        FieldResult::Changed(new_val) => {
                            if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                                mob.insert("name".to_string(), toml::Value::String(new_val));
                                modified = true;
                            }
                        }
                        FieldResult::Reset => {
                            if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                                mob.remove("name");
                                modified = true;
                            }
                        }
                        FieldResult::NoChange => {}
                    }

                    // Spawnable
                    let is_patched = is_patch && patch_table.contains_key("spawnable");
                    match render_bool_field(
                        ui, "Spawnable:", "spawnable",
                        display_table.get("spawnable").and_then(|v| v.as_bool()).unwrap_or(true),
                        is_patched, is_patch,
                    ) {
                        FieldResult::Changed(new_val) => {
                            if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                                mob.insert("spawnable".to_string(), toml::Value::Boolean(new_val));
                                modified = true;
                            }
                        }
                        FieldResult::Reset => {
                            if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                                mob.remove("spawnable");
                                modified = true;
                            }
                        }
                        FieldResult::NoChange => {}
                    }

                    // Sprite (dropdown picker from registered sprites)
                    if let Some(sprite_result) = render_sprite_picker(
                        ui,
                        &display_table,
                        &patch_table,
                        session,
                        sprite_registry,
                        is_patch,
                        &mut modified,
                    ) {
                        result.sprite_browse = Some(sprite_result);
                    }
                });

            // Decorations section
            if let Some(dec_result) = render_decorations_section(
                ui,
                &display_table,
                &patch_table,
                session,
                sprite_registry,
                is_patch,
                &mut modified,
            ) {
                result.decoration_browse = Some(dec_result);
            }

            // Combat section
            egui::CollapsingHeader::new("Combat")
                .default_open(true)
                .show(ui, |ui| {
                    // Health
                    let is_patched = is_patch && patch_table.contains_key("health");
                    match render_int_field(
                        ui, "Health:", "health",
                        display_table.get("health").and_then(|v| v.as_integer()).unwrap_or(50) as i32,
                        1..=10000,
                        is_patched, is_patch,
                    ) {
                        FieldResult::Changed(new_val) => {
                            if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                                mob.insert("health".to_string(), toml::Value::Integer(new_val as i64));
                                modified = true;
                            }
                        }
                        FieldResult::Reset => {
                            if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                                mob.remove("health");
                                modified = true;
                            }
                        }
                        FieldResult::NoChange => {}
                    }

                    // Projectile Speed
                    let is_patched = is_patch && patch_table.contains_key("projectile_speed");
                    match render_float_field(
                        ui, "Proj Speed:", "projectile_speed",
                        display_table.get("projectile_speed").and_then(|v| v.as_float()).unwrap_or(100.0) as f32,
                        0.0..=1000.0,
                        is_patched, is_patch,
                    ) {
                        FieldResult::Changed(new_val) => {
                            if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                                mob.insert("projectile_speed".to_string(), toml::Value::Float(new_val as f64));
                                modified = true;
                            }
                        }
                        FieldResult::Reset => {
                            if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                                mob.remove("projectile_speed");
                                modified = true;
                            }
                        }
                        FieldResult::NoChange => {}
                    }

                    // Projectile Damage
                    let is_patched = is_patch && patch_table.contains_key("projectile_damage");
                    match render_int_field(
                        ui, "Proj Damage:", "projectile_damage",
                        display_table.get("projectile_damage").and_then(|v| v.as_integer()).unwrap_or(5) as i32,
                        0..=1000,
                        is_patched, is_patch,
                    ) {
                        FieldResult::Changed(new_val) => {
                            if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                                mob.insert("projectile_damage".to_string(), toml::Value::Integer(new_val as i64));
                                modified = true;
                            }
                        }
                        FieldResult::Reset => {
                            if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                                mob.remove("projectile_damage");
                                modified = true;
                            }
                        }
                        FieldResult::NoChange => {}
                    }
                });

            // Movement section
            egui::CollapsingHeader::new("Movement")
                .default_open(false)
                .show(ui, |ui| {
                    // Max Linear Speed
                    let is_patched = is_patch && patch_table.contains_key("max_linear_speed");
                    let speed = get_vec2_value(&display_table, "max_linear_speed", 20.0, 20.0);
                    match render_vec2_field(ui, "Max Speed:", "max_linear_speed", speed.0, speed.1, 0.0..=500.0, is_patched, is_patch) {
                        FieldResult::Changed((new_x, new_y)) => {
                            if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                                set_vec2_value(mob, "max_linear_speed", new_x, new_y);
                                modified = true;
                            }
                        }
                        FieldResult::Reset => {
                            if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                                mob.remove("max_linear_speed");
                                modified = true;
                            }
                        }
                        FieldResult::NoChange => {}
                    }

                    // Linear Acceleration
                    let is_patched = is_patch && patch_table.contains_key("linear_acceleration");
                    let accel = get_vec2_value(&display_table, "linear_acceleration", 0.1, 0.1);
                    match render_vec2_field_slow(ui, "Acceleration:", "linear_acceleration", accel.0, accel.1, 0.0..=10.0, 0.01, is_patched, is_patch) {
                        FieldResult::Changed((new_x, new_y)) => {
                            if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                                set_vec2_value(mob, "linear_acceleration", new_x, new_y);
                                modified = true;
                            }
                        }
                        FieldResult::Reset => {
                            if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                                mob.remove("linear_acceleration");
                                modified = true;
                            }
                        }
                        FieldResult::NoChange => {}
                    }

                    // Linear Deceleration
                    let is_patched = is_patch && patch_table.contains_key("linear_deceleration");
                    let decel = get_vec2_value(&display_table, "linear_deceleration", 0.3, 0.3);
                    match render_vec2_field_slow(ui, "Deceleration:", "linear_deceleration", decel.0, decel.1, 0.0..=10.0, 0.01, is_patched, is_patch) {
                        FieldResult::Changed((new_x, new_y)) => {
                            if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                                set_vec2_value(mob, "linear_deceleration", new_x, new_y);
                                modified = true;
                            }
                        }
                        FieldResult::Reset => {
                            if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                                mob.remove("linear_deceleration");
                                modified = true;
                            }
                        }
                        FieldResult::NoChange => {}
                    }

                    // Max Angular Speed
                    let is_patched = is_patch && patch_table.contains_key("max_angular_speed");
                    match render_float_field_slow(ui, "Angular Speed:", "max_angular_speed", display_table.get("max_angular_speed").and_then(|v| v.as_float()).unwrap_or(1.0) as f32, 0.0..=20.0, 0.1, is_patched, is_patch) {
                        FieldResult::Changed(new_val) => {
                            if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                                mob.insert("max_angular_speed".to_string(), toml::Value::Float(new_val as f64));
                                modified = true;
                            }
                        }
                        FieldResult::Reset => {
                            if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                                mob.remove("max_angular_speed");
                                modified = true;
                            }
                        }
                        FieldResult::NoChange => {}
                    }
                });

            // Physics section
            egui::CollapsingHeader::new("Physics")
                .default_open(false)
                .show(ui, |ui| {
                    // Z Level
                    let is_patched = is_patch && patch_table.contains_key("z_level");
                    match render_float_field_slow(ui, "Z Level:", "z_level", display_table.get("z_level").and_then(|v| v.as_float()).unwrap_or(0.0) as f32, -10.0..=10.0, 0.1, is_patched, is_patch) {
                        FieldResult::Changed(new_val) => {
                            if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                                mob.insert("z_level".to_string(), toml::Value::Float(new_val as f64));
                                modified = true;
                            }
                        }
                        FieldResult::Reset => {
                            if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                                mob.remove("z_level");
                                modified = true;
                            }
                        }
                        FieldResult::NoChange => {}
                    }

                    // Rotation Locked
                    let is_patched = is_patch && patch_table.contains_key("rotation_locked");
                    match render_bool_field(ui, "Rot Locked:", "rotation_locked", display_table.get("rotation_locked").and_then(|v| v.as_bool()).unwrap_or(true), is_patched, is_patch) {
                        FieldResult::Changed(new_val) => {
                            if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                                mob.insert("rotation_locked".to_string(), toml::Value::Boolean(new_val));
                                modified = true;
                            }
                        }
                        FieldResult::Reset => {
                            if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                                mob.remove("rotation_locked");
                                modified = true;
                            }
                        }
                        FieldResult::NoChange => {}
                    }

                    // Restitution
                    let is_patched = is_patch && patch_table.contains_key("restitution");
                    match render_float_field_slow(ui, "Restitution:", "restitution", display_table.get("restitution").and_then(|v| v.as_float()).unwrap_or(0.5) as f32, 0.0..=1.0, 0.01, is_patched, is_patch) {
                        FieldResult::Changed(new_val) => {
                            if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                                mob.insert("restitution".to_string(), toml::Value::Float(new_val as f64));
                                modified = true;
                            }
                        }
                        FieldResult::Reset => {
                            if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                                mob.remove("restitution");
                                modified = true;
                            }
                        }
                        FieldResult::NoChange => {}
                    }

                    // Friction
                    let is_patched = is_patch && patch_table.contains_key("friction");
                    match render_float_field_slow(ui, "Friction:", "friction", display_table.get("friction").and_then(|v| v.as_float()).unwrap_or(0.5) as f32, 0.0..=2.0, 0.01, is_patched, is_patch) {
                        FieldResult::Changed(new_val) => {
                            if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                                mob.insert("friction".to_string(), toml::Value::Float(new_val as f64));
                                modified = true;
                            }
                        }
                        FieldResult::Reset => {
                            if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                                mob.remove("friction");
                                modified = true;
                            }
                        }
                        FieldResult::NoChange => {}
                    }

                    // Density
                    let is_patched = is_patch && patch_table.contains_key("collider_density");
                    match render_float_field_slow(ui, "Density:", "collider_density", display_table.get("collider_density").and_then(|v| v.as_float()).unwrap_or(1.0) as f32, 0.1..=10.0, 0.1, is_patched, is_patch) {
                        FieldResult::Changed(new_val) => {
                            if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                                mob.insert("collider_density".to_string(), toml::Value::Float(new_val as f64));
                                modified = true;
                            }
                        }
                        FieldResult::Reset => {
                            if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                                mob.remove("collider_density");
                                modified = true;
                            }
                        }
                        FieldResult::NoChange => {}
                    }
                });

            // Colliders section
            render_colliders_section(ui, &display_table, &patch_table, session, is_patch, &mut modified);

            // Projectile Spawners section
            render_projectile_spawners_section(ui, &display_table, &patch_table, session, is_patch, &mut modified);

            // Mob Spawners section
            render_mob_spawners_section(ui, &display_table, &patch_table, session, is_patch, &mut modified);

            // Behavior section (simplified)
            egui::CollapsingHeader::new("Behavior")
                .default_open(false)
                .show(ui, |ui| {
                    let is_patched = is_patch && patch_table.contains_key("behavior");
                    let has_behavior = display_table.contains_key("behavior");

                    ui.horizontal(|ui| {
                        render_patch_indicator(ui, is_patched, is_patch && has_behavior);
                        if has_behavior {
                            ui.label("Behavior tree defined");
                        } else {
                            ui.label("No behavior tree");
                        }
                    });
                    ui.colored_label(
                        egui::Color32::GRAY,
                        "Behavior tree editing coming soon",
                    );
                });
        });

    // Check if modified after all edits
    if modified {
        // For patches, also update the merged preview
        if is_patch {
            if let (Some(base), Some(patch)) = (&session.base_mob, &session.current_mob) {
                let mut merged = base.clone();
                crate::file::merge_toml_values(&mut merged, patch.clone());
                session.merged_for_preview = Some(merged);
            }
        }
        session.check_modified();
    }

    result
}

/// Render a patch indicator (● for patched, ○ for inherited)
/// Returns true if the reset button was clicked (to remove patch)
fn render_patch_indicator(ui: &mut egui::Ui, is_patched: bool, show_indicator: bool) -> bool {
    if show_indicator {
        if is_patched {
            ui.label(egui::RichText::new("●").color(PATCHED_COLOR));
        } else {
            ui.label(egui::RichText::new("○").color(INHERITED_COLOR));
        }
    }
    false
}

/// Render a reset button for patched fields
/// Returns true if clicked
fn render_reset_button(ui: &mut egui::Ui, is_patched: bool, is_patch_file: bool) -> bool {
    if is_patch_file && is_patched {
        let response = ui.add(
            egui::Button::new(egui::RichText::new("×").color(egui::Color32::WHITE))
                .fill(egui::Color32::from_rgb(120, 60, 60))
                .min_size(egui::vec2(16.0, 16.0))
        );
        if response.on_hover_text("Remove from patch (use base value)").clicked() {
            return true;
        }
    }
    false
}

/// Field edit result
enum FieldResult<T> {
    NoChange,
    Changed(T),
    Reset, // Remove from patch
}

/// Render a string field with patch awareness
fn render_string_field(
    ui: &mut egui::Ui,
    label: &str,
    _key: &str,
    current_value: &str,
    is_patched: bool,
    is_patch_file: bool,
) -> FieldResult<String> {
    let mut result = FieldResult::NoChange;
    ui.horizontal(|ui| {
        render_patch_indicator(ui, is_patched, is_patch_file);

        let text_color = if is_patch_file && !is_patched {
            INHERITED_COLOR
        } else {
            ui.style().visuals.text_color()
        };

        ui.label(egui::RichText::new(label).color(text_color));

        let mut value = current_value.to_string();
        let response = ui.text_edit_singleline(&mut value);

        if response.changed() {
            result = FieldResult::Changed(value);
        }

        if render_reset_button(ui, is_patched, is_patch_file) {
            result = FieldResult::Reset;
        }
    });
    result
}

/// Render a boolean field with patch awareness
fn render_bool_field(
    ui: &mut egui::Ui,
    label: &str,
    _key: &str,
    current_value: bool,
    is_patched: bool,
    is_patch_file: bool,
) -> FieldResult<bool> {
    let mut result = FieldResult::NoChange;
    ui.horizontal(|ui| {
        render_patch_indicator(ui, is_patched, is_patch_file);

        let text_color = if is_patch_file && !is_patched {
            INHERITED_COLOR
        } else {
            ui.style().visuals.text_color()
        };

        ui.label(egui::RichText::new(label).color(text_color));

        let mut value = current_value;
        if ui.checkbox(&mut value, "").changed() {
            result = FieldResult::Changed(value);
        }

        if render_reset_button(ui, is_patched, is_patch_file) {
            result = FieldResult::Reset;
        }
    });
    result
}

/// Render an integer field with patch awareness
fn render_int_field(
    ui: &mut egui::Ui,
    label: &str,
    _key: &str,
    current_value: i32,
    range: std::ops::RangeInclusive<i32>,
    is_patched: bool,
    is_patch_file: bool,
) -> FieldResult<i32> {
    let mut result = FieldResult::NoChange;
    ui.horizontal(|ui| {
        render_patch_indicator(ui, is_patched, is_patch_file);

        let text_color = if is_patch_file && !is_patched {
            INHERITED_COLOR
        } else {
            ui.style().visuals.text_color()
        };

        ui.label(egui::RichText::new(label).color(text_color));

        let mut value = current_value;
        if ui.add(egui::DragValue::new(&mut value).range(range)).changed() {
            result = FieldResult::Changed(value);
        }

        if render_reset_button(ui, is_patched, is_patch_file) {
            result = FieldResult::Reset;
        }
    });
    result
}

/// Render a float field with patch awareness
fn render_float_field(
    ui: &mut egui::Ui,
    label: &str,
    _key: &str,
    current_value: f32,
    range: std::ops::RangeInclusive<f32>,
    is_patched: bool,
    is_patch_file: bool,
) -> FieldResult<f32> {
    let mut result = FieldResult::NoChange;
    ui.horizontal(|ui| {
        render_patch_indicator(ui, is_patched, is_patch_file);

        let text_color = if is_patch_file && !is_patched {
            INHERITED_COLOR
        } else {
            ui.style().visuals.text_color()
        };

        ui.label(egui::RichText::new(label).color(text_color));

        let mut value = current_value;
        if ui.add(egui::DragValue::new(&mut value).range(range)).changed() {
            result = FieldResult::Changed(value);
        }

        if render_reset_button(ui, is_patched, is_patch_file) {
            result = FieldResult::Reset;
        }
    });
    result
}

/// Render a float field with slower drag speed
fn render_float_field_slow(
    ui: &mut egui::Ui,
    label: &str,
    _key: &str,
    current_value: f32,
    range: std::ops::RangeInclusive<f32>,
    speed: f64,
    is_patched: bool,
    is_patch_file: bool,
) -> FieldResult<f32> {
    let mut result = FieldResult::NoChange;
    ui.horizontal(|ui| {
        render_patch_indicator(ui, is_patched, is_patch_file);

        let text_color = if is_patch_file && !is_patched {
            INHERITED_COLOR
        } else {
            ui.style().visuals.text_color()
        };

        ui.label(egui::RichText::new(label).color(text_color));

        let mut value = current_value;
        if ui.add(egui::DragValue::new(&mut value).range(range).speed(speed)).changed() {
            result = FieldResult::Changed(value);
        }

        if render_reset_button(ui, is_patched, is_patch_file) {
            result = FieldResult::Reset;
        }
    });
    result
}

/// Render a Vec2 field with patch awareness
fn render_vec2_field(
    ui: &mut egui::Ui,
    label: &str,
    _key: &str,
    x: f32,
    y: f32,
    range: std::ops::RangeInclusive<f32>,
    is_patched: bool,
    is_patch_file: bool,
) -> FieldResult<(f32, f32)> {
    let mut result = FieldResult::NoChange;

    ui.horizontal(|ui| {
        render_patch_indicator(ui, is_patched, is_patch_file);

        let text_color = if is_patch_file && !is_patched {
            INHERITED_COLOR
        } else {
            ui.style().visuals.text_color()
        };

        ui.label(egui::RichText::new(label).color(text_color));

        if render_reset_button(ui, is_patched, is_patch_file) {
            result = FieldResult::Reset;
        }
    });

    if !matches!(result, FieldResult::Reset) {
        ui.horizontal(|ui| {
            ui.add_space(16.0); // Indent
            ui.label("X:");
            let mut new_x = x;
            let x_changed = ui.add(egui::DragValue::new(&mut new_x).range(range.clone())).changed();
            ui.label("Y:");
            let mut new_y = y;
            let y_changed = ui.add(egui::DragValue::new(&mut new_y).range(range)).changed();

            if x_changed || y_changed {
                result = FieldResult::Changed((new_x, new_y));
            }
        });
    }

    result
}

/// Render a Vec2 field with slower drag speed
fn render_vec2_field_slow(
    ui: &mut egui::Ui,
    label: &str,
    _key: &str,
    x: f32,
    y: f32,
    range: std::ops::RangeInclusive<f32>,
    speed: f64,
    is_patched: bool,
    is_patch_file: bool,
) -> FieldResult<(f32, f32)> {
    let mut result = FieldResult::NoChange;

    ui.horizontal(|ui| {
        render_patch_indicator(ui, is_patched, is_patch_file);

        let text_color = if is_patch_file && !is_patched {
            INHERITED_COLOR
        } else {
            ui.style().visuals.text_color()
        };

        ui.label(egui::RichText::new(label).color(text_color));

        if render_reset_button(ui, is_patched, is_patch_file) {
            result = FieldResult::Reset;
        }
    });

    if !matches!(result, FieldResult::Reset) {
        ui.horizontal(|ui| {
            ui.add_space(16.0);
            ui.label("X:");
            let mut new_x = x;
            let x_changed = ui.add(egui::DragValue::new(&mut new_x).range(range.clone()).speed(speed)).changed();
            ui.label("Y:");
            let mut new_y = y;
            let y_changed = ui.add(egui::DragValue::new(&mut new_y).range(range).speed(speed)).changed();

            if x_changed || y_changed {
                result = FieldResult::Changed((new_x, new_y));
            }
        });
    }

    result
}

/// Helper to get a Vec2 value from TOML
fn get_vec2_value(
    table: &toml::value::Table,
    key: &str,
    default_x: f32,
    default_y: f32,
) -> (f32, f32) {
    if let Some(arr) = table.get(key).and_then(|v| v.as_array()) {
        let x = arr.first().and_then(|v| v.as_float()).unwrap_or(default_x as f64) as f32;
        let y = arr.get(1).and_then(|v| v.as_float()).unwrap_or(default_y as f64) as f32;
        (x, y)
    } else {
        (default_x, default_y)
    }
}

/// Helper to set a Vec2 value in TOML
fn set_vec2_value(table: &mut toml::value::Table, key: &str, x: f32, y: f32) {
    table.insert(
        key.to_string(),
        toml::Value::Array(vec![
            toml::Value::Float(x as f64),
            toml::Value::Float(y as f64),
        ]),
    );
}

/// Render the colliders section
fn render_colliders_section(
    ui: &mut egui::Ui,
    display_table: &toml::value::Table,
    patch_table: &toml::value::Table,
    session: &mut EditorSession,
    is_patch: bool,
    modified: &mut bool,
) {
    let is_patched = is_patch && patch_table.contains_key("colliders");

    egui::CollapsingHeader::new("Colliders")
        .default_open(false)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                render_patch_indicator(ui, is_patched, is_patch);
                if is_patch && !is_patched {
                    ui.label(egui::RichText::new("(inherited from base)").small().color(INHERITED_COLOR));
                    // Add "Override" button to copy colliders to patch
                    if ui.button("Override").clicked() {
                        if let Some(colliders) = display_table.get("colliders").cloned() {
                            if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                                mob.insert("colliders".to_string(), colliders);
                                *modified = true;
                            }
                        }
                    }
                } else if is_patch && is_patched {
                    ui.label(egui::RichText::new("(overriding base)").small().color(PATCHED_COLOR));
                    // Add "Reset" button to remove colliders from patch
                    if ui.button("Reset to base").clicked() {
                        if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                            mob.remove("colliders");
                            *modified = true;
                        }
                    }
                }
            });

            let colliders = display_table
                .get("colliders")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();

            if colliders.is_empty() {
                ui.label("No colliders defined");
            } else {
                // Only allow editing if not a patch OR if colliders are in the patch
                let can_edit = !is_patch || is_patched;
                let mut delete_index: Option<usize> = None;

                for (i, collider) in colliders.iter().enumerate() {
                    egui::CollapsingHeader::new(format!("Collider {}", i + 1))
                        .id_salt(format!("collider_{}", i))
                        .default_open(false)
                        .show(ui, |ui| {
                            // Delete button at top of collider
                            if can_edit {
                                ui.horizontal(|ui| {
                                    if ui.add(egui::Button::new("🗑 Delete").fill(egui::Color32::from_rgb(120, 60, 60))).clicked() {
                                        delete_index = Some(i);
                                    }
                                });
                            }
                            if let Some(table) = collider.as_table() {
                                // Shape info (read-only - changing shape type is complex)
                                if let Some(shape) = table.get("shape").and_then(|s| s.as_table()) {
                                    if let Some(radius) = shape.get("Circle").and_then(|v| v.as_float()) {
                                        ui.label(format!("Shape: Circle"));
                                        if can_edit {
                                            let mut r = radius as f32;
                                            ui.horizontal(|ui| {
                                                ui.label("Radius:");
                                                if ui.add(egui::DragValue::new(&mut r).range(1.0..=100.0).speed(0.5)).changed() {
                                                    update_collider_circle_radius(session, i, r as f64);
                                                    *modified = true;
                                                }
                                            });
                                        } else {
                                            ui.label(format!("Radius: {}", radius));
                                        }
                                    } else if let Some(dims) = shape.get("Rectangle").and_then(|v| v.as_array()) {
                                        let w = dims.first().and_then(|v| v.as_float()).unwrap_or(10.0);
                                        let h = dims.get(1).and_then(|v| v.as_float()).unwrap_or(10.0);
                                        ui.label("Shape: Rectangle");
                                        if can_edit {
                                            let mut width = w as f32;
                                            let mut height = h as f32;
                                            ui.horizontal(|ui| {
                                                ui.label("W:");
                                                let w_changed = ui.add(egui::DragValue::new(&mut width).range(1.0..=200.0).speed(0.5)).changed();
                                                ui.label("H:");
                                                let h_changed = ui.add(egui::DragValue::new(&mut height).range(1.0..=200.0).speed(0.5)).changed();
                                                if w_changed || h_changed {
                                                    update_collider_rectangle_dims(session, i, width as f64, height as f64);
                                                    *modified = true;
                                                }
                                            });
                                        } else {
                                            ui.label(format!("Size: {} x {}", w, h));
                                        }
                                    }
                                }

                                // Position
                                let pos = table.get("position").and_then(|v| v.as_array());
                                let pos_x = pos.and_then(|a| a.first()).and_then(|v| v.as_float()).unwrap_or(0.0);
                                let pos_y = pos.and_then(|a| a.get(1)).and_then(|v| v.as_float()).unwrap_or(0.0);
                                if can_edit {
                                    let mut x = pos_x as f32;
                                    let mut y = pos_y as f32;
                                    ui.horizontal(|ui| {
                                        ui.label("Position X:");
                                        let x_changed = ui.add(egui::DragValue::new(&mut x).range(-100.0..=100.0).speed(0.5)).changed();
                                        ui.label("Y:");
                                        let y_changed = ui.add(egui::DragValue::new(&mut y).range(-100.0..=100.0).speed(0.5)).changed();
                                        if x_changed || y_changed {
                                            update_collider_position(session, i, x as f64, y as f64);
                                            *modified = true;
                                        }
                                    });
                                } else {
                                    ui.label(format!("Position: ({}, {})", pos_x, pos_y));
                                }

                                // Rotation
                                let rot = table.get("rotation").and_then(|v| v.as_float()).unwrap_or(0.0);
                                if can_edit {
                                    let mut r = rot as f32;
                                    ui.horizontal(|ui| {
                                        ui.label("Rotation:");
                                        if ui.add(egui::DragValue::new(&mut r).range(-180.0..=180.0).speed(1.0).suffix("°")).changed() {
                                            update_collider_rotation(session, i, r as f64);
                                            *modified = true;
                                        }
                                    });
                                } else {
                                    ui.label(format!("Rotation: {}°", rot));
                                }
                            }
                        });
                }

                // Handle deletion after the loop
                if let Some(idx) = delete_index {
                    delete_collider(session, idx);
                    *modified = true;
                }
            }

            // Add new collider button (only when can_edit is true)
            let can_add = !is_patch || is_patched;
            if can_add {
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("+ Add Rectangle").clicked() {
                        add_new_collider(session, "Rectangle");
                        *modified = true;
                    }
                    if ui.button("+ Add Circle").clicked() {
                        add_new_collider(session, "Circle");
                        *modified = true;
                    }
                });
            }
        });
}

/// Helper to update collider circle radius
fn update_collider_circle_radius(session: &mut EditorSession, index: usize, radius: f64) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        if let Some(colliders) = mob.get_mut("colliders").and_then(|v| v.as_array_mut()) {
            if let Some(collider) = colliders.get_mut(index).and_then(|v| v.as_table_mut()) {
                if let Some(shape) = collider.get_mut("shape").and_then(|v| v.as_table_mut()) {
                    shape.insert("Circle".to_string(), toml::Value::Float(radius));
                }
            }
        }
    }
}

/// Helper to update collider rectangle dimensions
fn update_collider_rectangle_dims(session: &mut EditorSession, index: usize, width: f64, height: f64) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        if let Some(colliders) = mob.get_mut("colliders").and_then(|v| v.as_array_mut()) {
            if let Some(collider) = colliders.get_mut(index).and_then(|v| v.as_table_mut()) {
                if let Some(shape) = collider.get_mut("shape").and_then(|v| v.as_table_mut()) {
                    shape.insert("Rectangle".to_string(), toml::Value::Array(vec![
                        toml::Value::Float(width),
                        toml::Value::Float(height),
                    ]));
                }
            }
        }
    }
}

/// Helper to update collider position
fn update_collider_position(session: &mut EditorSession, index: usize, x: f64, y: f64) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        if let Some(colliders) = mob.get_mut("colliders").and_then(|v| v.as_array_mut()) {
            if let Some(collider) = colliders.get_mut(index).and_then(|v| v.as_table_mut()) {
                collider.insert("position".to_string(), toml::Value::Array(vec![
                    toml::Value::Float(x),
                    toml::Value::Float(y),
                ]));
            }
        }
    }
}

/// Helper to update collider rotation
fn update_collider_rotation(session: &mut EditorSession, index: usize, rotation: f64) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        if let Some(colliders) = mob.get_mut("colliders").and_then(|v| v.as_array_mut()) {
            if let Some(collider) = colliders.get_mut(index).and_then(|v| v.as_table_mut()) {
                collider.insert("rotation".to_string(), toml::Value::Float(rotation));
            }
        }
    }
}

/// Add a new collider to the mob
fn add_new_collider(session: &mut EditorSession, shape_type: &str) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        // Ensure colliders array exists
        if !mob.contains_key("colliders") {
            mob.insert("colliders".to_string(), toml::Value::Array(vec![]));
        }

        if let Some(colliders) = mob.get_mut("colliders").and_then(|v| v.as_array_mut()) {
            let mut collider = toml::value::Table::new();

            // Create shape
            let mut shape = toml::value::Table::new();
            match shape_type {
                "Circle" => {
                    shape.insert("Circle".to_string(), toml::Value::Float(10.0));
                }
                _ => { // Rectangle
                    shape.insert("Rectangle".to_string(), toml::Value::Array(vec![
                        toml::Value::Float(20.0),
                        toml::Value::Float(20.0),
                    ]));
                }
            }
            collider.insert("shape".to_string(), toml::Value::Table(shape));

            // Default position and rotation
            collider.insert("position".to_string(), toml::Value::Array(vec![
                toml::Value::Float(0.0),
                toml::Value::Float(0.0),
            ]));
            collider.insert("rotation".to_string(), toml::Value::Float(0.0));

            colliders.push(toml::Value::Table(collider));
        }
    }
}

/// Delete a collider by index
fn delete_collider(session: &mut EditorSession, index: usize) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        if let Some(colliders) = mob.get_mut("colliders").and_then(|v| v.as_array_mut()) {
            if index < colliders.len() {
                colliders.remove(index);
            }
        }
        // If colliders array is now empty, remove it entirely
        // This is important for patches - empty array overrides base, removing inherits from base
        if mob.get("colliders").and_then(|v| v.as_array()).map(|a| a.is_empty()).unwrap_or(false) {
            mob.remove("colliders");
        }
    }
}

/// Helper to check if a specific spawner field is patched
fn is_spawner_field_patched(patch_table: &toml::value::Table, spawner_type: &str, spawner_name: &str, field: &str) -> bool {
    patch_table
        .get(spawner_type)
        .and_then(|v| v.as_table())
        .and_then(|t| t.get("spawners"))
        .and_then(|v| v.as_table())
        .and_then(|s| s.get(spawner_name))
        .and_then(|v| v.as_table())
        .map(|t| t.contains_key(field))
        .unwrap_or(false)
}

/// Helper to set a spawner field value in the patch
fn set_spawner_field(session: &mut EditorSession, spawner_type: &str, spawner_name: &str, field: &str, value: toml::Value) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        // Ensure projectile_spawners exists
        if !mob.contains_key(spawner_type) {
            mob.insert(spawner_type.to_string(), toml::Value::Table(toml::value::Table::new()));
        }
        let spawners_section = mob.get_mut(spawner_type).unwrap().as_table_mut().unwrap();

        // Ensure spawners exists
        if !spawners_section.contains_key("spawners") {
            spawners_section.insert("spawners".to_string(), toml::Value::Table(toml::value::Table::new()));
        }
        let spawners = spawners_section.get_mut("spawners").unwrap().as_table_mut().unwrap();

        // Ensure this specific spawner exists
        if !spawners.contains_key(spawner_name) {
            spawners.insert(spawner_name.to_string(), toml::Value::Table(toml::value::Table::new()));
        }
        let spawner = spawners.get_mut(spawner_name).unwrap().as_table_mut().unwrap();

        // Set the field
        spawner.insert(field.to_string(), value);
    }
}

/// Helper to remove a spawner field from the patch
fn remove_spawner_field(session: &mut EditorSession, spawner_type: &str, spawner_name: &str, field: &str) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        if let Some(spawners_section) = mob.get_mut(spawner_type).and_then(|v| v.as_table_mut()) {
            if let Some(spawners) = spawners_section.get_mut("spawners").and_then(|v| v.as_table_mut()) {
                if let Some(spawner) = spawners.get_mut(spawner_name).and_then(|v| v.as_table_mut()) {
                    spawner.remove(field);

                    // Clean up empty tables
                    if spawner.is_empty() {
                        spawners.remove(spawner_name);
                    }
                }
                if spawners.is_empty() {
                    spawners_section.remove("spawners");
                }
            }
            if spawners_section.is_empty() {
                mob.remove(spawner_type);
            }
        }
    }
}

/// Render the projectile spawners section
fn render_projectile_spawners_section(
    ui: &mut egui::Ui,
    display_table: &toml::value::Table,
    patch_table: &toml::value::Table,
    session: &mut EditorSession,
    is_patch: bool,
    modified: &mut bool,
) {
    let is_patched = is_patch && patch_table.contains_key("projectile_spawners");

    egui::CollapsingHeader::new("Projectile Spawners")
        .default_open(false)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                render_patch_indicator(ui, is_patched, is_patch);
                if is_patch && !is_patched {
                    ui.label(egui::RichText::new("(inherited from base)").small().color(INHERITED_COLOR));
                } else if is_patch && is_patched {
                    ui.label(egui::RichText::new("(overriding base)").small().color(PATCHED_COLOR));
                }
            });

            let mut delete_spawner: Option<String> = None;
            let mut rename_spawner: Option<(String, String)> = None;

            if let Some(proj_spawners) = display_table.get("projectile_spawners").and_then(|v| v.as_table()) {
                if let Some(spawners) = proj_spawners.get("spawners").and_then(|v| v.as_table()) {
                    let mut keys: Vec<_> = spawners.keys().cloned().collect();
                    keys.sort();

                    for key in keys {
                        if let Some(spawner) = spawners.get(&key).and_then(|v| v.as_table()) {
                            let spawner_key = key.clone();

                            egui::CollapsingHeader::new(format!("Spawner: {}", key))
                                .id_salt(format!("proj_spawner_{}", key))
                                .default_open(false)
                                .show(ui, |ui| {
                                    // Name editing and delete button
                                    ui.horizontal(|ui| {
                                        ui.label("Name:");
                                        let mut name = spawner_key.clone();
                                        let response = ui.text_edit_singleline(&mut name);
                                        if response.lost_focus() && name != spawner_key && !name.is_empty() {
                                            rename_spawner = Some((spawner_key.clone(), name));
                                        }
                                        if ui.add(egui::Button::new("🗑").fill(egui::Color32::from_rgb(120, 60, 60))).on_hover_text("Delete spawner").clicked() {
                                            delete_spawner = Some(spawner_key.clone());
                                        }
                                    });
                                    ui.separator();

                                    // Timer
                                    let field_patched = is_spawner_field_patched(patch_table, "projectile_spawners", &spawner_key, "timer");
                                    let timer = spawner.get("timer").and_then(|v| v.as_float()).unwrap_or(1.0) as f32;
                                    match render_float_field_slow(ui, "Timer:", "timer", timer, 0.01..=10.0, 0.01, field_patched, is_patch) {
                                        FieldResult::Changed(new_val) => {
                                            set_spawner_field(session, "projectile_spawners", &spawner_key, "timer", toml::Value::Float(new_val as f64));
                                            *modified = true;
                                        }
                                        FieldResult::Reset => {
                                            remove_spawner_field(session, "projectile_spawners", &spawner_key, "timer");
                                            *modified = true;
                                        }
                                        FieldResult::NoChange => {}
                                    }

                                    // Position
                                    let field_patched = is_spawner_field_patched(patch_table, "projectile_spawners", &spawner_key, "position");
                                    let pos = spawner.get("position").and_then(|v| v.as_array());
                                    let pos_x = pos.and_then(|a| a.first()).and_then(|v| v.as_float()).unwrap_or(0.0) as f32;
                                    let pos_y = pos.and_then(|a| a.get(1)).and_then(|v| v.as_float()).unwrap_or(0.0) as f32;
                                    match render_vec2_field(ui, "Position:", "position", pos_x, pos_y, -100.0..=100.0, field_patched, is_patch) {
                                        FieldResult::Changed((x, y)) => {
                                            let arr = toml::Value::Array(vec![toml::Value::Float(x as f64), toml::Value::Float(y as f64)]);
                                            set_spawner_field(session, "projectile_spawners", &spawner_key, "position", arr);
                                            *modified = true;
                                        }
                                        FieldResult::Reset => {
                                            remove_spawner_field(session, "projectile_spawners", &spawner_key, "position");
                                            *modified = true;
                                        }
                                        FieldResult::NoChange => {}
                                    }

                                    // Rotation
                                    let field_patched = is_spawner_field_patched(patch_table, "projectile_spawners", &spawner_key, "rotation");
                                    let rot = spawner.get("rotation").and_then(|v| v.as_float()).unwrap_or(0.0) as f32;
                                    match render_float_field(ui, "Rotation:", "rotation", rot, -180.0..=180.0, field_patched, is_patch) {
                                        FieldResult::Changed(new_val) => {
                                            set_spawner_field(session, "projectile_spawners", &spawner_key, "rotation", toml::Value::Float(new_val as f64));
                                            *modified = true;
                                        }
                                        FieldResult::Reset => {
                                            remove_spawner_field(session, "projectile_spawners", &spawner_key, "rotation");
                                            *modified = true;
                                        }
                                        FieldResult::NoChange => {}
                                    }

                                    // Projectile Type (dropdown)
                                    let field_patched = is_spawner_field_patched(patch_table, "projectile_spawners", &spawner_key, "projectile_type");
                                    let proj_type = spawner.get("projectile_type").and_then(|v| v.as_str()).unwrap_or("Bullet");
                                    ui.horizontal(|ui| {
                                        render_patch_indicator(ui, field_patched, is_patch);
                                        let text_color = if is_patch && !field_patched { INHERITED_COLOR } else { ui.style().visuals.text_color() };
                                        ui.label(egui::RichText::new("Type:").color(text_color));
                                        let mut selected = proj_type.to_string();
                                        egui::ComboBox::from_id_salt(format!("proj_type_{}", spawner_key))
                                            .selected_text(&selected)
                                            .show_ui(ui, |ui| {
                                                for &ptype in PROJECTILE_TYPES {
                                                    if ui.selectable_label(selected == ptype, ptype).clicked() {
                                                        selected = ptype.to_string();
                                                    }
                                                }
                                            });
                                        if selected != proj_type {
                                            set_spawner_field(session, "projectile_spawners", &spawner_key, "projectile_type", toml::Value::String(selected));
                                            *modified = true;
                                        }
                                        if render_reset_button(ui, field_patched, is_patch) {
                                            remove_spawner_field(session, "projectile_spawners", &spawner_key, "projectile_type");
                                            *modified = true;
                                        }
                                    });

                                    // Faction (dropdown)
                                    let field_patched = is_spawner_field_patched(patch_table, "projectile_spawners", &spawner_key, "faction");
                                    let faction = spawner.get("faction").and_then(|v| v.as_str()).unwrap_or("Enemy");
                                    ui.horizontal(|ui| {
                                        render_patch_indicator(ui, field_patched, is_patch);
                                        let text_color = if is_patch && !field_patched { INHERITED_COLOR } else { ui.style().visuals.text_color() };
                                        ui.label(egui::RichText::new("Faction:").color(text_color));
                                        let mut selected = faction.to_string();
                                        egui::ComboBox::from_id_salt(format!("proj_faction_{}", spawner_key))
                                            .selected_text(&selected)
                                            .show_ui(ui, |ui| {
                                                for &f in FACTIONS {
                                                    if ui.selectable_label(selected == f, f).clicked() {
                                                        selected = f.to_string();
                                                    }
                                                }
                                            });
                                        if selected != faction {
                                            set_spawner_field(session, "projectile_spawners", &spawner_key, "faction", toml::Value::String(selected));
                                            *modified = true;
                                        }
                                        if render_reset_button(ui, field_patched, is_patch) {
                                            remove_spawner_field(session, "projectile_spawners", &spawner_key, "faction");
                                            *modified = true;
                                        }
                                    });
                                });
                        }
                    }
                }
            } else {
                ui.label("No projectile spawners");
            }

            // Handle delete and rename after the loop
            if let Some(name) = delete_spawner {
                delete_spawner_by_name(session, "projectile_spawners", &name);
                *modified = true;
            }
            if let Some((old_name, new_name)) = rename_spawner {
                rename_spawner_by_name(session, "projectile_spawners", &old_name, &new_name);
                *modified = true;
            }

            // Add new spawner button
            ui.separator();
            if ui.button("+ Add Spawner").clicked() {
                add_new_projectile_spawner(session, display_table);
                *modified = true;
            }
        });
}

/// Render the mob spawners section
fn render_mob_spawners_section(
    ui: &mut egui::Ui,
    display_table: &toml::value::Table,
    patch_table: &toml::value::Table,
    session: &mut EditorSession,
    is_patch: bool,
    modified: &mut bool,
) {
    let is_patched = is_patch && patch_table.contains_key("mob_spawners");

    egui::CollapsingHeader::new("Mob Spawners")
        .default_open(false)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                render_patch_indicator(ui, is_patched, is_patch);
                if is_patch && !is_patched {
                    ui.label(egui::RichText::new("(inherited from base)").small().color(INHERITED_COLOR));
                } else if is_patch && is_patched {
                    ui.label(egui::RichText::new("(overriding base)").small().color(PATCHED_COLOR));
                }
            });

            let mut delete_spawner: Option<String> = None;
            let mut rename_spawner: Option<(String, String)> = None;

            if let Some(mob_spawners) = display_table.get("mob_spawners").and_then(|v| v.as_table()) {
                if let Some(spawners) = mob_spawners.get("spawners").and_then(|v| v.as_table()) {
                    let mut keys: Vec<_> = spawners.keys().cloned().collect();
                    keys.sort();

                    for key in keys {
                        if let Some(spawner) = spawners.get(&key).and_then(|v| v.as_table()) {
                            let spawner_key = key.clone();

                            egui::CollapsingHeader::new(format!("Spawner: {}", key))
                                .id_salt(format!("mob_spawner_{}", key))
                                .default_open(false)
                                .show(ui, |ui| {
                                    // Name editing and delete button
                                    ui.horizontal(|ui| {
                                        ui.label("Name:");
                                        let mut name = spawner_key.clone();
                                        let response = ui.text_edit_singleline(&mut name);
                                        if response.lost_focus() && name != spawner_key && !name.is_empty() {
                                            rename_spawner = Some((spawner_key.clone(), name));
                                        }
                                        if ui.add(egui::Button::new("🗑").fill(egui::Color32::from_rgb(120, 60, 60))).on_hover_text("Delete spawner").clicked() {
                                            delete_spawner = Some(spawner_key.clone());
                                        }
                                    });
                                    ui.separator();

                                    // Timer
                                    let field_patched = is_spawner_field_patched(patch_table, "mob_spawners", &spawner_key, "timer");
                                    let timer = spawner.get("timer").and_then(|v| v.as_float()).unwrap_or(1.0) as f32;
                                    match render_float_field_slow(ui, "Timer:", "timer", timer, 0.01..=60.0, 0.1, field_patched, is_patch) {
                                        FieldResult::Changed(new_val) => {
                                            set_spawner_field(session, "mob_spawners", &spawner_key, "timer", toml::Value::Float(new_val as f64));
                                            *modified = true;
                                        }
                                        FieldResult::Reset => {
                                            remove_spawner_field(session, "mob_spawners", &spawner_key, "timer");
                                            *modified = true;
                                        }
                                        FieldResult::NoChange => {}
                                    }

                                    // Position
                                    let field_patched = is_spawner_field_patched(patch_table, "mob_spawners", &spawner_key, "position");
                                    let pos = spawner.get("position").and_then(|v| v.as_array());
                                    let pos_x = pos.and_then(|a| a.first()).and_then(|v| v.as_float()).unwrap_or(0.0) as f32;
                                    let pos_y = pos.and_then(|a| a.get(1)).and_then(|v| v.as_float()).unwrap_or(0.0) as f32;
                                    match render_vec2_field(ui, "Position:", "position", pos_x, pos_y, -100.0..=100.0, field_patched, is_patch) {
                                        FieldResult::Changed((x, y)) => {
                                            let arr = toml::Value::Array(vec![toml::Value::Float(x as f64), toml::Value::Float(y as f64)]);
                                            set_spawner_field(session, "mob_spawners", &spawner_key, "position", arr);
                                            *modified = true;
                                        }
                                        FieldResult::Reset => {
                                            remove_spawner_field(session, "mob_spawners", &spawner_key, "position");
                                            *modified = true;
                                        }
                                        FieldResult::NoChange => {}
                                    }

                                    // Rotation
                                    let field_patched = is_spawner_field_patched(patch_table, "mob_spawners", &spawner_key, "rotation");
                                    let rot = spawner.get("rotation").and_then(|v| v.as_float()).unwrap_or(0.0) as f32;
                                    match render_float_field(ui, "Rotation:", "rotation", rot, -180.0..=180.0, field_patched, is_patch) {
                                        FieldResult::Changed(new_val) => {
                                            set_spawner_field(session, "mob_spawners", &spawner_key, "rotation", toml::Value::Float(new_val as f64));
                                            *modified = true;
                                        }
                                        FieldResult::Reset => {
                                            remove_spawner_field(session, "mob_spawners", &spawner_key, "rotation");
                                            *modified = true;
                                        }
                                        FieldResult::NoChange => {}
                                    }

                                    // Mob Ref
                                    let field_patched = is_spawner_field_patched(patch_table, "mob_spawners", &spawner_key, "mob_ref");
                                    let mob_ref = spawner.get("mob_ref").and_then(|v| v.as_str()).unwrap_or("");
                                    match render_string_field(ui, "Mob Ref:", "mob_ref", mob_ref, field_patched, is_patch) {
                                        FieldResult::Changed(new_val) => {
                                            set_spawner_field(session, "mob_spawners", &spawner_key, "mob_ref", toml::Value::String(new_val));
                                            *modified = true;
                                        }
                                        FieldResult::Reset => {
                                            remove_spawner_field(session, "mob_spawners", &spawner_key, "mob_ref");
                                            *modified = true;
                                        }
                                        FieldResult::NoChange => {}
                                    }
                                });
                        }
                    }
                }
            } else {
                ui.label("No mob spawners");
            }

            // Handle delete and rename after the loop
            if let Some(name) = delete_spawner {
                delete_spawner_by_name(session, "mob_spawners", &name);
                *modified = true;
            }
            if let Some((old_name, new_name)) = rename_spawner {
                rename_spawner_by_name(session, "mob_spawners", &old_name, &new_name);
                *modified = true;
            }

            // Add new mob spawner button
            ui.separator();
            if ui.button("+ Add Mob Spawner").clicked() {
                add_new_mob_spawner(session, display_table);
                *modified = true;
            }
        });
}

/// Generate a unique spawner name
fn generate_unique_spawner_name(existing_spawners: Option<&toml::value::Table>, prefix: &str) -> String {
    let directions = ["north", "south", "east", "west", "center"];
    for dir in directions {
        let name = format!("{}_{}", prefix, dir);
        if existing_spawners.map(|s| !s.contains_key(&name)).unwrap_or(true) {
            return name;
        }
    }
    // Fallback to numbered names
    for i in 1..100 {
        let name = format!("{}_{}", prefix, i);
        if existing_spawners.map(|s| !s.contains_key(&name)).unwrap_or(true) {
            return name;
        }
    }
    format!("{}_new", prefix)
}

/// Add a new projectile spawner
fn add_new_projectile_spawner(session: &mut EditorSession, display_table: &toml::value::Table) {
    let existing = display_table
        .get("projectile_spawners")
        .and_then(|v| v.as_table())
        .and_then(|t| t.get("spawners"))
        .and_then(|v| v.as_table());

    let name = generate_unique_spawner_name(existing, "spawner");

    // Create default spawner
    let mut spawner = toml::value::Table::new();
    spawner.insert("timer".to_string(), toml::Value::Float(1.0));
    spawner.insert("position".to_string(), toml::Value::Array(vec![
        toml::Value::Float(0.0),
        toml::Value::Float(0.0),
    ]));
    spawner.insert("rotation".to_string(), toml::Value::Float(0.0));
    spawner.insert("projectile_type".to_string(), toml::Value::String("Bullet".to_string()));
    spawner.insert("faction".to_string(), toml::Value::String("Enemy".to_string()));

    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        // Ensure projectile_spawners.spawners exists
        if !mob.contains_key("projectile_spawners") {
            mob.insert("projectile_spawners".to_string(), toml::Value::Table(toml::value::Table::new()));
        }
        let ps = mob.get_mut("projectile_spawners").unwrap().as_table_mut().unwrap();
        if !ps.contains_key("spawners") {
            ps.insert("spawners".to_string(), toml::Value::Table(toml::value::Table::new()));
        }
        let spawners = ps.get_mut("spawners").unwrap().as_table_mut().unwrap();
        spawners.insert(name, toml::Value::Table(spawner));
    }
}

/// Add a new mob spawner
fn add_new_mob_spawner(session: &mut EditorSession, display_table: &toml::value::Table) {
    let existing = display_table
        .get("mob_spawners")
        .and_then(|v| v.as_table())
        .and_then(|t| t.get("spawners"))
        .and_then(|v| v.as_table());

    let name = generate_unique_spawner_name(existing, "mob");

    // Create default mob spawner
    let mut spawner = toml::value::Table::new();
    spawner.insert("timer".to_string(), toml::Value::Float(5.0));
    spawner.insert("position".to_string(), toml::Value::Array(vec![
        toml::Value::Float(0.0),
        toml::Value::Float(0.0),
    ]));
    spawner.insert("rotation".to_string(), toml::Value::Float(0.0));
    spawner.insert("mob_ref".to_string(), toml::Value::String("".to_string()));

    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        // Ensure mob_spawners.spawners exists
        if !mob.contains_key("mob_spawners") {
            mob.insert("mob_spawners".to_string(), toml::Value::Table(toml::value::Table::new()));
        }
        let ms = mob.get_mut("mob_spawners").unwrap().as_table_mut().unwrap();
        if !ms.contains_key("spawners") {
            ms.insert("spawners".to_string(), toml::Value::Table(toml::value::Table::new()));
        }
        let spawners = ms.get_mut("spawners").unwrap().as_table_mut().unwrap();
        spawners.insert(name, toml::Value::Table(spawner));
    }
}

/// Delete a spawner by name
fn delete_spawner_by_name(session: &mut EditorSession, spawner_type: &str, name: &str) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        if let Some(spawners_section) = mob.get_mut(spawner_type).and_then(|v| v.as_table_mut()) {
            if let Some(spawners) = spawners_section.get_mut("spawners").and_then(|v| v.as_table_mut()) {
                spawners.remove(name);

                // Clean up empty tables
                if spawners.is_empty() {
                    spawners_section.remove("spawners");
                }
            }
            if spawners_section.is_empty() {
                mob.remove(spawner_type);
            }
        }
    }
}

/// Rename a spawner
fn rename_spawner_by_name(session: &mut EditorSession, spawner_type: &str, old_name: &str, new_name: &str) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        if let Some(spawners_section) = mob.get_mut(spawner_type).and_then(|v| v.as_table_mut()) {
            if let Some(spawners) = spawners_section.get_mut("spawners").and_then(|v| v.as_table_mut()) {
                // Check if new name already exists
                if spawners.contains_key(new_name) {
                    return; // Don't overwrite existing spawner
                }
                // Move the spawner data to the new key
                if let Some(data) = spawners.remove(old_name) {
                    spawners.insert(new_name.to_string(), data);
                }
            }
        }
    }
}

/// Render a sprite picker dropdown
/// Returns a BrowseResult if user clicked browse and selected a file
fn render_sprite_picker(
    ui: &mut egui::Ui,
    display_table: &toml::value::Table,
    patch_table: &toml::value::Table,
    session: &mut EditorSession,
    sprite_registry: &SpriteRegistry,
    is_patch: bool,
    modified: &mut bool,
) -> Option<BrowseResult> {
    let mut browse_result: Option<BrowseResult> = None;

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
                if session.can_use_extended_sprites() {
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
                                selected_path = if is_patch && sprite.source == SpriteSource::Extended {
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
            if let Some(mob) = session
                .current_mob
                .as_mut()
                .and_then(|v| v.as_table_mut())
            {
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
            if let Some(mob) = session
                .current_mob
                .as_mut()
                .and_then(|v| v.as_table_mut())
            {
                mob.remove("sprite");
                *modified = true;
            }
        }
    });

    // Browse & Register button row
    ui.horizontal(|ui| {
        ui.add_space(16.0);

        if ui.small_button("Browse & Register...").clicked() {
            // Open file dialog
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Aseprite Files", &["aseprite", "ase"])
                .set_title("Select Sprite to Register")
                .pick_file()
            {
                // Analyze the path to determine if it's valid
                let can_use_extended = session.can_use_extended_sprites();
                browse_result = Some(analyze_sprite_path(&path, is_patch, can_use_extended));
            }
        }

        ui.label(
            egui::RichText::new("Add new sprite to assets")
                .small()
                .color(egui::Color32::GRAY),
        );
    });

    // Show warning if unregistered
    if !sprite_registry.is_registered(current_sprite) && !current_sprite.is_empty() {
        ui.horizontal(|ui| {
            ui.add_space(16.0);
            ui.label(
                egui::RichText::new("⚠ Not in game.assets.ron")
                    .small()
                    .color(egui::Color32::YELLOW),
            );
        });
    }

    browse_result
}

/// Analyze a filesystem path to determine its asset path and whether it's base or extended
fn analyze_sprite_path(
    filesystem_path: &PathBuf,
    is_patch: bool,
    can_use_extended: bool,
) -> BrowseResult {
    let Some(cwd) = std::env::current_dir().ok() else {
        return BrowseResult::InvalidLocation(
            "Could not determine current directory".to_string()
        );
    };

    // Check if it's in extended assets directory
    let extended_assets_dir = cwd.join("thetawave-test-game/assets");
    let base_assets_dir = cwd.join("assets");

    if let Ok(relative) = filesystem_path.strip_prefix(&extended_assets_dir) {
        // Extended asset - check if allowed
        if !can_use_extended {
            return BrowseResult::InvalidLocation(
                "Extended sprites are only available for mobpatches and extended mobs. \
                 Base game mobs can only use sprites from 'assets/'."
                    .to_string(),
            );
        }

        let asset_path = relative.to_string_lossy().to_string();
        let mob_path = if is_patch {
            format!("extended://{}", asset_path)
        } else {
            asset_path.clone()
        };
        BrowseResult::Register(BrowseRegistrationRequest {
            asset_path,
            is_extended: true,
            mob_path,
        })
    } else if let Ok(relative) = filesystem_path.strip_prefix(&base_assets_dir) {
        // Base asset
        let asset_path = relative.to_string_lossy().to_string();
        BrowseResult::Register(BrowseRegistrationRequest {
            asset_path: asset_path.clone(),
            is_extended: false,
            mob_path: asset_path,
        })
    } else {
        // File is not in either assets directory
        BrowseResult::InvalidLocation(format!(
            "Sprite must be in 'assets/' or 'thetawave-test-game/assets/'. \
             Selected file is outside these directories: {}",
            filesystem_path.display()
        ))
    }
}

/// Render the decorations section with sprite pickers
/// Returns a DecorationBrowseResult if user clicked browse on a decoration
fn render_decorations_section(
    ui: &mut egui::Ui,
    display_table: &toml::value::Table,
    patch_table: &toml::value::Table,
    session: &mut EditorSession,
    sprite_registry: &SpriteRegistry,
    is_patch: bool,
    modified: &mut bool,
) -> Option<DecorationBrowseResult> {
    let mut browse_result: Option<DecorationBrowseResult> = None;
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
                            if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
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
                        if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
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
                    if let Some(result) = render_decoration_sprite_picker(
                        ui,
                        i,
                        sprite_path,
                        sprite_registry,
                        session,
                        is_patch,
                        can_edit,
                        modified,
                    ) {
                        browse_result = Some(result);
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

    browse_result
}

/// Render sprite picker for a decoration
/// Returns a DecorationBrowseResult if user clicked browse
fn render_decoration_sprite_picker(
    ui: &mut egui::Ui,
    index: usize,
    current_sprite: &str,
    sprite_registry: &SpriteRegistry,
    session: &mut EditorSession,
    is_patch: bool,
    can_edit: bool,
    modified: &mut bool,
) -> Option<DecorationBrowseResult> {
    let mut browse_result: Option<DecorationBrowseResult> = None;

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
                    if session.can_use_extended_sprites() {
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
                                    selected_path = if is_patch && sprite.source == SpriteSource::Extended {
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
                            ui.selectable_label(true, &sprite_registry.display_name_for(current_sprite));
                    }
                });

            // Apply change if different
            if selected_path != current_sprite {
                update_decoration_sprite(session, index, &selected_path);
                *modified = true;
            }
        } else {
            // Display-only mode when not editable
            ui.label(
                egui::RichText::new(&display_text)
                    .color(INHERITED_COLOR),
            );
        }
    });

    // Browse & Register button row (only when editable)
    if can_edit {
        ui.horizontal(|ui| {
            ui.add_space(16.0);

            if ui.small_button("Browse & Register...").clicked() {
                // Open file dialog
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Aseprite Files", &["aseprite", "ase"])
                    .set_title("Select Sprite to Register")
                    .pick_file()
                {
                    // Analyze the path to determine if it's valid
                    let can_use_extended = session.can_use_extended_sprites();
                    match analyze_sprite_path(&path, is_patch, can_use_extended) {
                        BrowseResult::Register(request) => {
                            browse_result = Some(DecorationBrowseResult::Register {
                                index,
                                request,
                            });
                        }
                        BrowseResult::InvalidLocation(message) => {
                            browse_result = Some(DecorationBrowseResult::InvalidLocation(message));
                        }
                    }
                }
            }
        });
    }

    // Show warning if unregistered
    if !sprite_registry.is_registered(current_sprite) && !current_sprite.is_empty() {
        ui.horizontal(|ui| {
            ui.add_space(16.0);
            ui.label(
                egui::RichText::new("⚠ Not registered")
                    .small()
                    .color(egui::Color32::YELLOW),
            );
        });
    }

    browse_result
}

/// Update a decoration's sprite path
pub fn update_decoration_sprite(session: &mut EditorSession, index: usize, sprite_path: &str) {
    if let Some(mob) = session
        .current_mob
        .as_mut()
        .and_then(|v| v.as_table_mut())
    {
        if let Some(decorations) = mob.get_mut("decorations").and_then(|v| v.as_array_mut()) {
            if let Some(decoration) = decorations.get_mut(index) {
                if let Some(arr) = decoration.as_array_mut() {
                    if !arr.is_empty() {
                        arr[0] = toml::Value::String(sprite_path.to_string());
                    }
                }
            }
        }
    }
}

/// Update a decoration's position
fn update_decoration_position(session: &mut EditorSession, index: usize, x: f32, y: f32) {
    if let Some(mob) = session
        .current_mob
        .as_mut()
        .and_then(|v| v.as_table_mut())
    {
        if let Some(decorations) = mob.get_mut("decorations").and_then(|v| v.as_array_mut()) {
            if let Some(decoration) = decorations.get_mut(index) {
                if let Some(arr) = decoration.as_array_mut() {
                    if arr.len() >= 2 {
                        arr[1] = toml::Value::Array(vec![
                            toml::Value::Float(x as f64),
                            toml::Value::Float(y as f64),
                        ]);
                    }
                }
            }
        }
    }
}

/// Delete a decoration at the given index
fn delete_decoration(session: &mut EditorSession, index: usize) {
    if let Some(mob) = session
        .current_mob
        .as_mut()
        .and_then(|v| v.as_table_mut())
    {
        if let Some(decorations) = mob.get_mut("decorations").and_then(|v| v.as_array_mut()) {
            if index < decorations.len() {
                decorations.remove(index);
            }
            // Remove empty decorations array
            if decorations.is_empty() {
                mob.remove("decorations");
            }
        }
    }
}

/// Add a new decoration with default values
fn add_new_decoration(session: &mut EditorSession, sprite_registry: &SpriteRegistry) {
    if let Some(mob) = session
        .current_mob
        .as_mut()
        .and_then(|v| v.as_table_mut())
    {
        // Get first available sprite as default, or empty string
        let default_sprite = sprite_registry
            .sprites
            .first()
            .map(|s| s.asset_path.clone())
            .unwrap_or_default();

        let new_decoration = toml::Value::Array(vec![
            toml::Value::String(default_sprite),
            toml::Value::Array(vec![toml::Value::Float(0.0), toml::Value::Float(0.0)]),
        ]);

        if let Some(decorations) = mob.get_mut("decorations").and_then(|v| v.as_array_mut()) {
            decorations.push(new_decoration);
        } else {
            mob.insert(
                "decorations".to_string(),
                toml::Value::Array(vec![new_decoration]),
            );
        }
    }
}
