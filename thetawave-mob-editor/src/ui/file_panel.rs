//! File browser panel UI.
//!
//! Renders the left panel with the file tree, context menus,
//! and dialog states for file operations.

use std::path::PathBuf;

use bevy::{ecs::message::MessageWriter, prelude::Resource};
use bevy_egui::egui;

use crate::{
    data::{EditorSession, MobAssetRegistry, SpriteRegistry},
    file::{DeleteDirectoryEvent, DeleteMobEvent, FileNode, FileTreeState, LoadMobEvent},
    plugin::EditorConfig,
};

/// Maximum length for displayed filenames in the file tree
use super::truncate_filename;

const MAX_FILENAME_DISPLAY_LEN: usize = 28;

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

/// State for delete directory confirmation dialog
#[derive(Resource, Default)]
pub struct DeleteDirectoryDialogState {
    pub is_open: bool,
    pub dir_path: PathBuf,
    pub dir_name: String,
    pub contained_files: Vec<String>,
}

impl DeleteDirectoryDialogState {
    pub fn open(&mut self, path: PathBuf, name: String) {
        self.is_open = true;
        self.dir_path = path.clone();
        self.dir_name = name;
        // Collect all files in the directory
        self.contained_files = Self::collect_files(&path);
    }

    fn collect_files(path: &PathBuf) -> Vec<String> {
        let mut files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                if entry_path.is_dir() {
                    // Recursively collect from subdirectories
                    for sub_file in Self::collect_files(&entry_path) {
                        files.push(format!("{}/{}", name, sub_file));
                    }
                } else {
                    files.push(name);
                }
            }
        }
        files.sort();
        files
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.contained_files.clear();
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

/// Dialog for creating a new folder
#[derive(Resource, Default)]
pub struct NewFolderDialog {
    pub is_open: bool,
    pub parent_path: PathBuf,
    pub folder_name: String,
    /// Error message to display in the dialog
    pub error_message: Option<String>,
}

impl NewFolderDialog {
    pub fn open(&mut self, parent_path: PathBuf) {
        self.is_open = true;
        self.parent_path = parent_path;
        self.folder_name = String::new();
        self.error_message = None;
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.folder_name.clear();
        self.error_message = None;
    }
}

/// Dialog for creating a new mob or patch file
#[derive(Resource, Default)]
pub struct NewMobDialog {
    pub is_open: bool,
    pub parent_path: PathBuf,
    pub file_name: String,
    pub is_patch: bool,
    /// Selected base mob reference for patches (e.g., "xhitara/spitter")
    pub selected_mob_ref: Option<String>,
    /// Error message to display in the dialog
    pub error_message: Option<String>,
}

impl NewMobDialog {
    pub fn open_mob(&mut self, parent_path: PathBuf) {
        self.is_open = true;
        self.parent_path = parent_path;
        self.file_name = String::new();
        self.is_patch = false;
        self.selected_mob_ref = None;
        self.error_message = None;
    }

    pub fn open_patch(&mut self) {
        self.is_open = true;
        self.parent_path = PathBuf::new(); // Not used for patches
        self.file_name = String::new();
        self.is_patch = true;
        self.selected_mob_ref = None;
        self.error_message = None;
    }

    /// Open patch dialog with a pre-selected base mob
    pub fn open_patch_for_mob(&mut self, mob_ref: String) {
        self.is_open = true;
        self.parent_path = PathBuf::new(); // Not used for patches
        self.file_name = String::new();
        self.is_patch = true;
        self.selected_mob_ref = Some(mob_ref);
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.file_name.clear();
        self.selected_mob_ref = None;
    }
}

