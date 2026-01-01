//! Type-specific rendering functions for behavior tree nodes.

use bevy_egui::egui;
use thetawave_mobs::{BehaviorNodeType, MobBehaviorCategory, MobBehaviorVariant, BY_CATEGORY};

use crate::data::EditorSession;

use super::super::fields::{INDENT_SPACING, INHERITED_COLOR};
use super::action_ops::{
    add_action_behavior, change_action_behavior_type, delete_action_behavior,
    move_action_behavior, remove_action_behavior_param, set_action_behavior_param,
};
use super::navigation::set_behavior_node_field;
use super::transmit_ops::{
    add_transmit_nested_behavior, change_transmit_nested_behavior_type,
    delete_transmit_nested_behavior, move_transmit_nested_behavior, set_transmit_nested_param,
};
use super::tree_ops::{
    add_behavior_child, add_if_else_child, add_if_then_child, add_if_then_condition,
    add_while_child, add_while_condition, remove_if_else_child,
};

/// Format a header string for a behavior node
pub fn format_node_header(
    table: &toml::value::Table,
    node_type: Option<BehaviorNodeType>,
) -> String {
    match node_type {
        Some(BehaviorNodeType::Action) => {
            let name = table.get("name").and_then(|v| v.as_str()).unwrap_or("");
            if name.is_empty() {
                "Action".to_string()
            } else {
                format!("Action: {}", name)
            }
        }
        Some(BehaviorNodeType::Wait) => {
            let seconds = table
                .get("seconds")
                .and_then(|v| v.as_float())
                .unwrap_or(1.0);
            format!("Wait: {:.1}s", seconds)
        }
        Some(t) => t.as_ref().to_string(),
        None => "Unknown".to_string(),
    }
}

/// Render a control node (Forever, Sequence, Fallback) with children
pub fn render_control_node(
    ui: &mut egui::Ui,
    session: &mut EditorSession,
    table: &toml::value::Table,
    path: &[usize],
    can_edit: bool,
    depth: usize,
    modified: &mut bool,
    render_fn: impl Fn(
        &mut egui::Ui,
        &mut EditorSession,
        &toml::Value,
        &[usize],
        bool,
        usize,
        &mut bool,
    ),
) {
    let children = table
        .get("children")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    ui.label(
        egui::RichText::new(format!("Children: {}", children.len()))
            .small()
            .color(INHERITED_COLOR),
    );

    for (i, child) in children.iter().enumerate() {
        let mut child_path = path.to_vec();
        child_path.push(i);
        render_fn(ui, session, child, &child_path, can_edit, depth + 1, modified);
    }

    if can_edit && ui.button("+ Add Child").clicked() {
        add_behavior_child(session, path);
        *modified = true;
    }
}

/// Render a While node (has condition and child)
pub fn render_while_node(
    ui: &mut egui::Ui,
    session: &mut EditorSession,
    table: &toml::value::Table,
    path: &[usize],
    can_edit: bool,
    depth: usize,
    modified: &mut bool,
    render_fn: impl Fn(
        &mut egui::Ui,
        &mut EditorSession,
        &toml::Value,
        &[usize],
        bool,
        usize,
        &mut bool,
    ),
) {
    // Condition (optional)
    ui.label(egui::RichText::new("Condition:").small());
    if let Some(condition) = table.get("condition") {
        let mut cond_path = path.to_vec();
        cond_path.push(0); // Use index 0 for condition
        render_fn(ui, session, condition, &cond_path, can_edit, depth + 1, modified);
    } else {
        ui.label(
            egui::RichText::new("(no condition)")
                .italics()
                .color(INHERITED_COLOR),
        );
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
        render_fn(ui, session, child, &child_path, can_edit, depth + 1, modified);
    } else {
        ui.label(
            egui::RichText::new("(no child)")
                .italics()
                .color(INHERITED_COLOR),
        );
        if can_edit && ui.small_button("Add child").clicked() {
            add_while_child(session, path);
            *modified = true;
        }
    }
}

