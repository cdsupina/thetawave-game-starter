use bevy::{ecs::message::MessageWriter, prelude::*};
use bevy_egui::egui;

use crate::{
    data::EditorSession,
    file::{FileNode, FileTreeState, LoadMobEvent, NewMobEvent},
};

/// Render the file browser panel
pub fn file_panel_ui(
    ui: &mut egui::Ui,
    file_tree: &mut FileTreeState,
    session: &mut EditorSession,
    load_events: &mut MessageWriter<LoadMobEvent>,
    _new_events: &mut MessageWriter<NewMobEvent>,
) {
    ui.heading("Files");
    ui.separator();

    // Refresh button
    if ui.button("Refresh").clicked() {
        file_tree.needs_refresh = true;
    }

    ui.separator();

    // File tree
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let roots = file_tree.roots.clone();
            for root in roots {
                render_file_node(ui, &root, file_tree, session, load_events);
            }

            if file_tree.roots.is_empty() {
                ui.colored_label(
                    egui::Color32::GRAY,
                    "No mob files found.\nCheck that the assets/mobs directory exists.",
                );
            }
        });
}

fn render_file_node(
    ui: &mut egui::Ui,
    node: &FileNode,
    file_tree: &mut FileTreeState,
    session: &EditorSession,
    load_events: &mut MessageWriter<LoadMobEvent>,
) {
    let is_selected = file_tree.selected.as_ref() == Some(&node.path);

    if node.is_directory {
        // Directory node with collapse/expand
        let icon = if node.expanded { "📂" } else { "📁" };
        let header = egui::CollapsingHeader::new(format!("{} {}", icon, node.name))
            .id_salt(&node.path)
            .default_open(node.expanded)
            .show(ui, |ui| {
                for child in &node.children {
                    render_file_node(ui, child, file_tree, session, load_events);
                }
            });

        // Track expansion state
        if header.header_response.clicked() {
            file_tree.toggle_expanded(&node.path);
        }
    } else {
        // File node
        let icon = if node.name.ends_with(".mob") {
            "📄"
        } else {
            "📋" // .mobpatch
        };

        let label = format!("{} {}", icon, node.name);

        let response = ui.selectable_label(is_selected, label);

        if response.clicked() {
            file_tree.selected = Some(node.path.clone());
            load_events.write(LoadMobEvent {
                path: node.path.clone(),
            });
        }

        // Context menu
        response.context_menu(|ui| {
            if ui.button("Open").clicked() {
                load_events.write(LoadMobEvent {
                    path: node.path.clone(),
                });
                ui.close();
            }

            ui.separator();

            if ui.button("Delete...").clicked() {
                // TODO: Show delete confirmation dialog
                ui.close();
            }
        });
    }
}
