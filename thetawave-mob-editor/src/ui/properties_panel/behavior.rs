//! Behavior tree editing functionality for the properties panel.
//!
//! This module handles the complete behavior tree editing UI, including
//! rendering and manipulating all node types (Forever, Sequence, Fallback,
//! While, IfThen, Wait, Action, Trigger).

use bevy_egui::egui;
use strum::IntoEnumIterator;
use thetawave_mobs::{BehaviorNodeType, MobBehaviorCategory, MobBehaviorVariant, BY_CATEGORY};

use crate::data::EditorSession;

use super::fields::{render_patch_indicator, INDENT_SPACING, INHERITED_COLOR, PATCHED_COLOR};

/// Render the behavior section of the properties panel.
pub fn render_behavior_section(
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
            ui.horizontal(|ui| {
                render_patch_indicator(ui, is_patched, is_patch);
                if is_patch && !is_patched {
                    ui.label(
                        egui::RichText::new("(inherited from base)")
                            .small()
                            .color(INHERITED_COLOR),
                    );
                    if ui.button("Override").clicked() {
                        if let Some(behavior) = display_table.get("behavior").cloned() {
                            if let Some(mob) =
                                session.current_mob.as_mut().and_then(|v| v.as_table_mut())
                            {
                                mob.insert("behavior".to_string(), behavior);
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
                    if ui.button("Reset to base").clicked() {
                        if let Some(mob) =
                            session.current_mob.as_mut().and_then(|v| v.as_table_mut())
                        {
                            mob.remove("behavior");
                            *modified = true;
                        }
                    }
                }
            });

            let behavior = display_table.get("behavior");
            let can_edit = !is_patch || is_patched;

            if let Some(behavior) = behavior {
                render_behavior_node(ui, session, behavior, &[], can_edit, 0, modified);
            } else {
                ui.label(
                    egui::RichText::new("No behavior tree defined")
                        .italics()
                        .color(INHERITED_COLOR),
                );
                if can_edit && ui.button("Create Default Behavior").clicked() {
                    add_default_behavior_tree(session);
                    *modified = true;
                }
            }
        });
}

/// Render a single behavior node recursively.
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
        ui.label(egui::RichText::new("Invalid node").color(egui::Color32::RED));
        return;
    };

    let node_type_str = table
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown");
    let node_type: Option<BehaviorNodeType> = node_type_str.parse().ok();

    // Create collapsible header for the node
    let header_text = format_node_header(table, node_type);

    egui::CollapsingHeader::new(&header_text)
        .id_salt(format!("behavior_{:?}", path))
        .default_open(depth < 2)
        .show(ui, |ui| {
            // Node controls row
            ui.horizontal(|ui| {
                if can_edit {
                    // Type selector
                    ui.label("Type:");
                    let mut current_type = node_type;
                    let display_text = match &current_type {
                        Some(t) => t.as_ref(),
                        None => "Unknown",
                    };
                    egui::ComboBox::from_id_salt(format!("node_type_{:?}", path))
                        .selected_text(display_text)
                        .width(100.0)
                        .show_ui(ui, |ui| {
                            for t in BehaviorNodeType::iter() {
                                let is_selected = current_type == Some(t);
                                if ui.selectable_label(is_selected, t.as_ref()).clicked() {
                                    current_type = Some(t);
                                }
                            }
                        });
                    if current_type != node_type {
                        if let Some(new_type) = current_type {
                            change_behavior_node_type(session, path, new_type);
                            *modified = true;
                        }
                    }

                    // Move/delete buttons (only for non-root nodes)
                    if !path.is_empty() {
                        ui.separator();
                        let index = path[path.len() - 1];
                        let parent_path = &path[..path.len() - 1];
                        let sibling_count = get_children_count(session, parent_path);

                        if index > 0 && ui.small_button("⏶").on_hover_text("Move up").clicked() {
                            move_behavior_node(session, path, -1);
                            *modified = true;
                        }
                        if index + 1 < sibling_count
                            && ui.small_button("⏷").on_hover_text("Move down").clicked()
                        {
                            move_behavior_node(session, path, 1);
                            *modified = true;
                        }
                        if ui.small_button("🗑").on_hover_text("Delete node").clicked() {
                            delete_behavior_node(session, path);
                            *modified = true;
                        }
                    }
                } else {
                    let type_display = match &node_type {
                        Some(t) => t.as_ref(),
                        None => node_type_str,
                    };
                    ui.label(format!("Type: {}", type_display));
                }
            });

            ui.add_space(4.0);

            // Render type-specific content
            match node_type {
                Some(BehaviorNodeType::Forever | BehaviorNodeType::Sequence | BehaviorNodeType::Fallback) => {
                    render_control_node(ui, session, table, path, can_edit, depth, modified);
                }
                Some(BehaviorNodeType::While) => {
                    render_while_node(ui, session, table, path, can_edit, depth, modified);
                }
                Some(BehaviorNodeType::IfThen) => {
                    render_if_then_node(ui, session, table, path, can_edit, depth, modified);
                }
                Some(BehaviorNodeType::Wait) => {
                    render_wait_node(ui, session, table, path, can_edit, modified);
                }
                Some(BehaviorNodeType::Action) => {
                    render_action_node(ui, session, table, path, can_edit, modified);
                }
                Some(BehaviorNodeType::Trigger) => {
                    render_trigger_node(ui, session, table, path, can_edit, modified);
                }
                None => {
                    ui.label(
                        egui::RichText::new(format!("Unknown node type: {}", node_type_str))
                            .color(egui::Color32::YELLOW),
                    );
                }
            }
        });
}