/// Render an IfThen node (has condition, then_child, and optional else_child)
pub fn render_if_then_node(
    ui: &mut egui::Ui,
    session: &mut EditorSession,
    table: &toml::value::Table,
    path: &[usize],
    can_edit: bool,
    depth: usize,
    modified: &mut bool,
    render_fn: impl Fn(
        &mut egui::Ui,
        &mut EditorSession,
        &toml::Value,
        &[usize],
        bool,
        usize,
        &mut bool,
    ),
) {
    // Condition
    ui.label(egui::RichText::new("Condition:").small());
    if let Some(condition) = table.get("condition") {
        let mut cond_path = path.to_vec();
        cond_path.push(0);
        render_fn(ui, session, condition, &cond_path, can_edit, depth + 1, modified);
    } else {
        ui.label(
            egui::RichText::new("(no condition)")
                .italics()
                .color(INHERITED_COLOR),
        );
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
        render_fn(ui, session, then_child, &then_path, can_edit, depth + 1, modified);
    } else {
        ui.label(
            egui::RichText::new("(no then branch)")
                .italics()
                .color(INHERITED_COLOR),
        );
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
        render_fn(ui, session, else_child, &else_path, can_edit, depth + 1, modified);

        if can_edit && ui.small_button("Remove else branch").clicked() {
            remove_if_else_child(session, path);
            *modified = true;
        }
    } else {
        ui.label(
            egui::RichText::new("(no else branch)")
                .italics()
                .color(INHERITED_COLOR),
        );
        if can_edit && ui.small_button("Add else branch").clicked() {
            add_if_else_child(session, path);
            *modified = true;
        }
    }
}

/// Render a Wait node
pub fn render_wait_node(
    ui: &mut egui::Ui,
    session: &mut EditorSession,
    table: &toml::value::Table,
    path: &[usize],
    can_edit: bool,
    modified: &mut bool,
) {
    let seconds = table
        .get("seconds")
        .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
        .unwrap_or(1.0) as f32;

    ui.horizontal(|ui| {
        ui.label("Seconds:");
        if can_edit {
            let mut value = seconds;
            if ui
                .add(
                    egui::DragValue::new(&mut value)
                        .speed(0.1)
                        .range(0.0..=100.0),
                )
                .changed()
            {
                set_behavior_node_field(session, path, "seconds", toml::Value::Float(value as f64));
                *modified = true;
            }
        } else {
            ui.label(format!("{:.2}", seconds));
        }
    });
}