/// Render the file browser panel
pub fn file_panel_ui(
    ui: &mut egui::Ui,
    file_tree: &mut FileTreeState,
    sprite_registry: &mut SpriteRegistry,
    mob_registry: &mut MobAssetRegistry,
    session: &EditorSession,
    config: &EditorConfig,
    load_events: &mut MessageWriter<LoadMobEvent>,
    delete_dialog: &mut DeleteDialogState,
    delete_dir_dialog: &mut DeleteDirectoryDialogState,
    unsaved_dialog: &mut UnsavedChangesDialog,
    new_folder_dialog: &mut NewFolderDialog,
    new_mob_dialog: &mut NewMobDialog,
) {
    ui.heading("Mob Files");

    // Refresh button - refreshes file tree, sprite registry, and mob registry
    if ui.button("🔄 Refresh").clicked() {
        file_tree.needs_refresh = true;
        sprite_registry.needs_refresh = true;
        mob_registry.needs_refresh = true;
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
                    mob_registry,
                    session,
                    config,
                    load_events,
                    delete_dialog,
                    delete_dir_dialog,
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
    mob_registry: &MobAssetRegistry,
    session: &EditorSession,
    config: &EditorConfig,
    load_events: &mut MessageWriter<LoadMobEvent>,
    delete_dialog: &mut DeleteDialogState,
    delete_dir_dialog: &mut DeleteDirectoryDialogState,
    unsaved_dialog: &mut UnsavedChangesDialog,
    new_folder_dialog: &mut NewFolderDialog,
    new_mob_dialog: &mut NewMobDialog,
) {
    let is_selected = file_tree.selected.as_ref() == Some(&node.path);
    let is_current = session.current_path.as_ref() == Some(&node.path);

    if node.is_directory {
        // Directory node with collapse/expand
        let icon = if node.expanded { "📂" } else { "📁" };
        let display_name = truncate_filename(&node.name, MAX_FILENAME_DISPLAY_LEN);
        let header = egui::CollapsingHeader::new(format!("{} {}", icon, display_name))
            .id_salt(&node.path)
            .default_open(node.expanded)
            .show(ui, |ui| {
                for child in &node.children {
                    render_file_node(
                        ui,
                        child,
                        file_tree,
                        mob_registry,
                        session,
                        config,
                        load_events,
                        delete_dialog,
                        delete_dir_dialog,
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
                new_mob_dialog.open_patch();
                ui.close();
            }

            ui.separator();

            if ui.button("📁 New Folder...").clicked() {
                new_folder_dialog.open(node.path.clone());
                ui.close();
            }

            // Only show delete for non-root directories (not "base" or "extended")
            let is_root = node.name == "base" || node.name == "extended";
            if !is_root {
                ui.separator();
                if ui
                    .add(egui::Button::new("🗑 Delete...").fill(egui::Color32::from_rgb(80, 30, 30)))
                    .clicked()
                {
                    delete_dir_dialog.open(node.path.clone(), node.name.clone());
                    ui.close();
                }
            }
        });
    } else {
        // File node
        let icon = if node.name.ends_with(".mob") {
            "📄"
        } else {
            "📋" // .mobpatch
        };

        // Check registration status
        let is_registered = mob_registry.is_registered(&node.path, config);

        // Truncate long filenames, show full name on hover
        let display_name = truncate_filename(&node.name, MAX_FILENAME_DISPLAY_LEN);
        let label = format!("{} {}", icon, display_name);
        let mut text = egui::RichText::new(label);

        if is_current {
            text = text.strong();
            if session.is_modified {
                text = text.color(egui::Color32::YELLOW);
            }
        }

        let response = ui.horizontal(|ui| {
            let label_response = ui.selectable_label(is_selected, text);

            // Registration indicator (small, after filename)
            if is_registered {
                ui.label(egui::RichText::new("✔").small().color(egui::Color32::from_rgb(100, 200, 100)));
            } else {
                ui.label(egui::RichText::new("⚠").small().color(egui::Color32::YELLOW));
            }

            label_response
        }).inner;

        // Show full filename on hover if truncated
        if display_name != node.name {
            response.clone().on_hover_text(&node.name);
        }

        if response.clicked() {
            file_tree.selected = Some(node.path.clone());

            // Check if we're clicking the same file that's already loaded
            let is_same_file = session.current_path.as_ref() == Some(&node.path);
            if !is_same_file {
                // Check if we're loading a different file with unsaved changes
                if session.is_modified {
                    // Show unsaved changes dialog
                    unsaved_dialog.open(UnsavedAction::LoadFile(node.path.clone()));
                } else {
                    load_events.write(LoadMobEvent {
                        path: node.path.clone(),
                    });
                }
            }
            // Skip loading if same file is already loaded
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

            if is_mob && ui.button("📋 Create Patch...").clicked() {
                // Extract mob reference from path (e.g., "xhitara/spitter" from ".../mobs/xhitara/spitter.mob")
                let path_str = node.path.to_string_lossy();
                if let Some(mobs_idx) = path_str.find("mobs/") {
                    let relative = &path_str[mobs_idx + 5..]; // Skip "mobs/"
                    if let Some(mob_ref) = relative.strip_suffix(".mob") {
                        new_mob_dialog.open_patch_for_mob(mob_ref.to_string());
                    } else {
                        new_mob_dialog.open_patch();
                    }
                } else {
                    new_mob_dialog.open_patch();
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

/// Render the delete directory confirmation dialog window
pub fn render_delete_directory_dialog(
    ctx: &egui::Context,
    state: &mut DeleteDirectoryDialogState,
    delete_events: &mut MessageWriter<DeleteDirectoryEvent>,
) {
    if !state.is_open {
        return;
    }

    egui::Window::new("Confirm Delete Directory")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(format!(
                "Are you sure you want to delete '{}'?",
                state.dir_name
            ));

            if state.contained_files.is_empty() {
                ui.label(
                    egui::RichText::new("This directory is empty.")
                        .small()
                        .color(egui::Color32::GRAY),
                );
            } else {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(format!(
                        "The following {} file(s) will also be deleted:",
                        state.contained_files.len()
                    ))
                    .color(egui::Color32::from_rgb(255, 180, 50)),
                );

                egui::ScrollArea::vertical()
                    .max_height(150.0)
                    .show(ui, |ui| {
                        for file in &state.contained_files {
                            ui.label(
                                egui::RichText::new(format!("  • {}", file))
                                    .small()
                                    .color(egui::Color32::GRAY),
                            );
                        }
                    });
            }

            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("All contents will be moved to your system trash.")
                    .small()
                    .color(egui::Color32::GRAY),
            );

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    state.close();
                }

                if ui
                    .add(egui::Button::new("Delete All").fill(egui::Color32::from_rgb(150, 50, 50)))
                    .clicked()
                {
                    delete_events.write(DeleteDirectoryEvent {
                        path: state.dir_path.clone(),
                    });
                    state.close();
                }
            });
        });
}