/// Format a header string for a behavior node.
fn format_node_header(table: &toml::value::Table, node_type: Option<BehaviorNodeType>) -> String {
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

/// Render a control node (Forever, Sequence, Fallback) with children.
fn render_control_node(
    ui: &mut egui::Ui,
    session: &mut EditorSession,
    table: &toml::value::Table,
    path: &[usize],
    can_edit: bool,
    depth: usize,
    modified: &mut bool,
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
        render_behavior_node(ui, session, child, &child_path, can_edit, depth + 1, modified);
    }

    if can_edit {
        if ui.button("+ Add Child").clicked() {
            add_behavior_child(session, path);
            *modified = true;
        }
    }
}

/// Render a While node (has condition and child).
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
        render_behavior_node(ui, session, child, &child_path, can_edit, depth + 1, modified);
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

/// Render an IfThen node (has condition, then_child, and optional else_child).
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
        render_behavior_node(ui, session, then_child, &then_path, can_edit, depth + 1, modified);
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
        render_behavior_node(ui, session, else_child, &else_path, can_edit, depth + 1, modified);

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

/// Render a Wait node.
fn render_wait_node(
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

/// Render an Action node with its behaviors list.
fn render_action_node(
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
                    if current_action != action_type {
                        if let Some(new_action) = current_action {
                            change_action_behavior_type(session, path, i, new_action);
                            *modified = true;
                        }
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

/// Render parameters for specific action types.
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

/// Render parameters in read-only mode.
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

/// Render a Trigger node.
fn render_trigger_node(
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

/// Render the nested behaviors list for a TransmitMobBehavior action.
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
                    if current_nested != nested_action {
                        if let Some(new_action) = current_nested {
                            change_transmit_nested_behavior_type(
                                session,
                                path,
                                behavior_index,
                                j,
                                new_action,
                            );
                            *modified = true;
                        }
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

/// Render parameters for a nested behavior in TransmitMobBehavior.
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

// =============================================================================
// Behavior Tree TOML Manipulation Helpers
// =============================================================================

/// Add a default behavior tree (Forever with one Action child).
fn add_default_behavior_tree(session: &mut EditorSession) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        let mut action = toml::value::Table::new();
        action.insert(
            "type".to_string(),
            toml::Value::String("Action".to_string()),
        );
        action.insert(
            "name".to_string(),
            toml::Value::String("Movement".to_string()),
        );
        action.insert(
            "behaviors".to_string(),
            toml::Value::Array(vec![{
                let mut behavior = toml::value::Table::new();
                behavior.insert(
                    "action".to_string(),
                    toml::Value::String("MoveDown".to_string()),
                );
                toml::Value::Table(behavior)
            }]),
        );

        let mut root = toml::value::Table::new();
        root.insert(
            "type".to_string(),
            toml::Value::String("Forever".to_string()),
        );
        root.insert(
            "children".to_string(),
            toml::Value::Array(vec![toml::Value::Table(action)]),
        );

        mob.insert("behavior".to_string(), toml::Value::Table(root));
    }
}

/// Get a mutable reference to a behavior node at the given path.
fn get_behavior_node_mut<'a>(
    session: &'a mut EditorSession,
    path: &[usize],
) -> Option<&'a mut toml::Value> {
    let mob = session.current_mob.as_mut()?.as_table_mut()?;
    let mut current = mob.get_mut("behavior")?;

    for &index in path {
        let table = current.as_table_mut()?;
        let node_type = table.get("type").and_then(|v| v.as_str())?;

        current = match node_type {
            "Forever" | "Sequence" | "Fallback" => table
                .get_mut("children")?
                .as_array_mut()?
                .get_mut(index)?,
            "While" => {
                if index == 0 {
                    table.get_mut("condition")?
                } else {
                    table.get_mut("child")?
                }
            }
            "IfThen" => match index {
                0 => table.get_mut("condition")?,
                1 => table.get_mut("then_child")?,
                2 => table.get_mut("else_child")?,
                _ => return None,
            },
            _ => return None, // Leaf nodes have no children
        };
    }

    Some(current)
}

/// Set a field on a behavior node at the given path.
fn set_behavior_node_field(
    session: &mut EditorSession,
    path: &[usize],
    field: &str,
    value: toml::Value,
) {
    if let Some(node) = get_behavior_node_mut(session, path) {
        if let Some(table) = node.as_table_mut() {
            table.insert(field.to_string(), value);
        }
    }
}

/// Get the number of children for a control node at the given path.
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
                match table
                    .get("children")
                    .and_then(|v| v.as_array())
                    .and_then(|arr| arr.get(index))
                {
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
            "IfThen" => match index {
                0 => table.get("condition"),
                1 => table.get("then_child"),
                2 => table.get("else_child"),
                _ => None,
            }
            .unwrap_or(&toml::Value::Boolean(false)), // placeholder
            _ => return 0,
        };
    }

    // Now count children of the node at path
    let Some(table) = current.as_table() else {
        return 0;
    };
    let node_type = table.get("type").and_then(|v| v.as_str()).unwrap_or("");

    match node_type {
        "Forever" | "Sequence" | "Fallback" => table
            .get("children")
            .and_then(|v| v.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0),
        _ => 0,
    }
}

