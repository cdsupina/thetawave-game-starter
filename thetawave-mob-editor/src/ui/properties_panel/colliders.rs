//! Collider editing functionality for the properties panel.
//!
//! This module handles rendering and editing of mob colliders,
//! including rectangle and circle shapes.

use bevy_egui::egui;

use crate::data::EditorSession;

use super::fields::{render_patch_indicator, INHERITED_COLOR, PATCHED_COLOR};

/// Render the colliders section of the properties panel.
///
/// # Arguments
/// * `ui` - The egui UI context
/// * `display_table` - The merged mob data for display
/// * `patch_table` - The patch-only data (for checking what's overridden)
/// * `session` - The editor session
/// * `is_patch` - Whether we're editing a patch file
/// * `modified` - Set to true if any value was modified
pub fn render_colliders_section(
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
                    ui.label(
                        egui::RichText::new("(inherited from base)")
                            .small()
                            .color(INHERITED_COLOR),
                    );
                    // Add "Override" button to copy colliders to patch
                    if ui.button("Override").clicked()
                        && let Some(colliders) = display_table.get("colliders").cloned()
                            && let Some(mob) =
                                session.current_mob.as_mut().and_then(|v| v.as_table_mut())
                            {
                                mob.insert("colliders".to_string(), colliders);
                                *modified = true;
                            }
                } else if is_patch && is_patched {
                    ui.label(
                        egui::RichText::new("(overriding base)")
                            .small()
                            .color(PATCHED_COLOR),
                    );
                    // Add "Reset" button to remove colliders from patch
                    if ui.button("Reset to base").clicked()
                        && let Some(mob) =
                            session.current_mob.as_mut().and_then(|v| v.as_table_mut())
                        {
                            mob.remove("colliders");
                            *modified = true;
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
                                    if ui
                                        .add(
                                            egui::Button::new("🗑 Delete")
                                                .fill(egui::Color32::from_rgb(120, 60, 60)),
                                        )
                                        .clicked()
                                    {
                                        delete_index = Some(i);
                                    }
                                });
                            }
                            if let Some(table) = collider.as_table() {
                                // Shape info (read-only - changing shape type is complex)
                                if let Some(shape) = table.get("shape").and_then(|s| s.as_table()) {
                                    if let Some(radius) =
                                        shape.get("Circle").and_then(|v| v.as_float())
                                    {
                                        ui.label("Shape: Circle");
                                        if can_edit {
                                            let mut r = radius as f32;
                                            ui.horizontal(|ui| {
                                                ui.label("Radius:");
                                                if ui
                                                    .add(
                                                        egui::DragValue::new(&mut r)
                                                            .range(1.0..=100.0)
                                                            .speed(0.5),
                                                    )
                                                    .changed()
                                                {
                                                    update_collider_circle_radius(
                                                        session, i, r as f64,
                                                    );
                                                    *modified = true;
                                                }
                                            });
                                        } else {
                                            ui.label(format!("Radius: {}", radius));
                                        }
                                    } else if let Some(dims) =
                                        shape.get("Rectangle").and_then(|v| v.as_array())
                                    {
                                        let w = dims
                                            .first()
                                            .and_then(|v| v.as_float())
                                            .unwrap_or(10.0);
                                        let h =
                                            dims.get(1).and_then(|v| v.as_float()).unwrap_or(10.0);
                                        ui.label("Shape: Rectangle");
                                        if can_edit {
                                            let mut width = w as f32;
                                            let mut height = h as f32;
                                            ui.horizontal(|ui| {
                                                ui.label("W:");
                                                let w_changed = ui
                                                    .add(
                                                        egui::DragValue::new(&mut width)
                                                            .range(1.0..=200.0)
                                                            .speed(0.5),
                                                    )
                                                    .changed();
                                                ui.label("H:");
                                                let h_changed = ui
                                                    .add(
                                                        egui::DragValue::new(&mut height)
                                                            .range(1.0..=200.0)
                                                            .speed(0.5),
                                                    )
                                                    .changed();
                                                if w_changed || h_changed {
                                                    update_collider_rectangle_dims(
                                                        session,
                                                        i,
                                                        width as f64,
                                                        height as f64,
                                                    );
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
                                let pos_x = pos
                                    .and_then(|a| a.first())
                                    .and_then(|v| v.as_float())
                                    .unwrap_or(0.0);
                                let pos_y = pos
                                    .and_then(|a| a.get(1))
                                    .and_then(|v| v.as_float())
                                    .unwrap_or(0.0);
                                if can_edit {
                                    let mut x = pos_x as f32;
                                    let mut y = pos_y as f32;
                                    ui.horizontal(|ui| {
                                        ui.label("Position X:");
                                        let x_changed = ui
                                            .add(
                                                egui::DragValue::new(&mut x)
                                                    .range(-100.0..=100.0)
                                                    .speed(0.5),
                                            )
                                            .changed();
                                        ui.label("Y:");
                                        let y_changed = ui
                                            .add(
                                                egui::DragValue::new(&mut y)
                                                    .range(-100.0..=100.0)
                                                    .speed(0.5),
                                            )
                                            .changed();
                                        if x_changed || y_changed {
                                            update_collider_position(session, i, x as f64, y as f64);
                                            *modified = true;
                                        }
                                    });
                                } else {
                                    ui.label(format!("Position: ({}, {})", pos_x, pos_y));
                                }

                                // Rotation
                                let rot = table
                                    .get("rotation")
                                    .and_then(|v| v.as_float())
                                    .unwrap_or(0.0);
                                if can_edit {
                                    let mut r = rot as f32;
                                    ui.horizontal(|ui| {
                                        ui.label("Rotation:");
                                        if ui
                                            .add(
                                                egui::DragValue::new(&mut r)
                                                    .range(-180.0..=180.0)
                                                    .speed(1.0)
                                                    .suffix("°"),
                                            )
                                            .changed()
                                        {
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

/// Helper to mutate a collider at a specific index.
fn with_collider_mut<F>(session: &mut EditorSession, index: usize, f: F)
where
    F: FnOnce(&mut toml::value::Table),
{
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut())
        && let Some(colliders) = mob.get_mut("colliders").and_then(|v| v.as_array_mut())
            && let Some(collider) = colliders.get_mut(index).and_then(|v| v.as_table_mut()) {
                f(collider);
            }
}

/// Helper to mutate a collider's shape at a specific index.
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

/// Update a circle collider's radius.
fn update_collider_circle_radius(session: &mut EditorSession, index: usize, radius: f64) {
    with_collider_shape_mut(session, index, |shape| {
        shape.insert("Circle".to_string(), toml::Value::Float(radius));
    });
}

/// Update a rectangle collider's dimensions.
fn update_collider_rectangle_dims(
    session: &mut EditorSession,
    index: usize,
    width: f64,
    height: f64,
) {
    with_collider_shape_mut(session, index, |shape| {
        shape.insert(
            "Rectangle".to_string(),
            toml::Value::Array(vec![
                toml::Value::Float(width),
                toml::Value::Float(height),
            ]),
        );
    });
}

/// Update a collider's position.
fn update_collider_position(session: &mut EditorSession, index: usize, x: f64, y: f64) {
    with_collider_mut(session, index, |collider| {
        collider.insert(
            "position".to_string(),
            toml::Value::Array(vec![toml::Value::Float(x), toml::Value::Float(y)]),
        );
    });
}

/// Update a collider's rotation.
fn update_collider_rotation(session: &mut EditorSession, index: usize, rotation: f64) {
    with_collider_mut(session, index, |collider| {
        collider.insert("rotation".to_string(), toml::Value::Float(rotation));
    });
}

/// Add a new collider to the mob.
///
/// # Arguments
/// * `session` - The editor session
/// * `shape_type` - Either "Rectangle" or "Circle"
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
                _ => {
                    // Rectangle
                    shape.insert(
                        "Rectangle".to_string(),
                        toml::Value::Array(vec![
                            toml::Value::Float(20.0),
                            toml::Value::Float(20.0),
                        ]),
                    );
                }
            }
            collider.insert("shape".to_string(), toml::Value::Table(shape));

            // Default position and rotation
            collider.insert(
                "position".to_string(),
                toml::Value::Array(vec![toml::Value::Float(0.0), toml::Value::Float(0.0)]),
            );
            collider.insert("rotation".to_string(), toml::Value::Float(0.0));

            colliders.push(toml::Value::Table(collider));
        }
    }
}

/// Delete a collider by index.
fn delete_collider(session: &mut EditorSession, index: usize) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        if let Some(colliders) = mob.get_mut("colliders").and_then(|v| v.as_array_mut())
            && index < colliders.len() {
                colliders.remove(index);
            }
        // If colliders array is now empty, remove it entirely
        // This is important for patches - empty array overrides base, removing inherits from base
        if mob
            .get("colliders")
            .and_then(|v| v.as_array())
            .map(|a| a.is_empty())
            .unwrap_or(false)
        {
            mob.remove("colliders");
        }
    }
}
