use std::path::PathBuf;

use bevy::{ecs::message::MessageWriter, prelude::Resource};
use bevy_egui::egui;

use crate::{
    data::EditorSession,
    file::{DeleteMobEvent, FileNode, FileTreeState, LoadMobEvent},
};

/// State for delete confirmation dialog
#[derive(Resource, Default)]
pub struct DeleteDialogState {
    pub is_open: bool,
    pub file_path: PathBuf,
    pub file_name: String,
}

impl DeleteDialogState {
    pub fn open(&mut self, path: PathBuf, name: String) {
        self.is_open = true;
        self.file_path = path;
        self.file_name = name;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }
}

/// Action to perform after confirming unsaved changes
#[derive(Clone)]
pub enum UnsavedAction {
    LoadFile(PathBuf),
    Exit,
}

/// Dialog for unsaved changes confirmation
#[derive(Resource, Default)]
pub struct UnsavedChangesDialog {
    pub is_open: bool,
    pub pending_action: Option<UnsavedAction>,
}

impl UnsavedChangesDialog {
    pub fn open(&mut self, action: UnsavedAction) {
        self.is_open = true;
        self.pending_action = Some(action);
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.pending_action = None;
    }
}

/// Dialog for displaying critical errors
#[derive(Resource, Default)]
pub struct ErrorDialog {
    pub is_open: bool,
    pub title: String,
    pub message: String,
    pub details: Option<String>,
}

/// Dialog for displaying validation errors
#[derive(Resource, Default)]
pub struct ValidationDialog {
    pub is_open: bool,
    pub errors: Vec<crate::data::ValidationError>,
}

impl ErrorDialog {
    pub fn close(&mut self) {
        self.is_open = false;
    }
}

/// Dialog for creating a new folder
#[derive(Resource, Default)]
pub struct NewFolderDialog {
    pub is_open: bool,
    pub parent_path: PathBuf,
    pub folder_name: String,
}

impl NewFolderDialog {
    pub fn open(&mut self, parent_path: PathBuf) {
        self.is_open = true;
        self.parent_path = parent_path;
        self.folder_name = String::new();
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.folder_name.clear();
    }
}

/// Dialog for creating a new mob or patch file
#[derive(Resource, Default)]
pub struct NewMobDialog {
    pub is_open: bool,
    pub parent_path: PathBuf,
    pub file_name: String,
    pub is_patch: bool,
}

impl NewMobDialog {
    pub fn open_mob(&mut self, parent_path: PathBuf) {
        self.is_open = true;
        self.parent_path = parent_path;
        self.file_name = String::new();
        self.is_patch = false;
    }

    pub fn open_patch(&mut self, parent_path: PathBuf) {
        self.is_open = true;
        self.parent_path = parent_path;
        self.file_name = String::new();
        self.is_patch = true;
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.file_name.clear();
    }
}