/// Add a child to a control node (Forever, Sequence, Fallback).
fn add_behavior_child(session: &mut EditorSession, path: &[usize]) {
    if let Some(node) = get_behavior_node_mut(session, path) {
        if let Some(table) = node.as_table_mut() {
            // Create a default Action child
            let mut action = toml::value::Table::new();
            action.insert(
                "type".to_string(),
                toml::Value::String("Action".to_string()),
            );
            action.insert(
                "name".to_string(),
                toml::Value::String("New Action".to_string()),
            );
            action.insert("behaviors".to_string(), toml::Value::Array(vec![]));

            if let Some(children) = table.get_mut("children").and_then(|v| v.as_array_mut()) {
                children.push(toml::Value::Table(action));
            } else {
                table.insert(
                    "children".to_string(),
                    toml::Value::Array(vec![toml::Value::Table(action)]),
                );
            }
        }
    }
}

/// Delete a behavior node at the given path.
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
                    if let Some(children) = table.get_mut("children").and_then(|v| v.as_array_mut())
                    {
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
                    if index == 2 {
                        table.remove("else_child");
                    }
                    // Don't allow deleting condition or then_child
                }
                _ => {}
            }
        }
    }
}

/// Move a behavior node up or down within its parent.
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

/// Change the type of a behavior node.
fn change_behavior_node_type(session: &mut EditorSession, path: &[usize], new_type: BehaviorNodeType) {
    if let Some(node) = get_behavior_node_mut(session, path) {
        if let Some(table) = node.as_table_mut() {
            let old_type_str = table
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let old_type: Option<BehaviorNodeType> = old_type_str.parse().ok();

            // Only proceed if type is actually changing
            if old_type == Some(new_type) {
                return;
            }

            // Update type
            table.insert("type".to_string(), toml::Value::String(new_type.as_ref().to_string()));

            // Handle structure changes based on old/new type categories
            let old_is_control = old_type.map(|t: BehaviorNodeType| t.is_control_node()).unwrap_or(false);
            let new_is_control = new_type.is_control_node();

            if old_is_control && new_is_control {
                // Keep children array as-is
            } else if old_is_control && !new_is_control {
                // Switching from control to leaf - remove children
                table.remove("children");
                add_fields_for_node_type(table, new_type);
            } else if !old_is_control && new_is_control {
                // Switching from leaf to control - add empty children array
                remove_leaf_fields(table);
                table.insert("children".to_string(), toml::Value::Array(vec![]));
            } else {
                // Switching between different leaf/special types
                remove_leaf_fields(table);
                add_fields_for_node_type(table, new_type);
            }
        }
    }
}

/// Remove leaf-specific fields from a node.
fn remove_leaf_fields(table: &mut toml::value::Table) {
    table.remove("seconds");
    table.remove("name");
    table.remove("behaviors");
    table.remove("trigger_type");
    table.remove("child");
    table.remove("condition");
    table.remove("then_child");
    table.remove("else_child");
    table.remove("children");
}

