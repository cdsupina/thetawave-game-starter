use bevy_egui::egui;

use crate::data::{EditorSession, FileType, SpriteRegistry, SpriteSource};
use crate::file::FileTreeState;
use crate::preview::JointedMobCache;

/// Color for patched/overridden values
const PATCHED_COLOR: egui::Color32 = egui::Color32::from_rgb(100, 200, 255);
/// Color for inherited values (from base mob)
const INHERITED_COLOR: egui::Color32 = egui::Color32::from_rgb(140, 140, 140);

/// Valid projectile types
const PROJECTILE_TYPES: &[&str] = &["Bullet", "Blast"];
/// Valid faction types
const FACTIONS: &[&str] = &["Enemy", "Ally"];

/// Results from the properties panel for sprite browser actions
#[derive(Debug, Clone, Default)]
pub struct PropertiesPanelResult {
    /// Request to open the sprite browser for main sprite
    pub open_sprite_browser: bool,
    /// Request to open the sprite browser for a decoration (by index)
    pub open_decoration_browser: Option<usize>,
}

/// Render the properties editing panel
/// Returns browse results if the user clicked browse to add a new sprite
pub fn properties_panel_ui(
    ui: &mut egui::Ui,
    session: &mut EditorSession,
    sprite_registry: &SpriteRegistry,
    jointed_cache: &JointedMobCache,
    file_tree: &FileTreeState,
    config: &crate::plugin::EditorConfig,
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
                        ui, "Name:",
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
                        ui, "Spawnable:",
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
                    if render_sprite_picker(
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
                });

            // Decorations section
            if let Some(idx) = render_decorations_section(
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

            // Combat section
            egui::CollapsingHeader::new("Combat")
                .default_open(true)
                .show(ui, |ui| {
                    // Health
                    let is_patched = is_patch && patch_table.contains_key("health");
                    match render_int_field(
                        ui, "Health:",
                        display_table.get("health").and_then(|v| v.as_integer()).unwrap_or(50) as i32,
                        1..=10000, is_patched, is_patch,
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
                        ui, "Proj Speed:",
                        display_table.get("projectile_speed").and_then(|v| v.as_float()).unwrap_or(100.0) as f32,
                        0.0..=1000.0, None, is_patched, is_patch,
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
                        ui, "Proj Damage:",
                        display_table.get("projectile_damage").and_then(|v| v.as_integer()).unwrap_or(5) as i32,
                        0..=1000, is_patched, is_patch,
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
                    match render_vec2_field(ui, "Max Speed:", speed.0, speed.1, 0.0..=500.0, None, is_patched, is_patch) {
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
                    match render_vec2_field(ui, "Acceleration:", accel.0, accel.1, 0.0..=10.0, Some(0.01), is_patched, is_patch) {
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
                    match render_vec2_field(ui, "Deceleration:", decel.0, decel.1, 0.0..=10.0, Some(0.01), is_patched, is_patch) {
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
                    match render_float_field(ui, "Angular Speed:", display_table.get("max_angular_speed").and_then(|v| v.as_float()).unwrap_or(1.0) as f32, 0.0..=20.0, Some(0.1), is_patched, is_patch) {
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
                    match render_float_field(ui, "Z Level:", display_table.get("z_level").and_then(|v| v.as_float()).unwrap_or(0.0) as f32, -10.0..=10.0, Some(0.1), is_patched, is_patch) {
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
                    match render_bool_field(ui, "Rot Locked:", display_table.get("rotation_locked").and_then(|v| v.as_bool()).unwrap_or(true), is_patched, is_patch) {
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
                    match render_float_field(ui, "Restitution:", display_table.get("restitution").and_then(|v| v.as_float()).unwrap_or(0.5) as f32, 0.0..=1.0, Some(0.01), is_patched, is_patch) {
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
                    match render_float_field(ui, "Friction:", display_table.get("friction").and_then(|v| v.as_float()).unwrap_or(0.5) as f32, 0.0..=2.0, Some(0.01), is_patched, is_patch) {
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
                    match render_float_field(ui, "Density:", display_table.get("collider_density").and_then(|v| v.as_float()).unwrap_or(1.0) as f32, 0.1..=10.0, Some(0.1), is_patched, is_patch) {
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

            // Jointed Mobs section
            render_jointed_mobs_section(ui, &display_table, &patch_table, session, is_patch, &mut modified, file_tree);

            // Behavior tree section
            render_behavior_section(ui, &display_table, &patch_table, session, is_patch, &mut modified);
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

    // Parent mobs list section (shows which mobs use this mob as a jointed part)
    ui.separator();
    egui::CollapsingHeader::new("Used By (Parent Mobs)")
        .default_open(false)
        .show(ui, |ui| {
            if jointed_cache.parent_mobs.is_empty() {
                ui.label(
                    egui::RichText::new("No parent mobs reference this file")
                        .italics()
                        .color(egui::Color32::GRAY),
                );
            } else {
                ui.label(
                    egui::RichText::new(format!(
                        "{} mob(s) use this file as a jointed part:",
                        jointed_cache.parent_mobs.len()
                    ))
                    .small()
                    .color(egui::Color32::GRAY),
                );
                ui.add_space(4.0);

                for parent in &jointed_cache.parent_mobs {
                    ui.horizontal(|ui| {
                        ui.label("•");
                        ui.label(egui::RichText::new(&parent.name).strong());
                        ui.label(
                            egui::RichText::new(format!("(as \"{}\")", parent.jointed_key))
                                .small()
                                .color(egui::Color32::GRAY),
                        );
                    });
                    ui.label(
                        egui::RichText::new(parent.path.display().to_string())
                            .small()
                            .monospace()
                            .color(egui::Color32::DARK_GRAY),
                    );
                    ui.add_space(4.0);
                }
            }
        });

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

/// Get label color based on patch state
fn label_color(ui: &egui::Ui, is_patch_file: bool, is_patched: bool) -> egui::Color32 {
    if is_patch_file && !is_patched {
        INHERITED_COLOR
    } else {
        ui.style().visuals.text_color()
    }
}

/// Render a string field with patch awareness
fn render_string_field(
    ui: &mut egui::Ui,
    label: &str,
    current_value: &str,
    is_patched: bool,
    is_patch_file: bool,
) -> FieldResult<String> {
    let mut result = FieldResult::NoChange;
    ui.horizontal(|ui| {
        render_patch_indicator(ui, is_patched, is_patch_file);
        ui.label(egui::RichText::new(label).color(label_color(ui, is_patch_file, is_patched)));

        let mut value = current_value.to_string();
        if ui.text_edit_singleline(&mut value).changed() {
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
    current_value: bool,
    is_patched: bool,
    is_patch_file: bool,
) -> FieldResult<bool> {
    let mut result = FieldResult::NoChange;
    ui.horizontal(|ui| {
        render_patch_indicator(ui, is_patched, is_patch_file);
        ui.label(egui::RichText::new(label).color(label_color(ui, is_patch_file, is_patched)));

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
    current_value: i32,
    range: std::ops::RangeInclusive<i32>,
    is_patched: bool,
    is_patch_file: bool,
) -> FieldResult<i32> {
    let mut result = FieldResult::NoChange;
    ui.horizontal(|ui| {
        render_patch_indicator(ui, is_patched, is_patch_file);
        ui.label(egui::RichText::new(label).color(label_color(ui, is_patch_file, is_patched)));

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

/// Render a float field with patch awareness (optional custom drag speed)
fn render_float_field(
    ui: &mut egui::Ui,
    label: &str,
    current_value: f32,
    range: std::ops::RangeInclusive<f32>,
    speed: Option<f64>,
    is_patched: bool,
    is_patch_file: bool,
) -> FieldResult<f32> {
    let mut result = FieldResult::NoChange;
    ui.horizontal(|ui| {
        render_patch_indicator(ui, is_patched, is_patch_file);
        ui.label(egui::RichText::new(label).color(label_color(ui, is_patch_file, is_patched)));

        let mut value = current_value;
        let mut drag = egui::DragValue::new(&mut value).range(range);
        if let Some(s) = speed {
            drag = drag.speed(s);
        }
        if ui.add(drag).changed() {
            result = FieldResult::Changed(value);
        }

        if render_reset_button(ui, is_patched, is_patch_file) {
            result = FieldResult::Reset;
        }
    });
    result
}

/// Render a Vec2 field with patch awareness (optional custom drag speed)
fn render_vec2_field(
    ui: &mut egui::Ui,
    label: &str,
    x: f32,
    y: f32,
    range: std::ops::RangeInclusive<f32>,
    speed: Option<f64>,
    is_patched: bool,
    is_patch_file: bool,
) -> FieldResult<(f32, f32)> {
    let mut result = FieldResult::NoChange;

    ui.horizontal(|ui| {
        render_patch_indicator(ui, is_patched, is_patch_file);
        ui.label(egui::RichText::new(label).color(label_color(ui, is_patch_file, is_patched)));

        if render_reset_button(ui, is_patched, is_patch_file) {
            result = FieldResult::Reset;
        }
    });

    if !matches!(result, FieldResult::Reset) {
        ui.horizontal(|ui| {
            ui.add_space(16.0);
            ui.label("X:");
            let mut new_x = x;
            let mut drag_x = egui::DragValue::new(&mut new_x).range(range.clone());
            if let Some(s) = speed {
                drag_x = drag_x.speed(s);
            }
            let x_changed = ui.add(drag_x).changed();

            ui.label("Y:");
            let mut new_y = y;
            let mut drag_y = egui::DragValue::new(&mut new_y).range(range);
            if let Some(s) = speed {
                drag_y = drag_y.speed(s);
            }
            let y_changed = ui.add(drag_y).changed();

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

/// Helper to mutate a collider at a specific index
fn with_collider_mut<F>(session: &mut EditorSession, index: usize, f: F)
where
    F: FnOnce(&mut toml::value::Table),
{
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        if let Some(colliders) = mob.get_mut("colliders").and_then(|v| v.as_array_mut()) {
            if let Some(collider) = colliders.get_mut(index).and_then(|v| v.as_table_mut()) {
                f(collider);
            }
        }
    }
}

/// Helper to mutate a collider's shape at a specific index
fn with_collider_shape_mut<F>(session: &mut EditorSession, index: usize, f: F)
where
    F: FnOnce(&mut toml::value::Table),
{
    with_collider_mut(session, index, |collider| {
        if let Some(shape) = collider.get_mut("shape").and_then(|v| v.as_table_mut()) {
            f(shape);
        }
    });
}

fn update_collider_circle_radius(session: &mut EditorSession, index: usize, radius: f64) {
    with_collider_shape_mut(session, index, |shape| {
        shape.insert("Circle".to_string(), toml::Value::Float(radius));
    });
}

fn update_collider_rectangle_dims(session: &mut EditorSession, index: usize, width: f64, height: f64) {
    with_collider_shape_mut(session, index, |shape| {
        shape.insert("Rectangle".to_string(), toml::Value::Array(vec![
            toml::Value::Float(width),
            toml::Value::Float(height),
        ]));
    });
}

fn update_collider_position(session: &mut EditorSession, index: usize, x: f64, y: f64) {
    with_collider_mut(session, index, |collider| {
        collider.insert("position".to_string(), toml::Value::Array(vec![
            toml::Value::Float(x),
            toml::Value::Float(y),
        ]));
    });
}

fn update_collider_rotation(session: &mut EditorSession, index: usize, rotation: f64) {
    with_collider_mut(session, index, |collider| {
        collider.insert("rotation".to_string(), toml::Value::Float(rotation));
    });
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
                                    match render_float_field(ui, "Timer:", timer, 0.01..=10.0, Some(0.01), field_patched, is_patch) {
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
                                    match render_vec2_field(ui, "Position:", pos_x, pos_y, -100.0..=100.0, None, field_patched, is_patch) {
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
                                    match render_float_field(ui, "Rotation:", rot, -180.0..=180.0, None, field_patched, is_patch) {
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
                                    match render_float_field(ui, "Timer:", timer, 0.01..=60.0, Some(0.1), field_patched, is_patch) {
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
                                    match render_vec2_field(ui, "Position:", pos_x, pos_y, -100.0..=100.0, None, field_patched, is_patch) {
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
                                    match render_float_field(ui, "Rotation:", rot, -180.0..=180.0, None, field_patched, is_patch) {
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
                                    match render_string_field(ui, "Mob Ref:", mob_ref, field_patched, is_patch) {
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

/// Render the jointed mobs section
fn render_jointed_mobs_section(
    ui: &mut egui::Ui,
    display_table: &toml::value::Table,
    patch_table: &toml::value::Table,
    session: &mut EditorSession,
    is_patch: bool,
    modified: &mut bool,
    file_tree: &FileTreeState,
) {
    let is_patched = is_patch && patch_table.contains_key("jointed_mobs");

    egui::CollapsingHeader::new("Jointed Mobs")
        .default_open(false)
        .show(ui, |ui| {
            // Patch status indicator
            ui.horizontal(|ui| {
                render_patch_indicator(ui, is_patched, is_patch);
                if is_patch && !is_patched {
                    ui.label(egui::RichText::new("(inherited from base)").small().color(INHERITED_COLOR));
                    if ui.button("Override").clicked() {
                        if let Some(jointed_mobs) = display_table.get("jointed_mobs").cloned() {
                            if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                                mob.insert("jointed_mobs".to_string(), jointed_mobs);
                                *modified = true;
                            }
                        }
                    }
                } else if is_patch && is_patched {
                    ui.label(egui::RichText::new("(overriding base)").small().color(PATCHED_COLOR));
                    if ui.button("Reset to base").clicked() {
                        if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                            mob.remove("jointed_mobs");
                            *modified = true;
                        }
                    }
                }
            });

            let jointed_mobs = display_table
                .get("jointed_mobs")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();

            if jointed_mobs.is_empty() {
                ui.label("No jointed mobs defined");
            } else {
                let can_edit = !is_patch || is_patched;
                let mut delete_index: Option<usize> = None;

                for (i, jointed) in jointed_mobs.iter().enumerate() {
                    let Some(table) = jointed.as_table() else {
                        continue;
                    };

                    let key = table.get("key").and_then(|v| v.as_str()).unwrap_or("unnamed");
                    let is_selected = session.selected_jointed_mob == Some(i);

                    // Highlight selected joint
                    let header_text = if is_selected {
                        egui::RichText::new(format!("Joint: {} *", key))
                            .strong()
                            .color(egui::Color32::YELLOW)
                    } else {
                        egui::RichText::new(format!("Joint: {}", key))
                    };

                    egui::CollapsingHeader::new(header_text)
                        .id_salt(format!("jointed_mob_{}", i))
                        .default_open(false)
                        .show(ui, |ui| {
                            // Select button
                            ui.horizontal(|ui| {
                                if ui.button(if is_selected { "Deselect" } else { "Select" }).clicked() {
                                    session.selected_jointed_mob = if is_selected { None } else { Some(i) };
                                }

                                if can_edit {
                                    if ui.add(egui::Button::new("🗑").fill(egui::Color32::from_rgb(120, 60, 60)))
                                        .on_hover_text("Delete joint")
                                        .clicked()
                                    {
                                        delete_index = Some(i);
                                    }
                                }
                            });
                            ui.separator();

                            if can_edit {
                                // Key field
                                let key_value = table.get("key").and_then(|v| v.as_str()).unwrap_or("");
                                ui.horizontal(|ui| {
                                    ui.label("Key:");
                                    let mut value = key_value.to_string();
                                    if ui.text_edit_singleline(&mut value).changed() {
                                        set_jointed_mob_field(session, i, "key", toml::Value::String(value));
                                        *modified = true;
                                    }
                                });

                                // Mob ref field (dropdown)
                                let mob_ref = table.get("mob_ref").and_then(|v| v.as_str()).unwrap_or("");
                                let mob_refs = file_tree.get_mob_refs();
                                ui.horizontal(|ui| {
                                    ui.label("Mob Ref:");
                                    let mut selected = mob_ref.to_string();
                                    egui::ComboBox::from_id_salt(format!("mob_ref_{}", i))
                                        .selected_text(if selected.is_empty() { "(none)" } else { &selected })
                                        .show_ui(ui, |ui| {
                                            for ref_name in &mob_refs {
                                                if ui.selectable_label(selected == *ref_name, ref_name).clicked() {
                                                    selected = ref_name.clone();
                                                }
                                            }
                                        });
                                    if selected != mob_ref {
                                        set_jointed_mob_field(session, i, "mob_ref", toml::Value::String(selected));
                                        *modified = true;
                                    }
                                });

                                // Offset position
                                let offset = get_jointed_vec2(table, "offset_pos");
                                ui.horizontal(|ui| {
                                    ui.label("Offset Pos:");
                                    let mut x = offset.0;
                                    let mut y = offset.1;
                                    let x_changed = ui.add(egui::DragValue::new(&mut x).speed(0.5).prefix("x: ")).changed();
                                    let y_changed = ui.add(egui::DragValue::new(&mut y).speed(0.5).prefix("y: ")).changed();
                                    if x_changed || y_changed {
                                        set_jointed_mob_vec2(session, i, "offset_pos", x, y);
                                        *modified = true;
                                    }
                                });

                                // Anchor 1 position
                                let anchor1 = get_jointed_vec2(table, "anchor_1_pos");
                                ui.horizontal(|ui| {
                                    ui.label("Anchor 1:");
                                    let mut x = anchor1.0;
                                    let mut y = anchor1.1;
                                    let x_changed = ui.add(egui::DragValue::new(&mut x).speed(0.5).prefix("x: ")).changed();
                                    let y_changed = ui.add(egui::DragValue::new(&mut y).speed(0.5).prefix("y: ")).changed();
                                    if x_changed || y_changed {
                                        set_jointed_mob_vec2(session, i, "anchor_1_pos", x, y);
                                        *modified = true;
                                    }
                                });

                                // Anchor 2 position
                                let anchor2 = get_jointed_vec2(table, "anchor_2_pos");
                                ui.horizontal(|ui| {
                                    ui.label("Anchor 2:");
                                    let mut x = anchor2.0;
                                    let mut y = anchor2.1;
                                    let x_changed = ui.add(egui::DragValue::new(&mut x).speed(0.5).prefix("x: ")).changed();
                                    let y_changed = ui.add(egui::DragValue::new(&mut y).speed(0.5).prefix("y: ")).changed();
                                    if x_changed || y_changed {
                                        set_jointed_mob_vec2(session, i, "anchor_2_pos", x, y);
                                        *modified = true;
                                    }
                                });

                                // Compliance (very small values)
                                let compliance = table.get("compliance")
                                    .and_then(|v| v.as_float())
                                    .unwrap_or(0.0) as f32;
                                ui.horizontal(|ui| {
                                    ui.label("Compliance:");
                                    let mut value = compliance;
                                    if ui.add(egui::DragValue::new(&mut value)
                                        .speed(0.0000001)
                                        .range(0.0..=1.0)
                                        .min_decimals(7)
                                        .max_decimals(10))
                                        .changed()
                                    {
                                        set_jointed_mob_field(session, i, "compliance", toml::Value::Float(value as f64));
                                        *modified = true;
                                    }
                                });

                                // Angle limits section
                                render_angle_limit_subsection(ui, session, i, table, modified);

                                // Chain configuration section
                                render_chain_subsection(ui, session, i, table, modified);
                            } else {
                                // Read-only display
                                ui.label(format!("Key: {}", key));
                                let mob_ref = table.get("mob_ref").and_then(|v| v.as_str()).unwrap_or("(none)");
                                ui.label(format!("Mob Ref: {}", mob_ref));
                                ui.colored_label(egui::Color32::GRAY, "(Override to edit)");
                            }
                        });
                }

                // Handle delete after the loop
                if let Some(idx) = delete_index {
                    delete_jointed_mob(session, idx);
                    if session.selected_jointed_mob == Some(idx) {
                        session.selected_jointed_mob = None;
                    } else if let Some(selected) = session.selected_jointed_mob {
                        if selected > idx {
                            session.selected_jointed_mob = Some(selected - 1);
                        }
                    }
                    *modified = true;
                }
            }

            // Add new jointed mob button
            let can_add = !is_patch || is_patched;
            if can_add {
                ui.separator();
                if ui.button("+ Add Jointed Mob").clicked() {
                    add_new_jointed_mob(session);
                    *modified = true;
                }
            }
        });
}

/// Render angle limit subsection for a jointed mob
fn render_angle_limit_subsection(
    ui: &mut egui::Ui,
    session: &mut EditorSession,
    index: usize,
    table: &toml::value::Table,
    modified: &mut bool,
) {
    let has_angle_limit = table.get("angle_limit_range").is_some();

    egui::CollapsingHeader::new("Angle Limits")
        .id_salt(format!("angle_limit_{}", index))
        .default_open(false)
        .show(ui, |ui| {
            if has_angle_limit {
                if let Some(angle_table) = table.get("angle_limit_range").and_then(|v| v.as_table()) {
                    let min = angle_table.get("min")
                        .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
                        .unwrap_or(-45.0) as f32;
                    let max = angle_table.get("max")
                        .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
                        .unwrap_or(45.0) as f32;
                    let torque = angle_table.get("torque")
                        .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
                        .unwrap_or(0.001) as f32;

                    ui.horizontal(|ui| {
                        ui.label("Min:");
                        let mut value = min;
                        if ui.add(egui::DragValue::new(&mut value).speed(1.0).range(-180.0..=180.0).suffix("°")).changed() {
                            set_jointed_nested_field(session, index, "angle_limit_range", "min", toml::Value::Float(value as f64));
                            *modified = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Max:");
                        let mut value = max;
                        if ui.add(egui::DragValue::new(&mut value).speed(1.0).range(-180.0..=180.0).suffix("°")).changed() {
                            set_jointed_nested_field(session, index, "angle_limit_range", "max", toml::Value::Float(value as f64));
                            *modified = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Torque:");
                        let mut value = torque;
                        if ui.add(egui::DragValue::new(&mut value).speed(0.0001).range(0.0..=1.0).min_decimals(4)).changed() {
                            set_jointed_nested_field(session, index, "angle_limit_range", "torque", toml::Value::Float(value as f64));
                            *modified = true;
                        }
                    });

                    if ui.button("Remove Angle Limits").clicked() {
                        remove_jointed_mob_field(session, index, "angle_limit_range");
                        *modified = true;
                    }
                }
            } else {
                ui.label(egui::RichText::new("No angle limits").italics().color(egui::Color32::GRAY));
                if ui.button("Add Angle Limits").clicked() {
                    add_angle_limit_to_jointed_mob(session, index);
                    *modified = true;
                }
            }
        });
}

/// Render chain configuration subsection for a jointed mob
fn render_chain_subsection(
    ui: &mut egui::Ui,
    session: &mut EditorSession,
    index: usize,
    table: &toml::value::Table,
    modified: &mut bool,
) {
    let has_chain = table.get("chain").is_some();

    egui::CollapsingHeader::new("Chain Configuration")
        .id_salt(format!("chain_{}", index))
        .default_open(false)
        .show(ui, |ui| {
            if has_chain {
                if let Some(chain_table) = table.get("chain").and_then(|v| v.as_table()) {
                    let length = chain_table.get("length")
                        .and_then(|v| v.as_integer())
                        .unwrap_or(1) as i32;
                    let pos_offset = get_jointed_vec2(chain_table, "pos_offset");
                    let anchor_offset = get_jointed_vec2(chain_table, "anchor_offset");

                    ui.horizontal(|ui| {
                        ui.label("Length:");
                        let mut value = length;
                        if ui.add(egui::DragValue::new(&mut value).range(1..=100)).changed() {
                            set_jointed_nested_field(session, index, "chain", "length", toml::Value::Integer(value as i64));
                            *modified = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Pos Offset:");
                        let mut x = pos_offset.0;
                        let mut y = pos_offset.1;
                        let x_changed = ui.add(egui::DragValue::new(&mut x).speed(0.5).prefix("x: ")).changed();
                        let y_changed = ui.add(egui::DragValue::new(&mut y).speed(0.5).prefix("y: ")).changed();
                        if x_changed || y_changed {
                            set_jointed_nested_vec2(session, index, "chain", "pos_offset", x, y);
                            *modified = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Anchor Offset:");
                        let mut x = anchor_offset.0;
                        let mut y = anchor_offset.1;
                        let x_changed = ui.add(egui::DragValue::new(&mut x).speed(0.5).prefix("x: ")).changed();
                        let y_changed = ui.add(egui::DragValue::new(&mut y).speed(0.5).prefix("y: ")).changed();
                        if x_changed || y_changed {
                            set_jointed_nested_vec2(session, index, "chain", "anchor_offset", x, y);
                            *modified = true;
                        }
                    });

                    // Random chain subsection
                    ui.separator();
                    let has_random = chain_table.get("random_chain").is_some();
                    if has_random {
                        if let Some(random_table) = chain_table.get("random_chain").and_then(|v| v.as_table()) {
                            ui.label(egui::RichText::new("Random Chain:").strong());

                            let min_length = random_table.get("min_length")
                                .and_then(|v| v.as_integer())
                                .unwrap_or(1) as i32;
                            let end_chance = random_table.get("end_chance")
                                .and_then(|v| v.as_float())
                                .unwrap_or(0.1) as f32;

                            ui.horizontal(|ui| {
                                ui.label("Min Length:");
                                let mut value = min_length;
                                if ui.add(egui::DragValue::new(&mut value).range(1..=100)).changed() {
                                    set_jointed_deep_nested_field(session, index, "chain", "random_chain", "min_length", toml::Value::Integer(value as i64));
                                    *modified = true;
                                }
                            });

                            ui.horizontal(|ui| {
                                ui.label("End Chance:");
                                let mut value = end_chance;
                                if ui.add(egui::DragValue::new(&mut value).speed(0.01).range(0.0..=1.0).min_decimals(2)).changed() {
                                    set_jointed_deep_nested_field(session, index, "chain", "random_chain", "end_chance", toml::Value::Float(value as f64));
                                    *modified = true;
                                }
                            });

                            if ui.button("Remove Random").clicked() {
                                remove_jointed_nested_field(session, index, "chain", "random_chain");
                                *modified = true;
                            }
                        }
                    } else {
                        ui.label(egui::RichText::new("Random: Not configured").small().color(egui::Color32::GRAY));
                        if ui.button("Add Random Chain").clicked() {
                            add_random_chain_to_jointed_mob(session, index);
                            *modified = true;
                        }
                    }

                    ui.separator();
                    if ui.button("Remove Chain Config").clicked() {
                        remove_jointed_mob_field(session, index, "chain");
                        *modified = true;
                    }
                }
            } else {
                ui.label(egui::RichText::new("No chain configuration").italics().color(egui::Color32::GRAY));
                if ui.button("Add Chain").clicked() {
                    add_chain_to_jointed_mob(session, index);
                    *modified = true;
                }
            }
        });
}

/// Get Vec2 from a jointed mob table
fn get_jointed_vec2(table: &toml::value::Table, key: &str) -> (f32, f32) {
    if let Some(arr) = table.get(key).and_then(|v| v.as_array()) {
        let x = arr.first()
            .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
            .unwrap_or(0.0) as f32;
        let y = arr.get(1)
            .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
            .unwrap_or(0.0) as f32;
        (x, y)
    } else {
        (0.0, 0.0)
    }
}

/// Set a field on a jointed mob at the given index
fn set_jointed_mob_field(session: &mut EditorSession, index: usize, field: &str, value: toml::Value) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        if let Some(arr) = mob.get_mut("jointed_mobs").and_then(|v| v.as_array_mut()) {
            if let Some(joint) = arr.get_mut(index).and_then(|v| v.as_table_mut()) {
                joint.insert(field.to_string(), value);
            }
        }
    }
}

/// Set a Vec2 field on a jointed mob
fn set_jointed_mob_vec2(session: &mut EditorSession, index: usize, field: &str, x: f32, y: f32) {
    let arr = toml::Value::Array(vec![
        toml::Value::Float(x as f64),
        toml::Value::Float(y as f64),
    ]);
    set_jointed_mob_field(session, index, field, arr);
}

/// Remove a field from a jointed mob at the given index
fn remove_jointed_mob_field(session: &mut EditorSession, index: usize, field: &str) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        if let Some(arr) = mob.get_mut("jointed_mobs").and_then(|v| v.as_array_mut()) {
            if let Some(joint) = arr.get_mut(index).and_then(|v| v.as_table_mut()) {
                joint.remove(field);
            }
        }
    }
}

/// Set a nested field on a jointed mob (e.g., angle_limit_range.min)
fn set_jointed_nested_field(session: &mut EditorSession, index: usize, parent: &str, field: &str, value: toml::Value) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        if let Some(arr) = mob.get_mut("jointed_mobs").and_then(|v| v.as_array_mut()) {
            if let Some(joint) = arr.get_mut(index).and_then(|v| v.as_table_mut()) {
                if let Some(parent_table) = joint.get_mut(parent).and_then(|v| v.as_table_mut()) {
                    parent_table.insert(field.to_string(), value);
                }
            }
        }
    }
}

/// Set a nested Vec2 field on a jointed mob (e.g., chain.pos_offset)
fn set_jointed_nested_vec2(session: &mut EditorSession, index: usize, parent: &str, field: &str, x: f32, y: f32) {
    let arr = toml::Value::Array(vec![
        toml::Value::Float(x as f64),
        toml::Value::Float(y as f64),
    ]);
    set_jointed_nested_field(session, index, parent, field, arr);
}

/// Remove a nested field from a jointed mob
fn remove_jointed_nested_field(session: &mut EditorSession, index: usize, parent: &str, field: &str) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        if let Some(arr) = mob.get_mut("jointed_mobs").and_then(|v| v.as_array_mut()) {
            if let Some(joint) = arr.get_mut(index).and_then(|v| v.as_table_mut()) {
                if let Some(parent_table) = joint.get_mut(parent).and_then(|v| v.as_table_mut()) {
                    parent_table.remove(field);
                }
            }
        }
    }
}

/// Set a deeply nested field (e.g., chain.random_chain.min_length)
fn set_jointed_deep_nested_field(session: &mut EditorSession, index: usize, parent: &str, nested: &str, field: &str, value: toml::Value) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        if let Some(arr) = mob.get_mut("jointed_mobs").and_then(|v| v.as_array_mut()) {
            if let Some(joint) = arr.get_mut(index).and_then(|v| v.as_table_mut()) {
                if let Some(parent_table) = joint.get_mut(parent).and_then(|v| v.as_table_mut()) {
                    if let Some(nested_table) = parent_table.get_mut(nested).and_then(|v| v.as_table_mut()) {
                        nested_table.insert(field.to_string(), value);
                    }
                }
            }
        }
    }
}

/// Delete a jointed mob at the given index
fn delete_jointed_mob(session: &mut EditorSession, index: usize) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        if let Some(arr) = mob.get_mut("jointed_mobs").and_then(|v| v.as_array_mut()) {
            if index < arr.len() {
                arr.remove(index);
            }
            // Clean up empty array
            if arr.is_empty() {
                mob.remove("jointed_mobs");
            }
        }
    }
}

/// Add a new jointed mob with defaults
fn add_new_jointed_mob(session: &mut EditorSession) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        if !mob.contains_key("jointed_mobs") {
            mob.insert("jointed_mobs".to_string(), toml::Value::Array(vec![]));
        }

        if let Some(arr) = mob.get_mut("jointed_mobs").and_then(|v| v.as_array_mut()) {
            let mut new_joint = toml::value::Table::new();
            new_joint.insert("key".to_string(), toml::Value::String(format!("joint_{}", arr.len())));
            new_joint.insert("mob_ref".to_string(), toml::Value::String("mobs/example.mob".to_string()));
            new_joint.insert("offset_pos".to_string(), toml::Value::Array(vec![
                toml::Value::Float(0.0),
                toml::Value::Float(-10.0),
            ]));
            new_joint.insert("anchor_1_pos".to_string(), toml::Value::Array(vec![
                toml::Value::Float(0.0),
                toml::Value::Float(0.0),
            ]));
            new_joint.insert("anchor_2_pos".to_string(), toml::Value::Array(vec![
                toml::Value::Float(0.0),
                toml::Value::Float(0.0),
            ]));
            new_joint.insert("compliance".to_string(), toml::Value::Float(0.000001));

            arr.push(toml::Value::Table(new_joint));
        }
    }
}

/// Add angle_limit_range to a jointed mob
fn add_angle_limit_to_jointed_mob(session: &mut EditorSession, index: usize) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        if let Some(arr) = mob.get_mut("jointed_mobs").and_then(|v| v.as_array_mut()) {
            if let Some(joint) = arr.get_mut(index).and_then(|v| v.as_table_mut()) {
                let mut angle_table = toml::value::Table::new();
                angle_table.insert("min".to_string(), toml::Value::Float(-45.0));
                angle_table.insert("max".to_string(), toml::Value::Float(45.0));
                angle_table.insert("torque".to_string(), toml::Value::Float(0.001));
                joint.insert("angle_limit_range".to_string(), toml::Value::Table(angle_table));
            }
        }
    }
}

/// Add chain configuration to a jointed mob
fn add_chain_to_jointed_mob(session: &mut EditorSession, index: usize) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        if let Some(arr) = mob.get_mut("jointed_mobs").and_then(|v| v.as_array_mut()) {
            if let Some(joint) = arr.get_mut(index).and_then(|v| v.as_table_mut()) {
                let mut chain_table = toml::value::Table::new();
                chain_table.insert("length".to_string(), toml::Value::Integer(3));
                chain_table.insert("pos_offset".to_string(), toml::Value::Array(vec![
                    toml::Value::Float(0.0),
                    toml::Value::Float(-5.0),
                ]));
                chain_table.insert("anchor_offset".to_string(), toml::Value::Array(vec![
                    toml::Value::Float(0.0),
                    toml::Value::Float(0.0),
                ]));
                joint.insert("chain".to_string(), toml::Value::Table(chain_table));
            }
        }
    }
}

/// Add random_chain to a jointed mob's chain config
fn add_random_chain_to_jointed_mob(session: &mut EditorSession, index: usize) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        if let Some(arr) = mob.get_mut("jointed_mobs").and_then(|v| v.as_array_mut()) {
            if let Some(joint) = arr.get_mut(index).and_then(|v| v.as_table_mut()) {
                if let Some(chain) = joint.get_mut("chain").and_then(|v| v.as_table_mut()) {
                    let mut random_table = toml::value::Table::new();
                    random_table.insert("min_length".to_string(), toml::Value::Integer(1));
                    random_table.insert("end_chance".to_string(), toml::Value::Float(0.15));
                    chain.insert("random_chain".to_string(), toml::Value::Table(random_table));
                }
            }
        }
    }
}

/// Render a sprite picker dropdown
/// Returns true if the sprite browser should be opened
fn render_sprite_picker(
    ui: &mut egui::Ui,
    display_table: &toml::value::Table,
    patch_table: &toml::value::Table,
    session: &mut EditorSession,
    sprite_registry: &SpriteRegistry,
    is_patch: bool,
    modified: &mut bool,
    config: &crate::plugin::EditorConfig,
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

        if ui.small_button("➕ Register New Sprite...")
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
            ui.add_space(16.0);
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
/// Returns Some(decoration_index) if the sprite browser should be opened for a decoration
fn render_decorations_section(
    ui: &mut egui::Ui,
    display_table: &toml::value::Table,
    patch_table: &toml::value::Table,
    session: &mut EditorSession,
    sprite_registry: &SpriteRegistry,
    is_patch: bool,
    modified: &mut bool,
    config: &crate::plugin::EditorConfig,
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
    config: &crate::plugin::EditorConfig,
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

            if ui.small_button("➕ Register New Sprite...")
                .on_hover_text("Find an aseprite file not yet in the sprite list and add it to game.assets.ron")
                .clicked()
            {
                open_browser_for = Some(index);
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

    open_browser_for
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

// ============================================================================
// Behavior Tree Editor
// ============================================================================

/// Node types available for behavior trees
const BEHAVIOR_NODE_TYPES: &[&str] = &[
    "Forever",   // Control: loops children forever
    "Sequence",  // Control: runs children in sequence
    "Fallback",  // Control: runs until one succeeds
    "While",     // Control: repeats while condition holds
    "IfThen",    // Control: conditional execution
    "Wait",      // Leaf: waits for seconds
    "Action",    // Leaf: executes behaviors
    "Trigger",   // Leaf: triggers event (future use)
];

/// Action types that can be used within Action nodes
const BEHAVIOR_ACTION_TYPES: &[(&str, &[&str])] = &[
    // Movement actions
    ("Movement", &["MoveDown", "MoveUp", "MoveLeft", "MoveRight", "MoveTo", "MoveForward", "BrakeHorizontal", "BrakeAngular"]),
    // Targeting actions
    ("Targeting", &["FindPlayerTarget", "MoveToTarget", "RotateToTarget", "LoseTarget"]),
    // Spawning actions
    ("Spawning", &["SpawnMob", "SpawnProjectile"]),
    // Timing actions
    ("Timing", &["DoForTime"]),
    // Communication actions
    ("Communication", &["TransmitMobBehavior"]),
];

/// Render the behavior tree section
fn render_behavior_section(
    ui: &mut egui::Ui,
    display_table: &toml::value::Table,
    patch_table: &toml::value::Table,
    session: &mut EditorSession,
    is_patch: bool,
    modified: &mut bool,
) {
    let is_patched = is_patch && patch_table.contains_key("behavior");

    egui::CollapsingHeader::new("Behavior Tree")
        .default_open(false)
        .show(ui, |ui| {
            // Patch status indicator
            ui.horizontal(|ui| {
                render_patch_indicator(ui, is_patched, is_patch);
                if is_patch && !is_patched {
                    ui.label(egui::RichText::new("(inherited from base)").small().color(INHERITED_COLOR));
                    if ui.button("Override").clicked() {
                        if let Some(behavior) = display_table.get("behavior").cloned() {
                            if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                                mob.insert("behavior".to_string(), behavior);
                                *modified = true;
                            }
                        }
                    }
                } else if is_patch && is_patched {
                    ui.label(egui::RichText::new("(overriding base)").small().color(PATCHED_COLOR));
                    if ui.button("Reset to base").clicked() {
                        if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                            mob.remove("behavior");
                            session.selected_behavior_node = None;
                            *modified = true;
                        }
                    }
                }
            });

            let has_behavior = display_table.contains_key("behavior");
            let can_edit = !is_patch || is_patched;

            if !has_behavior {
                ui.label("No behavior tree defined");
                if can_edit && ui.button("Add Behavior Tree").clicked() {
                    add_default_behavior_tree(session);
                    *modified = true;
                }
            } else if let Some(behavior) = display_table.get("behavior") {
                ui.separator();

                // Render the root node (path is empty)
                render_behavior_node(ui, session, behavior, &[], can_edit, 0, modified);

                // Delete root button
                if can_edit {
                    ui.separator();
                    if ui.add(egui::Button::new("Delete Behavior Tree").fill(egui::Color32::from_rgb(120, 60, 60))).clicked() {
                        if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
                            mob.remove("behavior");
                            session.selected_behavior_node = None;
                            *modified = true;
                        }
                    }
                }
            }
        });
}

/// Render a single behavior node and its children recursively
fn render_behavior_node(
    ui: &mut egui::Ui,
    session: &mut EditorSession,
    node: &toml::Value,
    path: &[usize],
    can_edit: bool,
    depth: usize,
    modified: &mut bool,
) {
    let Some(table) = node.as_table() else {
        ui.label("Invalid node (not a table)");
        return;
    };

    let node_type = table.get("type").and_then(|v| v.as_str()).unwrap_or("Unknown");
    let is_selected = session.selected_behavior_node.as_ref() == Some(&path.to_vec());

    // Build header text
    let header_text = build_node_header_text(table, node_type, is_selected);

    // Indent based on depth
    let indent = depth * 16;
    ui.add_space(indent as f32);

    egui::CollapsingHeader::new(header_text)
        .id_salt(format!("behavior_node_{:?}", path))
        .default_open(depth < 2) // Auto-expand first 2 levels
        .show(ui, |ui| {
            // Node controls row
            ui.horizontal(|ui| {
                // Select/Deselect button
                if ui.button(if is_selected { "Deselect" } else { "Select" }).clicked() {
                    session.selected_behavior_node = if is_selected { None } else { Some(path.to_vec()) };
                }

                if can_edit {
                    // Type dropdown
                    let mut current_type = node_type.to_string();
                    egui::ComboBox::from_id_salt(format!("type_combo_{:?}", path))
                        .selected_text(&current_type)
                        .width(80.0)
                        .show_ui(ui, |ui| {
                            for &type_name in BEHAVIOR_NODE_TYPES {
                                if ui.selectable_label(current_type == type_name, type_name).clicked() {
                                    current_type = type_name.to_string();
                                }
                            }
                        });
                    if current_type != node_type {
                        change_behavior_node_type(session, path, &current_type);
                        *modified = true;
                    }

                    // Move up/down buttons (only for non-root nodes)
                    if !path.is_empty() {
                        let parent_path = &path[..path.len() - 1];
                        let index = path[path.len() - 1];
                        let sibling_count = get_children_count(session, parent_path);

                        if index > 0 {
                            if ui.button("⏶").on_hover_text("Move up").clicked() {
                                move_behavior_node(session, path, -1);
                                // Update selected path if this was selected
                                if is_selected {
                                    let mut new_path = path.to_vec();
                                    let last_idx = new_path.len() - 1;
                                    new_path[last_idx] = index - 1;
                                    session.selected_behavior_node = Some(new_path);
                                }
                                *modified = true;
                            }
                        }
                        if index + 1 < sibling_count {
                            if ui.button("⏷").on_hover_text("Move down").clicked() {
                                move_behavior_node(session, path, 1);
                                // Update selected path if this was selected
                                if is_selected {
                                    let mut new_path = path.to_vec();
                                    let last_idx = new_path.len() - 1;
                                    new_path[last_idx] = index + 1;
                                    session.selected_behavior_node = Some(new_path);
                                }
                                *modified = true;
                            }
                        }

                        // Delete button (only for non-root nodes)
                        if ui.add(egui::Button::new("×").fill(egui::Color32::from_rgb(120, 60, 60)))
                            .on_hover_text("Delete node")
                            .clicked()
                        {
                            delete_behavior_node(session, path);
                            // Clear selection if deleted node was selected
                            if is_selected {
                                session.selected_behavior_node = None;
                            }
                            *modified = true;
                            return; // Don't render children after deletion
                        }
                    }
                }
            });

            ui.separator();

            // Render node-specific content based on type
            match node_type {
                "Forever" | "Sequence" | "Fallback" => {
                    render_control_node_children(ui, session, table, path, can_edit, depth, modified);
                }
                "While" => {
                    render_while_node(ui, session, table, path, can_edit, depth, modified);
                }
                "IfThen" => {
                    render_if_then_node(ui, session, table, path, can_edit, depth, modified);
                }
                "Wait" => {
                    render_wait_node(ui, session, table, path, can_edit, modified);
                }
                "Action" => {
                    render_action_node(ui, session, table, path, can_edit, modified);
                }
                "Trigger" => {
                    render_trigger_node(ui, session, table, path, can_edit, modified);
                }
                _ => {
                    ui.colored_label(egui::Color32::RED, format!("Unknown node type: {}", node_type));
                }
            }
        });
}

/// Build the header text for a behavior node
fn build_node_header_text(table: &toml::value::Table, node_type: &str, is_selected: bool) -> egui::RichText {
    let icon = match node_type {
        "Forever" => "↻",    // Loop
        "Sequence" => "○",   // Chain of steps
        "Fallback" => "⁉",   // Try alternatives
        "While" => "⟲",      // Repeat
        "IfThen" => "?",     // Conditional
        "Wait" => "⏱",       // Timer
        "Action" => "▶",     // Execute
        "Trigger" => "⚡",    // Event
        _ => "?",
    };

    let extra_info = match node_type {
        "Action" => {
            let name = table.get("name").and_then(|v| v.as_str()).unwrap_or("unnamed");
            format!(": \"{}\"", name)
        }
        "Wait" => {
            let seconds = table.get("seconds")
                .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
                .unwrap_or(0.0);
            format!(" {}s", seconds)
        }
        _ => String::new(),
    };

    let text = format!("{} {}{}", icon, node_type, extra_info);

    if is_selected {
        egui::RichText::new(format!("{} *", text))
            .strong()
            .color(egui::Color32::YELLOW)
    } else {
        egui::RichText::new(text)
    }
}

/// Render children for control nodes (Forever, Sequence, Fallback)
fn render_control_node_children(
    ui: &mut egui::Ui,
    session: &mut EditorSession,
    table: &toml::value::Table,
    path: &[usize],
    can_edit: bool,
    depth: usize,
    modified: &mut bool,
) {
    let children = table.get("children").and_then(|v| v.as_array()).cloned().unwrap_or_default();

    if children.is_empty() {
        ui.label(egui::RichText::new("(no children)").italics().color(INHERITED_COLOR));
    } else {
        for (i, child) in children.iter().enumerate() {
            let mut child_path = path.to_vec();
            child_path.push(i);
            render_behavior_node(ui, session, child, &child_path, can_edit, depth + 1, modified);
        }
    }

    if can_edit {
        ui.add_space(8.0);
        if ui.button("+ Add Child").clicked() {
            add_behavior_child(session, path);
            *modified = true;
        }
    }
}

/// Render a While node (has condition and child)
fn render_while_node(
    ui: &mut egui::Ui,
    session: &mut EditorSession,
    table: &toml::value::Table,
    path: &[usize],
    can_edit: bool,
    depth: usize,
    modified: &mut bool,
) {
    // Condition (optional)
    ui.label(egui::RichText::new("Condition:").small());
    if let Some(condition) = table.get("condition") {
        let mut cond_path = path.to_vec();
        cond_path.push(0); // Use index 0 for condition
        render_behavior_node(ui, session, condition, &cond_path, can_edit, depth + 1, modified);
    } else {
        ui.label(egui::RichText::new("(no condition)").italics().color(INHERITED_COLOR));
        if can_edit && ui.small_button("Add condition").clicked() {
            add_while_condition(session, path);
            *modified = true;
        }
    }

    ui.add_space(4.0);

    // Child (required)
    ui.label(egui::RichText::new("Child:").small());
    if let Some(child) = table.get("child") {
        let mut child_path = path.to_vec();
        child_path.push(1); // Use index 1 for child
        render_behavior_node(ui, session, child, &child_path, can_edit, depth + 1, modified);
    } else {
        ui.label(egui::RichText::new("(no child)").italics().color(INHERITED_COLOR));
        if can_edit && ui.small_button("Add child").clicked() {
            add_while_child(session, path);
            *modified = true;
        }
    }
}

/// Render an IfThen node (has condition, then_child, and optional else_child)
fn render_if_then_node(
    ui: &mut egui::Ui,
    session: &mut EditorSession,
    table: &toml::value::Table,
    path: &[usize],
    can_edit: bool,
    depth: usize,
    modified: &mut bool,
) {
    // Condition
    ui.label(egui::RichText::new("Condition:").small());
    if let Some(condition) = table.get("condition") {
        let mut cond_path = path.to_vec();
        cond_path.push(0);
        render_behavior_node(ui, session, condition, &cond_path, can_edit, depth + 1, modified);
    } else {
        ui.label(egui::RichText::new("(no condition)").italics().color(INHERITED_COLOR));
        if can_edit && ui.small_button("Add condition").clicked() {
            add_if_then_condition(session, path);
            *modified = true;
        }
    }

    ui.add_space(4.0);

    // Then child
    ui.label(egui::RichText::new("Then:").small());
    if let Some(then_child) = table.get("then_child") {
        let mut then_path = path.to_vec();
        then_path.push(1);
        render_behavior_node(ui, session, then_child, &then_path, can_edit, depth + 1, modified);
    } else {
        ui.label(egui::RichText::new("(no then branch)").italics().color(INHERITED_COLOR));
        if can_edit && ui.small_button("Add then branch").clicked() {
            add_if_then_child(session, path);
            *modified = true;
        }
    }

    ui.add_space(4.0);

    // Else child (optional)
    ui.label(egui::RichText::new("Else:").small());
    if let Some(else_child) = table.get("else_child") {
        let mut else_path = path.to_vec();
        else_path.push(2);
        render_behavior_node(ui, session, else_child, &else_path, can_edit, depth + 1, modified);

        if can_edit && ui.small_button("Remove else branch").clicked() {
            remove_if_else_child(session, path);
            *modified = true;
        }
    } else {
        ui.label(egui::RichText::new("(no else branch)").italics().color(INHERITED_COLOR));
        if can_edit && ui.small_button("Add else branch").clicked() {
            add_if_else_child(session, path);
            *modified = true;
        }
    }
}

/// Render a Wait node
fn render_wait_node(
    ui: &mut egui::Ui,
    session: &mut EditorSession,
    table: &toml::value::Table,
    path: &[usize],
    can_edit: bool,
    modified: &mut bool,
) {
    let seconds = table.get("seconds")
        .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
        .unwrap_or(1.0) as f32;

    ui.horizontal(|ui| {
        ui.label("Seconds:");
        if can_edit {
            let mut value = seconds;
            if ui.add(egui::DragValue::new(&mut value).speed(0.1).range(0.0..=100.0)).changed() {
                set_behavior_node_field(session, path, "seconds", toml::Value::Float(value as f64));
                *modified = true;
            }
        } else {
            ui.label(format!("{:.2}", seconds));
        }
    });
}

/// Render an Action node with its behaviors list
fn render_action_node(
    ui: &mut egui::Ui,
    session: &mut EditorSession,
    table: &toml::value::Table,
    path: &[usize],
    can_edit: bool,
    modified: &mut bool,
) {
    // Name field
    let name = table.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    ui.horizontal(|ui| {
        ui.label("Name:");
        if can_edit {
            let mut value = name.clone();
            if ui.text_edit_singleline(&mut value).changed() {
                set_behavior_node_field(session, path, "name", toml::Value::String(value));
                *modified = true;
            }
        } else {
            ui.label(&name);
        }
    });

    ui.add_space(4.0);

    // Behaviors list
    ui.label(egui::RichText::new("Behaviors:").small());
    let behaviors = table.get("behaviors").and_then(|v| v.as_array()).cloned().unwrap_or_default();

    if behaviors.is_empty() {
        ui.label(egui::RichText::new("(no behaviors)").italics().color(INHERITED_COLOR));
    } else {
        let mut delete_index: Option<usize> = None;

        for (i, behavior) in behaviors.iter().enumerate() {
            let Some(behavior_table) = behavior.as_table() else {
                continue;
            };

            let action_type = behavior_table.get("action").and_then(|v| v.as_str()).unwrap_or("Unknown");

            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.label(format!("{}.", i + 1));

                if can_edit {
                    // Action type combo
                    let mut current_action = action_type.to_string();
                    egui::ComboBox::from_id_salt(format!("action_combo_{:?}_{}", path, i))
                        .selected_text(&current_action)
                        .width(120.0)
                        .show_ui(ui, |ui| {
                            for (category, actions) in BEHAVIOR_ACTION_TYPES {
                                ui.label(egui::RichText::new(*category).small().color(INHERITED_COLOR));
                                for &action in *actions {
                                    if ui.selectable_label(current_action == action, action).clicked() {
                                        current_action = action.to_string();
                                    }
                                }
                                ui.separator();
                            }
                        });
                    if current_action != action_type {
                        change_action_behavior_type(session, path, i, &current_action);
                        *modified = true;
                    }

                    // Render action-specific parameters
                    render_action_parameters(ui, session, behavior_table, path, i, &current_action, modified);

                    // Move up/down
                    if i > 0 && ui.small_button("⏶").clicked() {
                        move_action_behavior(session, path, i, -1);
                        *modified = true;
                    }
                    if i + 1 < behaviors.len() && ui.small_button("⏷").clicked() {
                        move_action_behavior(session, path, i, 1);
                        *modified = true;
                    }

                    // Delete
                    if ui.small_button("×").clicked() {
                        delete_index = Some(i);
                    }
                } else {
                    ui.label(action_type);
                    render_action_parameters_readonly(ui, behavior_table, action_type);
                }
            });

            // Render nested behaviors for TransmitMobBehavior
            if action_type == "TransmitMobBehavior" {
                render_transmit_nested_behaviors(ui, session, behavior_table, path, i, can_edit, modified);
            }
        }

        // Process deferred deletion
        if let Some(index) = delete_index {
            delete_action_behavior(session, path, index);
            *modified = true;
        }
    }

    if can_edit {
        ui.add_space(4.0);
        if ui.small_button("+ Add Behavior").clicked() {
            add_action_behavior(session, path);
            *modified = true;
        }
    }
}

/// Render parameters for specific action types
fn render_action_parameters(
    ui: &mut egui::Ui,
    session: &mut EditorSession,
    behavior_table: &toml::value::Table,
    path: &[usize],
    behavior_index: usize,
    action_type: &str,
    modified: &mut bool,
) {
    match action_type {
        "MoveTo" => {
            let x = behavior_table.get("x")
                .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
                .unwrap_or(0.0) as f32;
            let y = behavior_table.get("y")
                .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
                .unwrap_or(0.0) as f32;

            let mut new_x = x;
            let mut new_y = y;
            let x_changed = ui.add(egui::DragValue::new(&mut new_x).speed(0.5).prefix("x: ")).changed();
            let y_changed = ui.add(egui::DragValue::new(&mut new_y).speed(0.5).prefix("y: ")).changed();
            if x_changed || y_changed {
                set_action_behavior_param(session, path, behavior_index, "x", toml::Value::Float(new_x as f64));
                set_action_behavior_param(session, path, behavior_index, "y", toml::Value::Float(new_y as f64));
                *modified = true;
            }
        }
        "DoForTime" => {
            let seconds = behavior_table.get("seconds")
                .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
                .unwrap_or(1.0) as f32;

            let mut value = seconds;
            if ui.add(egui::DragValue::new(&mut value).speed(0.1).range(0.0..=100.0).prefix("s: ")).changed() {
                set_action_behavior_param(session, path, behavior_index, "seconds", toml::Value::Float(value as f64));
                *modified = true;
            }
        }
        "SpawnMob" | "SpawnProjectile" => {
            let keys = behavior_table.get("keys")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", "))
                .unwrap_or_default();

            let mut value = keys;
            if ui.add(egui::TextEdit::singleline(&mut value).desired_width(100.0).hint_text("keys")).changed() {
                let keys_arr: Vec<toml::Value> = value.split(',')
                    .map(|s| toml::Value::String(s.trim().to_string()))
                    .filter(|v| !v.as_str().unwrap_or("").is_empty())
                    .collect();
                if keys_arr.is_empty() {
                    remove_action_behavior_param(session, path, behavior_index, "keys");
                } else {
                    set_action_behavior_param(session, path, behavior_index, "keys", toml::Value::Array(keys_arr));
                }
                *modified = true;
            }
        }
        "TransmitMobBehavior" => {
            let mob_type = behavior_table.get("mob_type").and_then(|v| v.as_str()).unwrap_or("").to_string();

            let mut value = mob_type;
            if ui.add(egui::TextEdit::singleline(&mut value).desired_width(100.0).hint_text("mob_type")).changed() {
                set_action_behavior_param(session, path, behavior_index, "mob_type", toml::Value::String(value));
                *modified = true;
            }
        }
        _ => {
            // No parameters for simple actions
        }
    }
}

/// Render parameters in read-only mode
fn render_action_parameters_readonly(ui: &mut egui::Ui, behavior_table: &toml::value::Table, action_type: &str) {
    match action_type {
        "MoveTo" => {
            let x = behavior_table.get("x").and_then(|v| v.as_float()).unwrap_or(0.0);
            let y = behavior_table.get("y").and_then(|v| v.as_float()).unwrap_or(0.0);
            ui.label(format!("({:.1}, {:.1})", x, y));
        }
        "DoForTime" => {
            let seconds = behavior_table.get("seconds").and_then(|v| v.as_float()).unwrap_or(1.0);
            ui.label(format!("{:.1}s", seconds));
        }
        "SpawnMob" | "SpawnProjectile" => {
            if let Some(keys) = behavior_table.get("keys").and_then(|v| v.as_array()) {
                let keys_str: Vec<_> = keys.iter().filter_map(|v| v.as_str()).collect();
                ui.label(format!("[{}]", keys_str.join(", ")));
            }
        }
        "TransmitMobBehavior" => {
            let mob_type = behavior_table.get("mob_type").and_then(|v| v.as_str()).unwrap_or("");
            ui.label(format!("▶ {}", mob_type));
        }
        _ => {}
    }
}

/// Render a Trigger node
fn render_trigger_node(
    ui: &mut egui::Ui,
    session: &mut EditorSession,
    table: &toml::value::Table,
    path: &[usize],
    can_edit: bool,
    modified: &mut bool,
) {
    let trigger_type = table.get("trigger_type").and_then(|v| v.as_str()).unwrap_or("").to_string();

    ui.horizontal(|ui| {
        ui.label("Trigger Type:");
        if can_edit {
            let mut value = trigger_type;
            if ui.text_edit_singleline(&mut value).changed() {
                set_behavior_node_field(session, path, "trigger_type", toml::Value::String(value));
                *modified = true;
            }
        } else {
            ui.label(&trigger_type);
        }
    });

    ui.label(egui::RichText::new("(Trigger nodes are for future use)").small().color(INHERITED_COLOR));
}

// ============================================================================
// Behavior Tree TOML Manipulation Helpers
// ============================================================================

/// Add a default behavior tree (Forever with one Action child)
fn add_default_behavior_tree(session: &mut EditorSession) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        let mut action = toml::value::Table::new();
        action.insert("type".to_string(), toml::Value::String("Action".to_string()));
        action.insert("name".to_string(), toml::Value::String("Movement".to_string()));
        action.insert("behaviors".to_string(), toml::Value::Array(vec![
            {
                let mut behavior = toml::value::Table::new();
                behavior.insert("action".to_string(), toml::Value::String("MoveDown".to_string()));
                toml::Value::Table(behavior)
            }
        ]));

        let mut root = toml::value::Table::new();
        root.insert("type".to_string(), toml::Value::String("Forever".to_string()));
        root.insert("children".to_string(), toml::Value::Array(vec![toml::Value::Table(action)]));

        mob.insert("behavior".to_string(), toml::Value::Table(root));
    }
}

/// Get a mutable reference to a behavior node at the given path
fn get_behavior_node_mut<'a>(session: &'a mut EditorSession, path: &[usize]) -> Option<&'a mut toml::Value> {
    let mob = session.current_mob.as_mut()?.as_table_mut()?;
    let mut current = mob.get_mut("behavior")?;

    for &index in path {
        let table = current.as_table_mut()?;
        let node_type = table.get("type").and_then(|v| v.as_str())?;

        current = match node_type {
            "Forever" | "Sequence" | "Fallback" => {
                table.get_mut("children")?.as_array_mut()?.get_mut(index)?
            }
            "While" => {
                if index == 0 {
                    table.get_mut("condition")?
                } else {
                    table.get_mut("child")?
                }
            }
            "IfThen" => {
                match index {
                    0 => table.get_mut("condition")?,
                    1 => table.get_mut("then_child")?,
                    2 => table.get_mut("else_child")?,
                    _ => return None,
                }
            }
            _ => return None, // Leaf nodes have no children
        };
    }

    Some(current)
}

/// Set a field on a behavior node at the given path
fn set_behavior_node_field(session: &mut EditorSession, path: &[usize], field: &str, value: toml::Value) {
    if let Some(node) = get_behavior_node_mut(session, path) {
        if let Some(table) = node.as_table_mut() {
            table.insert(field.to_string(), value);
        }
    }
}

/// Get the number of children for a control node at the given path
fn get_children_count(session: &EditorSession, path: &[usize]) -> usize {
    let Some(mob) = session.current_mob.as_ref().and_then(|v| v.as_table()) else {
        return 0;
    };

    let Some(behavior) = mob.get("behavior") else {
        return 0;
    };

    let mut current = behavior;

    for &index in path {
        let Some(table) = current.as_table() else {
            return 0;
        };
        let node_type = table.get("type").and_then(|v| v.as_str()).unwrap_or("");

        current = match node_type {
            "Forever" | "Sequence" | "Fallback" => {
                match table.get("children").and_then(|v| v.as_array()).and_then(|arr| arr.get(index)) {
                    Some(c) => c,
                    None => return 0,
                }
            }
            "While" => {
                if index == 0 {
                    match table.get("condition") {
                        Some(c) => c,
                        None => return 0,
                    }
                } else {
                    match table.get("child") {
                        Some(c) => c,
                        None => return 0,
                    }
                }
            }
            "IfThen" => {
                match index {
                    0 => table.get("condition"),
                    1 => table.get("then_child"),
                    2 => table.get("else_child"),
                    _ => None,
                }.unwrap_or(&toml::Value::Boolean(false)) // placeholder
            }
            _ => return 0,
        };
    }

    // Now count children of the node at path
    let Some(table) = current.as_table() else {
        return 0;
    };
    let node_type = table.get("type").and_then(|v| v.as_str()).unwrap_or("");

    match node_type {
        "Forever" | "Sequence" | "Fallback" => {
            table.get("children").and_then(|v| v.as_array()).map(|arr| arr.len()).unwrap_or(0)
        }
        _ => 0,
    }
}

/// Add a child to a control node (Forever, Sequence, Fallback)
fn add_behavior_child(session: &mut EditorSession, path: &[usize]) {
    if let Some(node) = get_behavior_node_mut(session, path) {
        if let Some(table) = node.as_table_mut() {
            // Create a default Action child
            let mut action = toml::value::Table::new();
            action.insert("type".to_string(), toml::Value::String("Action".to_string()));
            action.insert("name".to_string(), toml::Value::String("New Action".to_string()));
            action.insert("behaviors".to_string(), toml::Value::Array(vec![]));

            if let Some(children) = table.get_mut("children").and_then(|v| v.as_array_mut()) {
                children.push(toml::Value::Table(action));
            } else {
                table.insert("children".to_string(), toml::Value::Array(vec![toml::Value::Table(action)]));
            }
        }
    }
}

/// Delete a behavior node at the given path
fn delete_behavior_node(session: &mut EditorSession, path: &[usize]) {
    if path.is_empty() {
        // Can't delete root via this function
        return;
    }

    let parent_path = &path[..path.len() - 1];
    let index = path[path.len() - 1];

    if let Some(parent) = get_behavior_node_mut(session, parent_path) {
        if let Some(table) = parent.as_table_mut() {
            let node_type = table.get("type").and_then(|v| v.as_str()).unwrap_or("");

            match node_type {
                "Forever" | "Sequence" | "Fallback" => {
                    if let Some(children) = table.get_mut("children").and_then(|v| v.as_array_mut()) {
                        if index < children.len() {
                            children.remove(index);
                        }
                    }
                }
                "While" => {
                    if index == 0 {
                        table.remove("condition");
                    }
                    // Don't allow deleting the child - While always needs one
                }
                "IfThen" => {
                    match index {
                        2 => { table.remove("else_child"); }
                        // Don't allow deleting condition or then_child
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}

/// Move a behavior node up or down within its parent
fn move_behavior_node(session: &mut EditorSession, path: &[usize], direction: i32) {
    if path.is_empty() {
        return;
    }

    let parent_path = &path[..path.len() - 1];
    let index = path[path.len() - 1];

    if let Some(parent) = get_behavior_node_mut(session, parent_path) {
        if let Some(table) = parent.as_table_mut() {
            let node_type = table.get("type").and_then(|v| v.as_str()).unwrap_or("");

            if matches!(node_type, "Forever" | "Sequence" | "Fallback") {
                if let Some(children) = table.get_mut("children").and_then(|v| v.as_array_mut()) {
                    let new_index = (index as i32 + direction) as usize;
                    if new_index < children.len() {
                        children.swap(index, new_index);
                    }
                }
            }
        }
    }
}

/// Change the type of a behavior node
fn change_behavior_node_type(session: &mut EditorSession, path: &[usize], new_type: &str) {
    if let Some(node) = get_behavior_node_mut(session, path) {
        if let Some(table) = node.as_table_mut() {
            let old_type = table.get("type").and_then(|v| v.as_str()).unwrap_or("").to_string();

            // Only proceed if type is actually changing
            if old_type == new_type {
                return;
            }

            // Update type
            table.insert("type".to_string(), toml::Value::String(new_type.to_string()));

            // Handle structure changes based on old/new type categories
            let old_is_control = matches!(old_type.as_str(), "Forever" | "Sequence" | "Fallback");
            let new_is_control = matches!(new_type, "Forever" | "Sequence" | "Fallback");

            if old_is_control && new_is_control {
                // Keep children array as-is
            } else if old_is_control && !new_is_control {
                // Switching from control to leaf - remove children
                table.remove("children");

                // Add required fields for new type
                match new_type {
                    "Wait" => {
                        table.insert("seconds".to_string(), toml::Value::Float(1.0));
                    }
                    "Action" => {
                        table.insert("name".to_string(), toml::Value::String("New Action".to_string()));
                        table.insert("behaviors".to_string(), toml::Value::Array(vec![]));
                    }
                    "Trigger" => {
                        table.insert("trigger_type".to_string(), toml::Value::String("".to_string()));
                    }
                    "While" => {
                        // Need a child
                        let mut child = toml::value::Table::new();
                        child.insert("type".to_string(), toml::Value::String("Action".to_string()));
                        child.insert("name".to_string(), toml::Value::String("Child".to_string()));
                        child.insert("behaviors".to_string(), toml::Value::Array(vec![]));
                        table.insert("child".to_string(), toml::Value::Table(child));
                    }
                    "IfThen" => {
                        // Need condition and then_child
                        let mut cond = toml::value::Table::new();
                        cond.insert("type".to_string(), toml::Value::String("Wait".to_string()));
                        cond.insert("seconds".to_string(), toml::Value::Float(1.0));
                        table.insert("condition".to_string(), toml::Value::Table(cond));

                        let mut then = toml::value::Table::new();
                        then.insert("type".to_string(), toml::Value::String("Action".to_string()));
                        then.insert("name".to_string(), toml::Value::String("Then".to_string()));
                        then.insert("behaviors".to_string(), toml::Value::Array(vec![]));
                        table.insert("then_child".to_string(), toml::Value::Table(then));
                    }
                    _ => {}
                }
            } else if !old_is_control && new_is_control {
                // Switching from leaf to control - add empty children array
                // Remove old leaf-specific fields
                table.remove("seconds");
                table.remove("name");
                table.remove("behaviors");
                table.remove("trigger_type");
                table.remove("child");
                table.remove("condition");
                table.remove("then_child");
                table.remove("else_child");

                table.insert("children".to_string(), toml::Value::Array(vec![]));
            } else {
                // Switching between different leaf/special types
                // Clean up old fields and add new ones
                table.remove("seconds");
                table.remove("name");
                table.remove("behaviors");
                table.remove("trigger_type");
                table.remove("child");
                table.remove("condition");
                table.remove("then_child");
                table.remove("else_child");
                table.remove("children");

                match new_type {
                    "Wait" => {
                        table.insert("seconds".to_string(), toml::Value::Float(1.0));
                    }
                    "Action" => {
                        table.insert("name".to_string(), toml::Value::String("New Action".to_string()));
                        table.insert("behaviors".to_string(), toml::Value::Array(vec![]));
                    }
                    "Trigger" => {
                        table.insert("trigger_type".to_string(), toml::Value::String("".to_string()));
                    }
                    "While" => {
                        let mut child = toml::value::Table::new();
                        child.insert("type".to_string(), toml::Value::String("Action".to_string()));
                        child.insert("name".to_string(), toml::Value::String("Child".to_string()));
                        child.insert("behaviors".to_string(), toml::Value::Array(vec![]));
                        table.insert("child".to_string(), toml::Value::Table(child));
                    }
                    "IfThen" => {
                        let mut cond = toml::value::Table::new();
                        cond.insert("type".to_string(), toml::Value::String("Wait".to_string()));
                        cond.insert("seconds".to_string(), toml::Value::Float(1.0));
                        table.insert("condition".to_string(), toml::Value::Table(cond));

                        let mut then = toml::value::Table::new();
                        then.insert("type".to_string(), toml::Value::String("Action".to_string()));
                        then.insert("name".to_string(), toml::Value::String("Then".to_string()));
                        then.insert("behaviors".to_string(), toml::Value::Array(vec![]));
                        table.insert("then_child".to_string(), toml::Value::Table(then));
                    }
                    "Forever" | "Sequence" | "Fallback" => {
                        table.insert("children".to_string(), toml::Value::Array(vec![]));
                    }
                    _ => {}
                }
            }
        }
    }
}

// While/IfThen specific helpers

fn add_while_condition(session: &mut EditorSession, path: &[usize]) {
    if let Some(node) = get_behavior_node_mut(session, path) {
        if let Some(table) = node.as_table_mut() {
            let mut cond = toml::value::Table::new();
            cond.insert("type".to_string(), toml::Value::String("Wait".to_string()));
            cond.insert("seconds".to_string(), toml::Value::Float(1.0));
            table.insert("condition".to_string(), toml::Value::Table(cond));
        }
    }
}

fn add_while_child(session: &mut EditorSession, path: &[usize]) {
    if let Some(node) = get_behavior_node_mut(session, path) {
        if let Some(table) = node.as_table_mut() {
            let mut child = toml::value::Table::new();
            child.insert("type".to_string(), toml::Value::String("Action".to_string()));
            child.insert("name".to_string(), toml::Value::String("Child".to_string()));
            child.insert("behaviors".to_string(), toml::Value::Array(vec![]));
            table.insert("child".to_string(), toml::Value::Table(child));
        }
    }
}

fn add_if_then_condition(session: &mut EditorSession, path: &[usize]) {
    if let Some(node) = get_behavior_node_mut(session, path) {
        if let Some(table) = node.as_table_mut() {
            let mut cond = toml::value::Table::new();
            cond.insert("type".to_string(), toml::Value::String("Wait".to_string()));
            cond.insert("seconds".to_string(), toml::Value::Float(1.0));
            table.insert("condition".to_string(), toml::Value::Table(cond));
        }
    }
}

fn add_if_then_child(session: &mut EditorSession, path: &[usize]) {
    if let Some(node) = get_behavior_node_mut(session, path) {
        if let Some(table) = node.as_table_mut() {
            let mut child = toml::value::Table::new();
            child.insert("type".to_string(), toml::Value::String("Action".to_string()));
            child.insert("name".to_string(), toml::Value::String("Then".to_string()));
            child.insert("behaviors".to_string(), toml::Value::Array(vec![]));
            table.insert("then_child".to_string(), toml::Value::Table(child));
        }
    }
}

fn add_if_else_child(session: &mut EditorSession, path: &[usize]) {
    if let Some(node) = get_behavior_node_mut(session, path) {
        if let Some(table) = node.as_table_mut() {
            let mut child = toml::value::Table::new();
            child.insert("type".to_string(), toml::Value::String("Action".to_string()));
            child.insert("name".to_string(), toml::Value::String("Else".to_string()));
            child.insert("behaviors".to_string(), toml::Value::Array(vec![]));
            table.insert("else_child".to_string(), toml::Value::Table(child));
        }
    }
}

fn remove_if_else_child(session: &mut EditorSession, path: &[usize]) {
    if let Some(node) = get_behavior_node_mut(session, path) {
        if let Some(table) = node.as_table_mut() {
            table.remove("else_child");
        }
    }
}

// Action behavior manipulation helpers

fn add_action_behavior(session: &mut EditorSession, path: &[usize]) {
    if let Some(node) = get_behavior_node_mut(session, path) {
        if let Some(table) = node.as_table_mut() {
            let mut behavior = toml::value::Table::new();
            behavior.insert("action".to_string(), toml::Value::String("MoveDown".to_string()));

            if let Some(behaviors) = table.get_mut("behaviors").and_then(|v| v.as_array_mut()) {
                behaviors.push(toml::Value::Table(behavior));
            } else {
                table.insert("behaviors".to_string(), toml::Value::Array(vec![toml::Value::Table(behavior)]));
            }
        }
    }
}

fn delete_action_behavior(session: &mut EditorSession, path: &[usize], index: usize) {
    if let Some(node) = get_behavior_node_mut(session, path) {
        if let Some(table) = node.as_table_mut() {
            if let Some(behaviors) = table.get_mut("behaviors").and_then(|v| v.as_array_mut()) {
                if index < behaviors.len() {
                    behaviors.remove(index);
                }
            }
        }
    }
}

fn move_action_behavior(session: &mut EditorSession, path: &[usize], index: usize, direction: i32) {
    if let Some(node) = get_behavior_node_mut(session, path) {
        if let Some(table) = node.as_table_mut() {
            if let Some(behaviors) = table.get_mut("behaviors").and_then(|v| v.as_array_mut()) {
                let new_index = (index as i32 + direction) as usize;
                if new_index < behaviors.len() {
                    behaviors.swap(index, new_index);
                }
            }
        }
    }
}

fn change_action_behavior_type(session: &mut EditorSession, path: &[usize], index: usize, new_action: &str) {
    if let Some(node) = get_behavior_node_mut(session, path) {
        if let Some(table) = node.as_table_mut() {
            if let Some(behaviors) = table.get_mut("behaviors").and_then(|v| v.as_array_mut()) {
                if let Some(behavior) = behaviors.get_mut(index).and_then(|v| v.as_table_mut()) {
                    // Clear old fields except action
                    let keys_to_remove: Vec<_> = behavior.keys()
                        .filter(|k| *k != "action")
                        .cloned()
                        .collect();
                    for key in keys_to_remove {
                        behavior.remove(&key);
                    }

                    // Set new action type
                    behavior.insert("action".to_string(), toml::Value::String(new_action.to_string()));

                    // Add default parameters for actions that need them
                    match new_action {
                        "MoveTo" => {
                            behavior.insert("x".to_string(), toml::Value::Float(0.0));
                            behavior.insert("y".to_string(), toml::Value::Float(0.0));
                        }
                        "DoForTime" => {
                            behavior.insert("seconds".to_string(), toml::Value::Float(1.0));
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn set_action_behavior_param(session: &mut EditorSession, path: &[usize], index: usize, param: &str, value: toml::Value) {
    if let Some(node) = get_behavior_node_mut(session, path) {
        if let Some(table) = node.as_table_mut() {
            if let Some(behaviors) = table.get_mut("behaviors").and_then(|v| v.as_array_mut()) {
                if let Some(behavior) = behaviors.get_mut(index).and_then(|v| v.as_table_mut()) {
                    behavior.insert(param.to_string(), value);
                }
            }
        }
    }
}

fn remove_action_behavior_param(session: &mut EditorSession, path: &[usize], index: usize, param: &str) {
    if let Some(node) = get_behavior_node_mut(session, path) {
        if let Some(table) = node.as_table_mut() {
            if let Some(behaviors) = table.get_mut("behaviors").and_then(|v| v.as_array_mut()) {
                if let Some(behavior) = behaviors.get_mut(index).and_then(|v| v.as_table_mut()) {
                    behavior.remove(param);
                }
            }
        }
    }
}

// ============================================================================
// TransmitMobBehavior Nested Behaviors Editor
// ============================================================================

/// Render the nested behaviors list for a TransmitMobBehavior action
fn render_transmit_nested_behaviors(
    ui: &mut egui::Ui,
    session: &mut EditorSession,
    behavior_table: &toml::value::Table,
    path: &[usize],
    behavior_index: usize,
    can_edit: bool,
    modified: &mut bool,
) {
    let nested_behaviors = behavior_table
        .get("behaviors")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    ui.horizontal(|ui| {
        ui.add_space(32.0);
        ui.label(egui::RichText::new("Transmitted behaviors:").small());
    });

    if nested_behaviors.is_empty() {
        ui.horizontal(|ui| {
            ui.add_space(40.0);
            ui.label(egui::RichText::new("(none)").italics().color(INHERITED_COLOR));
        });
    } else {
        let mut delete_nested: Option<usize> = None;

        for (j, nested) in nested_behaviors.iter().enumerate() {
            let Some(nested_table) = nested.as_table() else {
                continue;
            };

            let nested_action = nested_table.get("action").and_then(|v| v.as_str()).unwrap_or("Unknown");

            ui.horizontal(|ui| {
                ui.add_space(40.0);
                ui.label(format!("{}.", j + 1));

                if can_edit {
                    // Action type combo for nested behavior
                    let mut current_nested = nested_action.to_string();
                    egui::ComboBox::from_id_salt(format!("nested_action_{:?}_{}_{}", path, behavior_index, j))
                        .selected_text(&current_nested)
                        .width(100.0)
                        .show_ui(ui, |ui| {
                            // Only show simple actions for nested behaviors (no TransmitMobBehavior to avoid infinite nesting)
                            for (category, actions) in BEHAVIOR_ACTION_TYPES {
                                if *category == "Communication" {
                                    continue; // Skip TransmitMobBehavior
                                }
                                ui.label(egui::RichText::new(*category).small().color(INHERITED_COLOR));
                                for &action in *actions {
                                    if ui.selectable_label(current_nested == action, action).clicked() {
                                        current_nested = action.to_string();
                                    }
                                }
                                ui.separator();
                            }
                        });
                    if current_nested != nested_action {
                        change_transmit_nested_behavior_type(session, path, behavior_index, j, &current_nested);
                        *modified = true;
                    }

                    // Render parameters for nested behavior
                    render_transmit_nested_params(ui, session, nested_table, path, behavior_index, j, &current_nested, modified);

                    // Move up/down
                    if j > 0 && ui.small_button("⏶").clicked() {
                        move_transmit_nested_behavior(session, path, behavior_index, j, -1);
                        *modified = true;
                    }
                    if j + 1 < nested_behaviors.len() && ui.small_button("⏷").clicked() {
                        move_transmit_nested_behavior(session, path, behavior_index, j, 1);
                        *modified = true;
                    }

                    // Delete
                    if ui.small_button("×").clicked() {
                        delete_nested = Some(j);
                    }
                } else {
                    ui.label(nested_action);
                }
            });
        }

        if let Some(idx) = delete_nested {
            delete_transmit_nested_behavior(session, path, behavior_index, idx);
            *modified = true;
        }
    }

    if can_edit {
        ui.horizontal(|ui| {
            ui.add_space(40.0);
            if ui.small_button("+ Add").clicked() {
                add_transmit_nested_behavior(session, path, behavior_index);
                *modified = true;
            }
        });
    }
}

/// Render parameters for a nested behavior in TransmitMobBehavior
fn render_transmit_nested_params(
    ui: &mut egui::Ui,
    session: &mut EditorSession,
    nested_table: &toml::value::Table,
    path: &[usize],
    behavior_index: usize,
    nested_index: usize,
    action_type: &str,
    modified: &mut bool,
) {
    match action_type {
        "MoveTo" => {
            let x = nested_table.get("x")
                .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
                .unwrap_or(0.0) as f32;
            let y = nested_table.get("y")
                .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
                .unwrap_or(0.0) as f32;

            let mut new_x = x;
            let mut new_y = y;
            let x_changed = ui.add(egui::DragValue::new(&mut new_x).speed(0.5).prefix("x: ")).changed();
            let y_changed = ui.add(egui::DragValue::new(&mut new_y).speed(0.5).prefix("y: ")).changed();
            if x_changed || y_changed {
                set_transmit_nested_param(session, path, behavior_index, nested_index, "x", toml::Value::Float(new_x as f64));
                set_transmit_nested_param(session, path, behavior_index, nested_index, "y", toml::Value::Float(new_y as f64));
                *modified = true;
            }
        }
        "DoForTime" => {
            let seconds = nested_table.get("seconds")
                .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
                .unwrap_or(1.0) as f32;

            let mut value = seconds;
            if ui.add(egui::DragValue::new(&mut value).speed(0.1).range(0.0..=100.0).prefix("s: ")).changed() {
                set_transmit_nested_param(session, path, behavior_index, nested_index, "seconds", toml::Value::Float(value as f64));
                *modified = true;
            }
        }
        _ => {
            // No parameters for simple actions
        }
    }
}

// Helper functions for TransmitMobBehavior nested behaviors

fn add_transmit_nested_behavior(session: &mut EditorSession, path: &[usize], behavior_index: usize) {
    if let Some(node) = get_behavior_node_mut(session, path) {
        if let Some(table) = node.as_table_mut() {
            if let Some(behaviors) = table.get_mut("behaviors").and_then(|v| v.as_array_mut()) {
                if let Some(behavior) = behaviors.get_mut(behavior_index).and_then(|v| v.as_table_mut()) {
                    let mut new_nested = toml::value::Table::new();
                    new_nested.insert("action".to_string(), toml::Value::String("MoveDown".to_string()));

                    if let Some(nested_arr) = behavior.get_mut("behaviors").and_then(|v| v.as_array_mut()) {
                        nested_arr.push(toml::Value::Table(new_nested));
                    } else {
                        behavior.insert("behaviors".to_string(), toml::Value::Array(vec![toml::Value::Table(new_nested)]));
                    }
                }
            }
        }
    }
}

fn delete_transmit_nested_behavior(session: &mut EditorSession, path: &[usize], behavior_index: usize, nested_index: usize) {
    if let Some(node) = get_behavior_node_mut(session, path) {
        if let Some(table) = node.as_table_mut() {
            if let Some(behaviors) = table.get_mut("behaviors").and_then(|v| v.as_array_mut()) {
                if let Some(behavior) = behaviors.get_mut(behavior_index).and_then(|v| v.as_table_mut()) {
                    if let Some(nested_arr) = behavior.get_mut("behaviors").and_then(|v| v.as_array_mut()) {
                        if nested_index < nested_arr.len() {
                            nested_arr.remove(nested_index);
                        }
                    }
                }
            }
        }
    }
}

fn move_transmit_nested_behavior(session: &mut EditorSession, path: &[usize], behavior_index: usize, nested_index: usize, direction: i32) {
    if let Some(node) = get_behavior_node_mut(session, path) {
        if let Some(table) = node.as_table_mut() {
            if let Some(behaviors) = table.get_mut("behaviors").and_then(|v| v.as_array_mut()) {
                if let Some(behavior) = behaviors.get_mut(behavior_index).and_then(|v| v.as_table_mut()) {
                    if let Some(nested_arr) = behavior.get_mut("behaviors").and_then(|v| v.as_array_mut()) {
                        let new_index = (nested_index as i32 + direction) as usize;
                        if new_index < nested_arr.len() {
                            nested_arr.swap(nested_index, new_index);
                        }
                    }
                }
            }
        }
    }
}

fn change_transmit_nested_behavior_type(session: &mut EditorSession, path: &[usize], behavior_index: usize, nested_index: usize, new_action: &str) {
    if let Some(node) = get_behavior_node_mut(session, path) {
        if let Some(table) = node.as_table_mut() {
            if let Some(behaviors) = table.get_mut("behaviors").and_then(|v| v.as_array_mut()) {
                if let Some(behavior) = behaviors.get_mut(behavior_index).and_then(|v| v.as_table_mut()) {
                    if let Some(nested_arr) = behavior.get_mut("behaviors").and_then(|v| v.as_array_mut()) {
                        if let Some(nested) = nested_arr.get_mut(nested_index).and_then(|v| v.as_table_mut()) {
                            // Clear old fields except action
                            let keys_to_remove: Vec<_> = nested.keys()
                                .filter(|k| *k != "action")
                                .cloned()
                                .collect();
                            for key in keys_to_remove {
                                nested.remove(&key);
                            }

                            // Set new action type
                            nested.insert("action".to_string(), toml::Value::String(new_action.to_string()));

                            // Add default parameters
                            match new_action {
                                "MoveTo" => {
                                    nested.insert("x".to_string(), toml::Value::Float(0.0));
                                    nested.insert("y".to_string(), toml::Value::Float(0.0));
                                }
                                "DoForTime" => {
                                    nested.insert("seconds".to_string(), toml::Value::Float(1.0));
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
}

fn set_transmit_nested_param(session: &mut EditorSession, path: &[usize], behavior_index: usize, nested_index: usize, param: &str, value: toml::Value) {
    if let Some(node) = get_behavior_node_mut(session, path) {
        if let Some(table) = node.as_table_mut() {
            if let Some(behaviors) = table.get_mut("behaviors").and_then(|v| v.as_array_mut()) {
                if let Some(behavior) = behaviors.get_mut(behavior_index).and_then(|v| v.as_table_mut()) {
                    if let Some(nested_arr) = behavior.get_mut("behaviors").and_then(|v| v.as_array_mut()) {
                        if let Some(nested) = nested_arr.get_mut(nested_index).and_then(|v| v.as_table_mut()) {
                            nested.insert(param.to_string(), value);
                        }
                    }
                }
            }
        }
    }
}
