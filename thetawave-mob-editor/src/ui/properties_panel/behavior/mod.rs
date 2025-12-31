//! Behavior tree editing functionality for the properties panel.
//!
//! This module handles the complete behavior tree editing UI, including
//! rendering and manipulating all node types (Forever, Sequence, Fallback,
//! While, IfThen, Wait, Action, Trigger).

mod action_ops;
mod navigation;
mod node_rendering;
mod transmit_ops;
mod tree_ops;

use bevy_egui::egui;
use strum::IntoEnumIterator;
use thetawave_mobs::BehaviorNodeType;

use crate::data::EditorSession;

use super::fields::{render_patch_indicator, INHERITED_COLOR, PATCHED_COLOR};
use navigation::get_children_count;
use node_rendering::{
    format_node_header, render_action_node, render_control_node, render_if_then_node,
    render_trigger_node, render_wait_node, render_while_node,
};
use tree_ops::{
    add_default_behavior_tree, change_behavior_node_type, delete_behavior_node, move_behavior_node,
};

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
                    if ui.button("Override").clicked()
                        && let Some(behavior) = display_table.get("behavior").cloned()
                        && let Some(mob) =
                            session.current_mob.as_mut().and_then(|v| v.as_table_mut())
                    {
                        mob.insert("behavior".to_string(), behavior);
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
                        mob.remove("behavior");
                        *modified = true;
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
                    if current_type != node_type
                        && let Some(new_type) = current_type
                    {
                        change_behavior_node_type(session, path, new_type);
                        *modified = true;
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
                Some(
                    BehaviorNodeType::Forever
                    | BehaviorNodeType::Sequence
                    | BehaviorNodeType::Fallback,
                ) => {
                    render_control_node(
                        ui,
                        session,
                        table,
                        path,
                        can_edit,
                        depth,
                        modified,
                        render_behavior_node,
                    );
                }
                Some(BehaviorNodeType::While) => {
                    render_while_node(
                        ui,
                        session,
                        table,
                        path,
                        can_edit,
                        depth,
                        modified,
                        render_behavior_node,
                    );
                }
                Some(BehaviorNodeType::IfThen) => {
                    render_if_then_node(
                        ui,
                        session,
                        table,
                        path,
                        can_edit,
                        depth,
                        modified,
                        render_behavior_node,
                    );
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