/// Add required fields for a new node type.
fn add_fields_for_node_type(table: &mut toml::value::Table, node_type: BehaviorNodeType) {
    match node_type {
        BehaviorNodeType::Wait => {
            table.insert("seconds".to_string(), toml::Value::Float(1.0));
        }
        BehaviorNodeType::Action => {
            table.insert(
                "name".to_string(),
                toml::Value::String("New Action".to_string()),
            );
            table.insert("behaviors".to_string(), toml::Value::Array(vec![]));
        }
        BehaviorNodeType::Trigger => {
            table.insert("trigger_type".to_string(), toml::Value::String(String::new()));
        }
        BehaviorNodeType::While => {
            let mut child = toml::value::Table::new();
            child.insert(
                "type".to_string(),
                toml::Value::String("Action".to_string()),
            );
            child.insert("name".to_string(), toml::Value::String("Child".to_string()));
            child.insert("behaviors".to_string(), toml::Value::Array(vec![]));
            table.insert("child".to_string(), toml::Value::Table(child));
        }
        BehaviorNodeType::IfThen => {
            let mut cond = toml::value::Table::new();
            cond.insert("type".to_string(), toml::Value::String("Wait".to_string()));
            cond.insert("seconds".to_string(), toml::Value::Float(1.0));
            table.insert("condition".to_string(), toml::Value::Table(cond));

            let mut then = toml::value::Table::new();
            then.insert(
                "type".to_string(),
                toml::Value::String("Action".to_string()),
            );
            then.insert("name".to_string(), toml::Value::String("Then".to_string()));
            then.insert("behaviors".to_string(), toml::Value::Array(vec![]));
            table.insert("then_child".to_string(), toml::Value::Table(then));
        }
        BehaviorNodeType::Forever | BehaviorNodeType::Sequence | BehaviorNodeType::Fallback => {
            table.insert("children".to_string(), toml::Value::Array(vec![]));
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
            child.insert(
                "type".to_string(),
                toml::Value::String("Action".to_string()),
            );
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
            child.insert(
                "type".to_string(),
                toml::Value::String("Action".to_string()),
            );
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
            child.insert(
                "type".to_string(),
                toml::Value::String("Action".to_string()),
            );
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
            behavior.insert(
                "action".to_string(),
                toml::Value::String("MoveDown".to_string()),
            );

            if let Some(behaviors) = table.get_mut("behaviors").and_then(|v| v.as_array_mut()) {
                behaviors.push(toml::Value::Table(behavior));
            } else {
                table.insert(
                    "behaviors".to_string(),
                    toml::Value::Array(vec![toml::Value::Table(behavior)]),
                );
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

fn change_action_behavior_type(
    session: &mut EditorSession,
    path: &[usize],
    index: usize,
    new_action: MobBehaviorVariant,
) {
    if let Some(node) = get_behavior_node_mut(session, path) {
        if let Some(table) = node.as_table_mut() {
            if let Some(behaviors) = table.get_mut("behaviors").and_then(|v| v.as_array_mut()) {
                if let Some(behavior) = behaviors.get_mut(index).and_then(|v| v.as_table_mut()) {
                    // Clear old fields except action
                    let keys_to_remove: Vec<_> = behavior
                        .keys()
                        .filter(|k| *k != "action")
                        .cloned()
                        .collect();
                    for key in keys_to_remove {
                        behavior.remove(&key);
                    }

                    // Set new action type
                    behavior.insert(
                        "action".to_string(),
                        toml::Value::String(new_action.as_ref().to_string()),
                    );

                    // Add default parameters for actions that need them
                    match new_action {
                        MobBehaviorVariant::MoveTo => {
                            behavior.insert("x".to_string(), toml::Value::Float(0.0));
                            behavior.insert("y".to_string(), toml::Value::Float(0.0));
                        }
                        MobBehaviorVariant::DoForTime => {
                            behavior.insert("seconds".to_string(), toml::Value::Float(1.0));
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn set_action_behavior_param(
    session: &mut EditorSession,
    path: &[usize],
    index: usize,
    param: &str,
    value: toml::Value,
) {
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

fn remove_action_behavior_param(
    session: &mut EditorSession,
    path: &[usize],
    index: usize,
    param: &str,
) {
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

// =============================================================================
// TransmitMobBehavior Nested Behaviors Helpers
// =============================================================================

fn add_transmit_nested_behavior(
    session: &mut EditorSession,
    path: &[usize],
    behavior_index: usize,
) {
    if let Some(node) = get_behavior_node_mut(session, path) {
        if let Some(table) = node.as_table_mut() {
            if let Some(behaviors) = table.get_mut("behaviors").and_then(|v| v.as_array_mut()) {
                if let Some(behavior) = behaviors
                    .get_mut(behavior_index)
                    .and_then(|v| v.as_table_mut())
                {
                    let mut new_nested = toml::value::Table::new();
                    new_nested.insert(
                        "action".to_string(),
                        toml::Value::String("MoveDown".to_string()),
                    );

                    if let Some(nested_arr) =
                        behavior.get_mut("behaviors").and_then(|v| v.as_array_mut())
                    {
                        nested_arr.push(toml::Value::Table(new_nested));
                    } else {
                        behavior.insert(
                            "behaviors".to_string(),
                            toml::Value::Array(vec![toml::Value::Table(new_nested)]),
                        );
                    }
                }
            }
        }
    }
}

fn delete_transmit_nested_behavior(
    session: &mut EditorSession,
    path: &[usize],
    behavior_index: usize,
    nested_index: usize,
) {
    if let Some(node) = get_behavior_node_mut(session, path) {
        if let Some(table) = node.as_table_mut() {
            if let Some(behaviors) = table.get_mut("behaviors").and_then(|v| v.as_array_mut()) {
                if let Some(behavior) = behaviors
                    .get_mut(behavior_index)
                    .and_then(|v| v.as_table_mut())
                {
                    if let Some(nested_arr) =
                        behavior.get_mut("behaviors").and_then(|v| v.as_array_mut())
                    {
                        if nested_index < nested_arr.len() {
                            nested_arr.remove(nested_index);
                        }
                    }
                }
            }
        }
    }
}

fn move_transmit_nested_behavior(
    session: &mut EditorSession,
    path: &[usize],
    behavior_index: usize,
    nested_index: usize,
    direction: i32,
) {
    if let Some(node) = get_behavior_node_mut(session, path) {
        if let Some(table) = node.as_table_mut() {
            if let Some(behaviors) = table.get_mut("behaviors").and_then(|v| v.as_array_mut()) {
                if let Some(behavior) = behaviors
                    .get_mut(behavior_index)
                    .and_then(|v| v.as_table_mut())
                {
                    if let Some(nested_arr) =
                        behavior.get_mut("behaviors").and_then(|v| v.as_array_mut())
                    {
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

fn change_transmit_nested_behavior_type(
    session: &mut EditorSession,
    path: &[usize],
    behavior_index: usize,
    nested_index: usize,
    new_action: MobBehaviorVariant,
) {
    if let Some(node) = get_behavior_node_mut(session, path) {
        if let Some(table) = node.as_table_mut() {
            if let Some(behaviors) = table.get_mut("behaviors").and_then(|v| v.as_array_mut()) {
                if let Some(behavior) = behaviors
                    .get_mut(behavior_index)
                    .and_then(|v| v.as_table_mut())
                {
                    if let Some(nested_arr) =
                        behavior.get_mut("behaviors").and_then(|v| v.as_array_mut())
                    {
                        if let Some(nested) =
                            nested_arr.get_mut(nested_index).and_then(|v| v.as_table_mut())
                        {
                            // Clear old fields except action
                            let keys_to_remove: Vec<_> = nested
                                .keys()
                                .filter(|k| *k != "action")
                                .cloned()
                                .collect();
                            for key in keys_to_remove {
                                nested.remove(&key);
                            }

                            // Set new action type
                            nested.insert(
                                "action".to_string(),
                                toml::Value::String(new_action.as_ref().to_string()),
                            );

                            // Add default parameters
                            match new_action {
                                MobBehaviorVariant::MoveTo => {
                                    nested.insert("x".to_string(), toml::Value::Float(0.0));
                                    nested.insert("y".to_string(), toml::Value::Float(0.0));
                                }
                                MobBehaviorVariant::DoForTime => {
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

fn set_transmit_nested_param(
    session: &mut EditorSession,
    path: &[usize],
    behavior_index: usize,
    nested_index: usize,
    param: &str,
    value: toml::Value,
) {
    if let Some(node) = get_behavior_node_mut(session, path) {
        if let Some(table) = node.as_table_mut() {
            if let Some(behaviors) = table.get_mut("behaviors").and_then(|v| v.as_array_mut()) {
                if let Some(behavior) = behaviors
                    .get_mut(behavior_index)
                    .and_then(|v| v.as_table_mut())
                {
                    if let Some(nested_arr) =
                        behavior.get_mut("behaviors").and_then(|v| v.as_array_mut())
                    {
                        if let Some(nested) =
                            nested_arr.get_mut(nested_index).and_then(|v| v.as_table_mut())
                        {
                            nested.insert(param.to_string(), value);
                        }
                    }
                }
            }
        }
    }
}
