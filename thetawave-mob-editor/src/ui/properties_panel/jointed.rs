//! Jointed mob editing functionality for the properties panel.
//!
//! This module handles rendering and editing of jointed mob configurations,
//! including joint settings, angle limits, and chain configurations.

use bevy_egui::egui;

use crate::data::EditorSession;
use crate::file::FileTreeState;

use super::fields::{INHERITED_COLOR, PATCHED_COLOR, header_color, render_patch_indicator};

/// Render the jointed mobs section
pub fn render_jointed_mobs_section(
    ui: &mut egui::Ui,
    display_table: &toml::value::Table,
    patch_table: &toml::value::Table,
    session: &mut EditorSession,
    is_patch: bool,
    modified: &mut bool,
    file_tree: &FileTreeState,
) {
    let is_patched = is_patch && patch_table.contains_key("jointed_mobs");
    let section_modified = session.is_field_modified("jointed_mobs");

    let header_text = egui::RichText::new("Jointed Mobs").color(header_color(ui, section_modified));
    egui::CollapsingHeader::new(header_text)
        .default_open(false)
        .show(ui, |ui| {
            // Patch status indicator
            ui.horizontal(|ui| {
                render_patch_indicator(ui, is_patched, is_patch);
                if is_patch && !is_patched {
                    let has_base_jointed = display_table.contains_key("jointed_mobs");
                    ui.label(
                        egui::RichText::new(if has_base_jointed {
                            "(inherited from base)"
                        } else {
                            "(no jointed mobs in base)"
                        })
                        .small()
                        .color(INHERITED_COLOR),
                    );
                    let button_label = if has_base_jointed {
                        "Override"
                    } else {
                        "Add Jointed Mobs"
                    };
                    if ui.button(button_label).clicked()
                        && let Some(mob) =
                            session.current_mob.as_mut().and_then(|v| v.as_table_mut())
                    {
                        // Copy from base if it exists, otherwise create empty array
                        let jointed_mobs = display_table
                            .get("jointed_mobs")
                            .cloned()
                            .unwrap_or_else(|| toml::Value::Array(vec![]));
                        mob.insert("jointed_mobs".to_string(), jointed_mobs);
                        *modified = true;
                    }
                } else if is_patch && is_patched {
                    ui.label(
                        egui::RichText::new("(overriding base)")
                            .small()
                            .color(PATCHED_COLOR),
                    );
                    if ui.button("Reset to base").clicked()
                        && let Some(mob) =
                            session.current_mob.as_mut().and_then(|v| v.as_table_mut())
                    {
                        mob.remove("jointed_mobs");
                        *modified = true;
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

                    let key = table
                        .get("key")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unnamed");
                    let is_selected = session.selected_jointed_mob == Some(i);
                    let item_modified = session.is_array_item_modified("jointed_mobs", i);

                    // Highlight selected and/or modified joint
                    let header_text = if is_selected {
                        egui::RichText::new(format!("Joint: {} *", key))
                            .strong()
                            .color(egui::Color32::YELLOW)
                    } else {
                        egui::RichText::new(format!("Joint: {}", key))
                            .color(header_color(ui, item_modified))
                    };

                    egui::CollapsingHeader::new(header_text)
                        .id_salt(format!("jointed_mob_{}", i))
                        .default_open(false)
                        .show(ui, |ui| {
                            // Select button
                            ui.horizontal(|ui| {
                                if ui
                                    .button(if is_selected { "Deselect" } else { "Select" })
                                    .clicked()
                                {
                                    session.selected_jointed_mob =
                                        if is_selected { None } else { Some(i) };
                                }

                                if can_edit
                                    && ui
                                        .add(
                                            egui::Button::new("🗑")
                                                .fill(crate::ui::DELETE_BUTTON_COLOR),
                                        )
                                        .on_hover_text("Delete joint")
                                        .clicked()
                                {
                                    delete_index = Some(i);
                                }
                            });
                            ui.separator();

                            if can_edit {
                                render_jointed_mob_fields(
                                    ui, session, i, table, modified, file_tree,
                                );
                            } else {
                                // Read-only display
                                ui.label(format!("Key: {}", key));
                                let mob_ref = table
                                    .get("mob_ref")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("(none)");
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
                    } else if let Some(selected) = session.selected_jointed_mob
                        && selected > idx
                    {
                        session.selected_jointed_mob = Some(selected - 1);
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

/// Render editable fields for a single jointed mob
fn render_jointed_mob_fields(
    ui: &mut egui::Ui,
    session: &mut EditorSession,
    index: usize,
    table: &toml::value::Table,
    modified: &mut bool,
    file_tree: &FileTreeState,
) {
    // Key field
    let key_value = table.get("key").and_then(|v| v.as_str()).unwrap_or("");
    ui.horizontal(|ui| {
        ui.label("Key:");
        let mut value = key_value.to_string();
        if ui.text_edit_singleline(&mut value).changed() {
            set_jointed_mob_field(session, index, "key", toml::Value::String(value));
            *modified = true;
        }
    });

    // Mob ref field (dropdown with categories)
    let mob_ref = table.get("mob_ref").and_then(|v| v.as_str()).unwrap_or("");
    let (base_refs, extended_refs) = file_tree.get_categorized_mob_refs();
    ui.horizontal(|ui| {
        ui.label("Mob Ref:");
        let mut selected = mob_ref.to_string();
        egui::ComboBox::from_id_salt(format!("mob_ref_{}", index))
            .selected_text(if selected.is_empty() {
                "(none)"
            } else {
                &selected
            })
            .show_ui(ui, |ui| {
                if ui.selectable_label(selected.is_empty(), "(none)").clicked() {
                    selected.clear();
                }
                // Base mobs section
                if !base_refs.is_empty() {
                    ui.separator();
                    ui.label(egui::RichText::new("Base Mobs").strong().small());
                    for ref_name in &base_refs {
                        if ui
                            .selectable_label(selected == *ref_name, ref_name)
                            .clicked()
                        {
                            selected = ref_name.clone();
                        }
                    }
                }
                // Extended mobs section
                if !extended_refs.is_empty() {
                    ui.separator();
                    ui.label(egui::RichText::new("Extended Mobs").strong().small());
                    for ref_name in &extended_refs {
                        if ui
                            .selectable_label(selected == *ref_name, ref_name)
                            .clicked()
                        {
                            selected = ref_name.clone();
                        }
                    }
                }
            });
        if selected != mob_ref {
            set_jointed_mob_field(session, index, "mob_ref", toml::Value::String(selected));
            *modified = true;
        }
    });

    // Offset position
    let offset = get_jointed_vec2(table, "offset_pos");
    ui.horizontal(|ui| {
        ui.label("Offset Pos:");
        let mut x = offset.0;
        let mut y = offset.1;
        let x_changed = ui
            .add(egui::DragValue::new(&mut x).speed(0.5).prefix("x: "))
            .changed();
        let y_changed = ui
            .add(egui::DragValue::new(&mut y).speed(0.5).prefix("y: "))
            .changed();
        if x_changed || y_changed {
            set_jointed_mob_vec2(session, index, "offset_pos", x, y);
            *modified = true;
        }
    });

    // Offset rotation
    let offset_rot = table
        .get("offset_rot")
        .and_then(|v| v.as_float())
        .unwrap_or(0.0) as f32;
    ui.horizontal(|ui| {
        ui.label("Offset Rot:");
        let mut rot = offset_rot;
        if ui
            .add(egui::DragValue::new(&mut rot).speed(1.0).suffix("°"))
            .changed()
        {
            set_jointed_mob_field(session, index, "offset_rot", toml::Value::Float(rot as f64));
            *modified = true;
        }
    });

    // Anchor 1 position
    let anchor1 = get_jointed_vec2(table, "anchor_1_pos");
    ui.horizontal(|ui| {
        ui.label("Anchor 1:");
        let mut x = anchor1.0;
        let mut y = anchor1.1;
        let x_changed = ui
            .add(egui::DragValue::new(&mut x).speed(0.5).prefix("x: "))
            .changed();
        let y_changed = ui
            .add(egui::DragValue::new(&mut y).speed(0.5).prefix("y: "))
            .changed();
        if x_changed || y_changed {
            set_jointed_mob_vec2(session, index, "anchor_1_pos", x, y);
            *modified = true;
        }
    });

    // Anchor 2 position
    let anchor2 = get_jointed_vec2(table, "anchor_2_pos");
    ui.horizontal(|ui| {
        ui.label("Anchor 2:");
        let mut x = anchor2.0;
        let mut y = anchor2.1;
        let x_changed = ui
            .add(egui::DragValue::new(&mut x).speed(0.5).prefix("x: "))
            .changed();
        let y_changed = ui
            .add(egui::DragValue::new(&mut y).speed(0.5).prefix("y: "))
            .changed();
        if x_changed || y_changed {
            set_jointed_mob_vec2(session, index, "anchor_2_pos", x, y);
            *modified = true;
        }
    });

    // Compliance (very small values)
    let compliance = table
        .get("compliance")
        .and_then(|v| v.as_float())
        .unwrap_or(0.0) as f32;
    ui.horizontal(|ui| {
        ui.label("Compliance:");
        let mut value = compliance;
        if ui
            .add(
                egui::DragValue::new(&mut value)
                    .speed(0.0000001)
                    .range(0.0..=1.0)
                    .min_decimals(7)
                    .max_decimals(10),
            )
            .changed()
        {
            set_jointed_mob_field(
                session,
                index,
                "compliance",
                toml::Value::Float(value as f64),
            );
            *modified = true;
        }
    });

    // Angle limits section
    render_angle_limit_subsection(ui, session, index, table, modified);

    // Chain configuration section
    render_chain_subsection(ui, session, index, table, modified);
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
                if let Some(angle_table) = table.get("angle_limit_range").and_then(|v| v.as_table())
                {
                    let min = angle_table
                        .get("min")
                        .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
                        .unwrap_or(-45.0) as f32;
                    let max = angle_table
                        .get("max")
                        .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
                        .unwrap_or(45.0) as f32;
                    let torque = angle_table
                        .get("torque")
                        .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
                        .unwrap_or(0.001) as f32;

                    ui.horizontal(|ui| {
                        ui.label("Min:");
                        let mut value = min;
                        if ui
                            .add(
                                egui::DragValue::new(&mut value)
                                    .speed(1.0)
                                    .range(-180.0..=180.0)
                                    .suffix("°"),
                            )
                            .changed()
                        {
                            set_jointed_nested_field(
                                session,
                                index,
                                "angle_limit_range",
                                "min",
                                toml::Value::Float(value as f64),
                            );
                            *modified = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Max:");
                        let mut value = max;
                        if ui
                            .add(
                                egui::DragValue::new(&mut value)
                                    .speed(1.0)
                                    .range(-180.0..=180.0)
                                    .suffix("°"),
                            )
                            .changed()
                        {
                            set_jointed_nested_field(
                                session,
                                index,
                                "angle_limit_range",
                                "max",
                                toml::Value::Float(value as f64),
                            );
                            *modified = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Torque:");
                        let mut value = torque;
                        if ui
                            .add(
                                egui::DragValue::new(&mut value)
                                    .speed(0.0001)
                                    .range(0.0..=1.0)
                                    .min_decimals(4),
                            )
                            .changed()
                        {
                            set_jointed_nested_field(
                                session,
                                index,
                                "angle_limit_range",
                                "torque",
                                toml::Value::Float(value as f64),
                            );
                            *modified = true;
                        }
                    });

                    if ui.button("Remove Angle Limits").clicked() {
                        remove_jointed_mob_field(session, index, "angle_limit_range");
                        *modified = true;
                    }
                }
            } else {
                ui.label(
                    egui::RichText::new("No angle limits")
                        .italics()
                        .color(egui::Color32::GRAY),
                );
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
                    let length = chain_table
                        .get("length")
                        .and_then(|v| v.as_integer())
                        .unwrap_or(1) as i32;
                    let pos_offset = get_jointed_vec2(chain_table, "pos_offset");
                    let anchor_offset = get_jointed_vec2(chain_table, "anchor_offset");

                    ui.horizontal(|ui| {
                        ui.label("Length:");
                        let mut value = length;
                        if ui
                            .add(egui::DragValue::new(&mut value).range(1..=100))
                            .changed()
                        {
                            set_jointed_nested_field(
                                session,
                                index,
                                "chain",
                                "length",
                                toml::Value::Integer(value as i64),
                            );
                            *modified = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Pos Offset:");
                        let mut x = pos_offset.0;
                        let mut y = pos_offset.1;
                        let x_changed = ui
                            .add(egui::DragValue::new(&mut x).speed(0.5).prefix("x: "))
                            .changed();
                        let y_changed = ui
                            .add(egui::DragValue::new(&mut y).speed(0.5).prefix("y: "))
                            .changed();
                        if x_changed || y_changed {
                            set_jointed_nested_vec2(session, index, "chain", "pos_offset", x, y);
                            *modified = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Anchor Offset:");
                        let mut x = anchor_offset.0;
                        let mut y = anchor_offset.1;
                        let x_changed = ui
                            .add(egui::DragValue::new(&mut x).speed(0.5).prefix("x: "))
                            .changed();
                        let y_changed = ui
                            .add(egui::DragValue::new(&mut y).speed(0.5).prefix("y: "))
                            .changed();
                        if x_changed || y_changed {
                            set_jointed_nested_vec2(session, index, "chain", "anchor_offset", x, y);
                            *modified = true;
                        }
                    });

                    // Random chain subsection
                    ui.separator();
                    let has_random = chain_table.get("random_chain").is_some();
                    if has_random {
                        if let Some(random_table) =
                            chain_table.get("random_chain").and_then(|v| v.as_table())
                        {
                            ui.label(egui::RichText::new("Random Chain:").strong());

                            let min_length = random_table
                                .get("min_length")
                                .and_then(|v| v.as_integer())
                                .unwrap_or(1) as i32;
                            let end_chance = random_table
                                .get("end_chance")
                                .and_then(|v| v.as_float())
                                .unwrap_or(0.1) as f32;

                            ui.horizontal(|ui| {
                                ui.label("Min Length:");
                                let mut value = min_length;
                                if ui
                                    .add(egui::DragValue::new(&mut value).range(1..=100))
                                    .changed()
                                {
                                    set_jointed_deep_nested_field(
                                        session,
                                        index,
                                        "chain",
                                        "random_chain",
                                        "min_length",
                                        toml::Value::Integer(value as i64),
                                    );
                                    *modified = true;
                                }
                            });

                            ui.horizontal(|ui| {
                                ui.label("End Chance:");
                                let mut value = end_chance;
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut value)
                                            .speed(0.01)
                                            .range(0.0..=1.0)
                                            .min_decimals(2),
                                    )
                                    .changed()
                                {
                                    set_jointed_deep_nested_field(
                                        session,
                                        index,
                                        "chain",
                                        "random_chain",
                                        "end_chance",
                                        toml::Value::Float(value as f64),
                                    );
                                    *modified = true;
                                }
                            });

                            if ui.button("Remove Random").clicked() {
                                remove_jointed_nested_field(
                                    session,
                                    index,
                                    "chain",
                                    "random_chain",
                                );
                                *modified = true;
                            }
                        }
                    } else {
                        ui.label(
                            egui::RichText::new("Random: Not configured")
                                .small()
                                .color(egui::Color32::GRAY),
                        );
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
                ui.label(
                    egui::RichText::new("No chain configuration")
                        .italics()
                        .color(egui::Color32::GRAY),
                );
                if ui.button("Add Chain").clicked() {
                    add_chain_to_jointed_mob(session, index);
                    *modified = true;
                }
            }
        });
}

// =============================================================================
// Helper functions for jointed mob manipulation
// =============================================================================

/// Get Vec2 from a jointed mob table
pub fn get_jointed_vec2(table: &toml::value::Table, key: &str) -> (f32, f32) {
    if let Some(arr) = table.get(key).and_then(|v| v.as_array()) {
        let x = arr
            .first()
            .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
            .unwrap_or(0.0) as f32;
        let y = arr
            .get(1)
            .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
            .unwrap_or(0.0) as f32;
        (x, y)
    } else {
        (0.0, 0.0)
    }
}

/// Set a field on a jointed mob at the given index
fn set_jointed_mob_field(
    session: &mut EditorSession,
    index: usize,
    field: &str,
    value: toml::Value,
) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut())
        && let Some(arr) = mob.get_mut("jointed_mobs").and_then(|v| v.as_array_mut())
        && let Some(joint) = arr.get_mut(index).and_then(|v| v.as_table_mut())
    {
        joint.insert(field.to_string(), value);
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
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut())
        && let Some(arr) = mob.get_mut("jointed_mobs").and_then(|v| v.as_array_mut())
        && let Some(joint) = arr.get_mut(index).and_then(|v| v.as_table_mut())
    {
        joint.remove(field);
    }
}

/// Set a nested field on a jointed mob (e.g., angle_limit_range.min)
fn set_jointed_nested_field(
    session: &mut EditorSession,
    index: usize,
    parent: &str,
    field: &str,
    value: toml::Value,
) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut())
        && let Some(arr) = mob.get_mut("jointed_mobs").and_then(|v| v.as_array_mut())
        && let Some(joint) = arr.get_mut(index).and_then(|v| v.as_table_mut())
        && let Some(parent_table) = joint.get_mut(parent).and_then(|v| v.as_table_mut())
    {
        parent_table.insert(field.to_string(), value);
    }
}

/// Set a nested Vec2 field on a jointed mob (e.g., chain.pos_offset)
fn set_jointed_nested_vec2(
    session: &mut EditorSession,
    index: usize,
    parent: &str,
    field: &str,
    x: f32,
    y: f32,
) {
    let arr = toml::Value::Array(vec![
        toml::Value::Float(x as f64),
        toml::Value::Float(y as f64),
    ]);
    set_jointed_nested_field(session, index, parent, field, arr);
}

/// Remove a nested field from a jointed mob
fn remove_jointed_nested_field(
    session: &mut EditorSession,
    index: usize,
    parent: &str,
    field: &str,
) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut())
        && let Some(arr) = mob.get_mut("jointed_mobs").and_then(|v| v.as_array_mut())
        && let Some(joint) = arr.get_mut(index).and_then(|v| v.as_table_mut())
        && let Some(parent_table) = joint.get_mut(parent).and_then(|v| v.as_table_mut())
    {
        parent_table.remove(field);
    }
}

/// Set a deeply nested field (e.g., chain.random_chain.min_length)
fn set_jointed_deep_nested_field(
    session: &mut EditorSession,
    index: usize,
    parent: &str,
    nested: &str,
    field: &str,
    value: toml::Value,
) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut())
        && let Some(arr) = mob.get_mut("jointed_mobs").and_then(|v| v.as_array_mut())
        && let Some(joint) = arr.get_mut(index).and_then(|v| v.as_table_mut())
        && let Some(parent_table) = joint.get_mut(parent).and_then(|v| v.as_table_mut())
        && let Some(nested_table) = parent_table.get_mut(nested).and_then(|v| v.as_table_mut())
    {
        nested_table.insert(field.to_string(), value);
    }
}

/// Delete a jointed mob at the given index
fn delete_jointed_mob(session: &mut EditorSession, index: usize) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut())
        && let Some(arr) = mob.get_mut("jointed_mobs").and_then(|v| v.as_array_mut())
    {
        if index < arr.len() {
            arr.remove(index);
        }
        // Clean up empty array
        if arr.is_empty() {
            mob.remove("jointed_mobs");
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
            new_joint.insert(
                "key".to_string(),
                toml::Value::String(format!("joint_{}", arr.len())),
            );
            new_joint.insert("mob_ref".to_string(), toml::Value::String(String::new()));
            new_joint.insert(
                "offset_pos".to_string(),
                toml::Value::Array(vec![toml::Value::Float(0.0), toml::Value::Float(-10.0)]),
            );
            new_joint.insert(
                "anchor_1_pos".to_string(),
                toml::Value::Array(vec![toml::Value::Float(0.0), toml::Value::Float(0.0)]),
            );
            new_joint.insert(
                "anchor_2_pos".to_string(),
                toml::Value::Array(vec![toml::Value::Float(0.0), toml::Value::Float(0.0)]),
            );
            new_joint.insert("compliance".to_string(), toml::Value::Float(0.000001));

            arr.push(toml::Value::Table(new_joint));
        }
    }
}