/// Render the file browser panel
pub fn file_panel_ui(
    ui: &mut egui::Ui,
    file_tree: &mut FileTreeState,
    session: &EditorSession,
    load_events: &mut MessageWriter<LoadMobEvent>,
    delete_dialog: &mut DeleteDialogState,
    unsaved_dialog: &mut UnsavedChangesDialog,
    new_folder_dialog: &mut NewFolderDialog,
    new_mob_dialog: &mut NewMobDialog,
) {
    ui.heading("Files");
    ui.separator();

    // Refresh button only
    if ui.button("🔄 Refresh").clicked() {
        file_tree.needs_refresh = true;
    }

    ui.separator();

    // File tree
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let roots = file_tree.roots.clone();
            for root in roots {
                render_file_node(
                    ui,
                    &root,
                    file_tree,
                    session,
                    load_events,
                    delete_dialog,
                    unsaved_dialog,
                    new_folder_dialog,
                    new_mob_dialog,
                );
            }

            if file_tree.roots.is_empty() {
                ui.colored_label(
                    egui::Color32::GRAY,
                    "No mob files found.\n\nRight-click on a folder to create\nnew mobs, patches, or folders.",
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
    delete_dialog: &mut DeleteDialogState,
    unsaved_dialog: &mut UnsavedChangesDialog,
    new_folder_dialog: &mut NewFolderDialog,
    new_mob_dialog: &mut NewMobDialog,
) {
    let is_selected = file_tree.selected.as_ref() == Some(&node.path);
    let is_current = session.current_path.as_ref() == Some(&node.path);

    if node.is_directory {
        // Directory node with collapse/expand
        let icon = if node.expanded { "📂" } else { "📁" };
        let header = egui::CollapsingHeader::new(format!("{} {}", icon, node.name))
            .id_salt(&node.path)
            .default_open(node.expanded)
            .show(ui, |ui| {
                for child in &node.children {
                    render_file_node(
                        ui,
                        child,
                        file_tree,
                        session,
                        load_events,
                        delete_dialog,
                        unsaved_dialog,
                        new_folder_dialog,
                        new_mob_dialog,
                    );
                }
            });

        // Track expansion state
        if header.header_response.clicked() {
            file_tree.toggle_expanded(&node.path);
        }

        // Context menu for directories
        header.header_response.context_menu(|ui| {
            if ui.button("📄 New Mob...").clicked() {
                new_mob_dialog.open_mob(node.path.clone());
                ui.close();
            }

            if ui.button("📋 New Patch...").clicked() {
                new_mob_dialog.open_patch(node.path.clone());
                ui.close();
            }

            ui.separator();

            if ui.button("📁 New Folder...").clicked() {
                new_folder_dialog.open(node.path.clone());
                ui.close();
            }
        });
    } else {
        // File node
        let icon = if node.name.ends_with(".mob") {
            "📄"
        } else {
            "📋" // .mobpatch
        };

        // Highlight currently edited file differently
        let label = format!("{} {}", icon, node.name);
        let mut text = egui::RichText::new(label);

        if is_current {
            text = text.strong();
            if session.is_modified {
                text = text.color(egui::Color32::YELLOW);
            }
        }

        let response = ui.selectable_label(is_selected, text);

        if response.clicked() {
            file_tree.selected = Some(node.path.clone());

            // Check if we're loading a different file with unsaved changes
            let is_different_file = session.current_path.as_ref() != Some(&node.path);
            if session.is_modified && is_different_file {
                // Show unsaved changes dialog
                unsaved_dialog.open(UnsavedAction::LoadFile(node.path.clone()));
            } else {
                load_events.write(LoadMobEvent {
                    path: node.path.clone(),
                });
            }
        }

        // Context menu for files
        response.context_menu(|ui| {
            if ui.button("📂 Open").clicked() {
                let is_different_file = session.current_path.as_ref() != Some(&node.path);
                if session.is_modified && is_different_file {
                    unsaved_dialog.open(UnsavedAction::LoadFile(node.path.clone()));
                } else {
                    load_events.write(LoadMobEvent {
                        path: node.path.clone(),
                    });
                }
                ui.close();
            }

            ui.separator();

            // Determine if this is a mob or patch
            let is_mob = node.name.ends_with(".mob");

            if is_mob
                && ui.button("📋 Create Patch...").clicked() {
                    // Create patch in same directory as the mob file
                    if let Some(parent) = node.path.parent() {
                        new_mob_dialog.open_patch(parent.to_path_buf());
                    }
                    ui.close();
                }

            ui.separator();

            // Delete button (with warning color)
            if ui
                .add(egui::Button::new("🗑 Delete...").fill(egui::Color32::from_rgb(80, 30, 30)))
                .clicked()
            {
                delete_dialog.open(node.path.clone(), node.name.clone());
                ui.close();
            }
        });
    }
}

/// Render the delete confirmation dialog window
pub fn render_delete_dialog(
    ctx: &egui::Context,
    state: &mut DeleteDialogState,
    delete_events: &mut MessageWriter<DeleteMobEvent>,
) {
    if !state.is_open {
        return;
    }

    egui::Window::new("Confirm Delete")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(format!(
                "Are you sure you want to delete '{}'?",
                state.file_name
            ));

            ui.label(
                egui::RichText::new("The file will be moved to your system trash.")
                    .small()
                    .color(egui::Color32::GRAY),
            );

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    state.close();
                }

                if ui
                    .add(egui::Button::new("Delete").fill(egui::Color32::from_rgb(150, 50, 50)))
                    .clicked()
                {
                    delete_events.write(DeleteMobEvent {
                        path: state.file_path.clone(),
                    });
                    state.close();
                }
            });
        });
}