/// Render an Action node with its behaviors list
pub fn render_action_node(
    ui: &mut egui::Ui,
    session: &mut EditorSession,
    table: &toml::value::Table,
    path: &[usize],
    can_edit: bool,
    modified: &mut bool,
) {
    // Name field
    let name = table
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
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
    let behaviors = table
        .get("behaviors")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    if behaviors.is_empty() {
        ui.label(
            egui::RichText::new("(no behaviors)")
                .italics()
                .color(INHERITED_COLOR),
        );
    } else {
        let mut delete_index: Option<usize> = None;

        for (i, behavior) in behaviors.iter().enumerate() {
            let Some(behavior_table) = behavior.as_table() else {
                continue;
            };

            let action_type_str = behavior_table
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");
            let action_type: Option<MobBehaviorVariant> = action_type_str.parse().ok();

            ui.horizontal(|ui| {
                ui.add_space(INDENT_SPACING);
                ui.label(format!("{}.", i + 1));

                if can_edit {
                    // Action type combo
                    let mut current_action = action_type;
                    let display_text = match &current_action {
                        Some(a) => a.as_ref(),
                        None => "Unknown",
                    };
                    egui::ComboBox::from_id_salt(format!("action_combo_{:?}_{}", path, i))
                        .selected_text(display_text)
                        .width(120.0)
                        .show_ui(ui, |ui| {
                            for (category, actions) in BY_CATEGORY.iter() {
                                ui.label(
                                    egui::RichText::new(category.as_str())
                                        .small()
                                        .color(INHERITED_COLOR),
                                );
                                for action in actions {
                                    let is_selected = current_action == Some(*action);
                                    if ui.selectable_label(is_selected, action.as_ref()).clicked()
                                    {
                                        current_action = Some(*action);
                                    }
                                }
                                ui.separator();
                            }
                        });
                    if current_action != action_type
                        && let Some(new_action) = current_action
                    {
                        change_action_behavior_type(session, path, i, new_action);
                        *modified = true;
                    }

                    // Render action-specific parameters
                    render_action_parameters(
                        ui,
                        session,
                        behavior_table,
                        path,
                        i,
                        current_action,
                        modified,
                    );

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
                    let display_text = match &action_type {
                        Some(a) => a.as_ref(),
                        None => action_type_str,
                    };
                    ui.label(display_text);
                    render_action_parameters_readonly(ui, behavior_table, action_type);
                }
            });

            // Render nested behaviors for TransmitMobBehavior
            if action_type == Some(MobBehaviorVariant::TransmitMobBehavior) {
                render_transmit_nested_behaviors(
                    ui,
                    session,
                    behavior_table,
                    path,
                    i,
                    can_edit,
                    modified,
                );
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
    action_type: Option<MobBehaviorVariant>,
    modified: &mut bool,
) {
    match action_type {
        Some(MobBehaviorVariant::MoveTo) => {
            let x = behavior_table
                .get("x")
                .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
                .unwrap_or(0.0) as f32;
            let y = behavior_table
                .get("y")
                .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
                .unwrap_or(0.0) as f32;

            let mut new_x = x;
            let mut new_y = y;
            let x_changed = ui
                .add(egui::DragValue::new(&mut new_x).speed(0.5).prefix("x: "))
                .changed();
            let y_changed = ui
                .add(egui::DragValue::new(&mut new_y).speed(0.5).prefix("y: "))
                .changed();
            if x_changed || y_changed {
                set_action_behavior_param(
                    session,
                    path,
                    behavior_index,
                    "x",
                    toml::Value::Float(new_x as f64),
                );
                set_action_behavior_param(
                    session,
                    path,
                    behavior_index,
                    "y",
                    toml::Value::Float(new_y as f64),
                );
                *modified = true;
            }
        }
        Some(MobBehaviorVariant::DoForTime) => {
            let seconds = behavior_table
                .get("seconds")
                .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
                .unwrap_or(1.0) as f32;

            let mut value = seconds;
            if ui
                .add(
                    egui::DragValue::new(&mut value)
                        .speed(0.1)
                        .range(0.0..=100.0)
                        .prefix("s: "),
                )
                .changed()
            {
                set_action_behavior_param(
                    session,
                    path,
                    behavior_index,
                    "seconds",
                    toml::Value::Float(value as f64),
                );
                *modified = true;
            }
        }
        Some(MobBehaviorVariant::SpawnMob | MobBehaviorVariant::SpawnProjectile) => {
            let keys = behavior_table
                .get("keys")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();

            let mut value = keys;
            if ui
                .add(
                    egui::TextEdit::singleline(&mut value)
                        .desired_width(100.0)
                        .hint_text("keys"),
                )
                .changed()
            {
                let keys_arr: Vec<toml::Value> = value
                    .split(',')
                    .map(|s| toml::Value::String(s.trim().to_string()))
                    .filter(|v| !v.as_str().unwrap_or("").is_empty())
                    .collect();
                if keys_arr.is_empty() {
                    remove_action_behavior_param(session, path, behavior_index, "keys");
                } else {
                    set_action_behavior_param(
                        session,
                        path,
                        behavior_index,
                        "keys",
                        toml::Value::Array(keys_arr),
                    );
                }
                *modified = true;
            }
        }
        Some(MobBehaviorVariant::TransmitMobBehavior) => {
            let mob_type = behavior_table
                .get("mob_type")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let mut value = mob_type;
            if ui
                .add(
                    egui::TextEdit::singleline(&mut value)
                        .desired_width(100.0)
                        .hint_text("mob_type"),
                )
                .changed()
            {
                set_action_behavior_param(
                    session,
                    path,
                    behavior_index,
                    "mob_type",
                    toml::Value::String(value),
                );
                *modified = true;
            }
        }
        _ => {
            // No parameters for simple actions
        }
    }
}

/// Render parameters in read-only mode
fn render_action_parameters_readonly(
    ui: &mut egui::Ui,
    behavior_table: &toml::value::Table,
    action_type: Option<MobBehaviorVariant>,
) {
    match action_type {
        Some(MobBehaviorVariant::MoveTo) => {
            let x = behavior_table
                .get("x")
                .and_then(|v| v.as_float())
                .unwrap_or(0.0);
            let y = behavior_table
                .get("y")
                .and_then(|v| v.as_float())
                .unwrap_or(0.0);
            ui.label(format!("({:.1}, {:.1})", x, y));
        }
        Some(MobBehaviorVariant::DoForTime) => {
            let seconds = behavior_table
                .get("seconds")
                .and_then(|v| v.as_float())
                .unwrap_or(1.0);
            ui.label(format!("{:.1}s", seconds));
        }
        Some(MobBehaviorVariant::SpawnMob | MobBehaviorVariant::SpawnProjectile) => {
            if let Some(keys) = behavior_table.get("keys").and_then(|v| v.as_array()) {
                let keys_str: Vec<_> = keys.iter().filter_map(|v| v.as_str()).collect();
                ui.label(format!("[{}]", keys_str.join(", ")));
            }
        }
        Some(MobBehaviorVariant::TransmitMobBehavior) => {
            let mob_type = behavior_table
                .get("mob_type")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            ui.label(format!("▶ {}", mob_type));
        }
        _ => {}
    }
}

/// Render a Trigger node
pub fn render_trigger_node(
    ui: &mut egui::Ui,
    session: &mut EditorSession,
    table: &toml::value::Table,
    path: &[usize],
    can_edit: bool,
    modified: &mut bool,
) {
    let trigger_type = table
        .get("trigger_type")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    ui.horizontal(|ui| {
        ui.label("Trigger Type:");
        if can_edit {
            let mut value = trigger_type;
            if ui.text_edit_singleline(&mut value).changed() {
                set_behavior_node_field(
                    session,
                    path,
                    "trigger_type",
                    toml::Value::String(value),
                );
                *modified = true;
            }
        } else {
            ui.label(&trigger_type);
        }
    });

    ui.label(
        egui::RichText::new("(Trigger nodes are for future use)")
            .small()
            .color(INHERITED_COLOR),
    );
}

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
            ui.label(
                egui::RichText::new("(none)")
                    .italics()
                    .color(INHERITED_COLOR),
            );
        });
    } else {
        let mut delete_nested: Option<usize> = None;

        for (j, nested) in nested_behaviors.iter().enumerate() {
            let Some(nested_table) = nested.as_table() else {
                continue;
            };

            let nested_action_str = nested_table
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");
            let nested_action: Option<MobBehaviorVariant> = nested_action_str.parse().ok();

            ui.horizontal(|ui| {
                ui.add_space(40.0);
                ui.label(format!("{}.", j + 1));

                if can_edit {
                    // Action type combo for nested behavior
                    let mut current_nested = nested_action;
                    let display_text = match &current_nested {
                        Some(a) => a.as_ref(),
                        None => "Unknown",
                    };
                    egui::ComboBox::from_id_salt(format!(
                        "nested_action_{:?}_{}_{}",
                        path, behavior_index, j
                    ))
                    .selected_text(display_text)
                    .width(100.0)
                    .show_ui(ui, |ui| {
                        // Only show simple actions for nested behaviors (no TransmitMobBehavior)
                        for (category, actions) in BY_CATEGORY.iter() {
                            if *category == MobBehaviorCategory::Communication {
                                continue; // Skip TransmitMobBehavior
                            }
                            ui.label(
                                egui::RichText::new(category.as_str())
                                    .small()
                                    .color(INHERITED_COLOR),
                            );
                            for action in actions {
                                let is_selected = current_nested == Some(*action);
                                if ui.selectable_label(is_selected, action.as_ref()).clicked() {
                                    current_nested = Some(*action);
                                }
                            }
                            ui.separator();
                        }
                    });
                    if current_nested != nested_action
                        && let Some(new_action) = current_nested
                    {
                        change_transmit_nested_behavior_type(
                            session,
                            path,
                            behavior_index,
                            j,
                            new_action,
                        );
                        *modified = true;
                    }

                    // Render parameters for nested behavior
                    render_transmit_nested_params(
                        ui,
                        session,
                        nested_table,
                        path,
                        behavior_index,
                        j,
                        current_nested,
                        modified,
                    );

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
                    let display = match &nested_action {
                        Some(a) => a.as_ref(),
                        None => nested_action_str,
                    };
                    ui.label(display);
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
    action_type: Option<MobBehaviorVariant>,
    modified: &mut bool,
) {
    match action_type {
        Some(MobBehaviorVariant::MoveTo) => {
            let x = nested_table
                .get("x")
                .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
                .unwrap_or(0.0) as f32;
            let y = nested_table
                .get("y")
                .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
                .unwrap_or(0.0) as f32;

            let mut new_x = x;
            let mut new_y = y;
            let x_changed = ui
                .add(egui::DragValue::new(&mut new_x).speed(0.5).prefix("x: "))
                .changed();
            let y_changed = ui
                .add(egui::DragValue::new(&mut new_y).speed(0.5).prefix("y: "))
                .changed();
            if x_changed || y_changed {
                set_transmit_nested_param(
                    session,
                    path,
                    behavior_index,
                    nested_index,
                    "x",
                    toml::Value::Float(new_x as f64),
                );
                set_transmit_nested_param(
                    session,
                    path,
                    behavior_index,
                    nested_index,
                    "y",
                    toml::Value::Float(new_y as f64),
                );
                *modified = true;
            }
        }
        Some(MobBehaviorVariant::DoForTime) => {
            let seconds = nested_table
                .get("seconds")
                .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)))
                .unwrap_or(1.0) as f32;

            let mut value = seconds;
            if ui
                .add(
                    egui::DragValue::new(&mut value)
                        .speed(0.1)
                        .range(0.0..=100.0)
                        .prefix("s: "),
                )
                .changed()
            {
                set_transmit_nested_param(
                    session,
                    path,
                    behavior_index,
                    nested_index,
                    "seconds",
                    toml::Value::Float(value as f64),
                );
                *modified = true;
            }
        }
        _ => {
            // No parameters for simple actions
        }
    }
}