/// Add angle_limit_range to a jointed mob
fn add_angle_limit_to_jointed_mob(session: &mut EditorSession, index: usize) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut())
        && let Some(arr) = mob.get_mut("jointed_mobs").and_then(|v| v.as_array_mut())
        && let Some(joint) = arr.get_mut(index).and_then(|v| v.as_table_mut())
    {
        let mut angle_table = toml::value::Table::new();
        angle_table.insert("min".to_string(), toml::Value::Float(-45.0));
        angle_table.insert("max".to_string(), toml::Value::Float(45.0));
        angle_table.insert("torque".to_string(), toml::Value::Float(0.001));
        joint.insert(
            "angle_limit_range".to_string(),
            toml::Value::Table(angle_table),
        );
    }
}

/// Add chain configuration to a jointed mob
fn add_chain_to_jointed_mob(session: &mut EditorSession, index: usize) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut())
        && let Some(arr) = mob.get_mut("jointed_mobs").and_then(|v| v.as_array_mut())
        && let Some(joint) = arr.get_mut(index).and_then(|v| v.as_table_mut())
    {
        let mut chain_table = toml::value::Table::new();
        chain_table.insert("length".to_string(), toml::Value::Integer(3));
        chain_table.insert(
            "pos_offset".to_string(),
            toml::Value::Array(vec![toml::Value::Float(0.0), toml::Value::Float(-5.0)]),
        );
        chain_table.insert(
            "anchor_offset".to_string(),
            toml::Value::Array(vec![toml::Value::Float(0.0), toml::Value::Float(0.0)]),
        );
        joint.insert("chain".to_string(), toml::Value::Table(chain_table));
    }
}

/// Add random_chain to a jointed mob's chain config
fn add_random_chain_to_jointed_mob(session: &mut EditorSession, index: usize) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut())
        && let Some(arr) = mob.get_mut("jointed_mobs").and_then(|v| v.as_array_mut())
        && let Some(joint) = arr.get_mut(index).and_then(|v| v.as_table_mut())
        && let Some(chain) = joint.get_mut("chain").and_then(|v| v.as_table_mut())
    {
        let mut random_table = toml::value::Table::new();
        random_table.insert("min_length".to_string(), toml::Value::Integer(1));
        random_table.insert("end_chance".to_string(), toml::Value::Float(0.15));
        chain.insert("random_chain".to_string(), toml::Value::Table(random_table));
    }
}
