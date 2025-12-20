use std::path::PathBuf;
use std::sync::{Arc, Mutex, mpsc::{channel, Receiver, Sender}};

use bevy::{ecs::message::MessageWriter, prelude::*};
use bevy_egui::egui;

use crate::{
    data::EditorSession,
    file::{DeleteMobEvent, FileNode, FileTreeState, LoadMobEvent},
    plugin::EditorConfig,
};

/// Result from a file dialog operation
pub enum FileDialogResult {
    NewFile { path: PathBuf, is_patch: bool },
    OpenFile { path: PathBuf },
    Cancelled,
}

/// State for managing file dialogs (uses native OS dialogs via rfd)
#[derive(Resource)]
pub struct FileDialogState {
    /// Receiver wrapped in Arc<Mutex> for thread safety
    result_receiver: Arc<Mutex<Receiver<FileDialogResult>>>,
    /// Sender clone for spawning dialogs
    result_sender: Sender<FileDialogResult>,
    /// Whether a dialog is currently open
    pub dialog_open: bool,
}

impl Default for FileDialogState {
    fn default() -> Self {
        let (sender, receiver) = channel();
        Self {
            result_receiver: Arc::new(Mutex::new(receiver)),
            result_sender: sender,
            dialog_open: false,
        }
    }
}

impl FileDialogState {
    /// Open a save dialog for creating a new mob file
    pub fn open_new_mob_dialog(&mut self, config: &EditorConfig) {
        if self.dialog_open {
            return;
        }
        self.dialog_open = true;

        let sender = self.result_sender.clone();
        let start_dir = config.base_assets_dir.clone();

        std::thread::spawn(move || {
            let result = rfd::FileDialog::new()
                .set_title("Create New Mob")
                .set_file_name("new_mob.mob")
                .add_filter("Mob files", &["mob"])
                .set_directory(&start_dir)
                .save_file();

            let dialog_result = match result {
                Some(path) => FileDialogResult::NewFile {
                    path,
                    is_patch: false,
                },
                None => FileDialogResult::Cancelled,
            };

            let _ = sender.send(dialog_result);
        });
    }

    /// Open a save dialog for creating a new mob patch file
    pub fn open_new_patch_dialog(&mut self, config: &EditorConfig) {
        if self.dialog_open {
            return;
        }
        self.dialog_open = true;

        let sender = self.result_sender.clone();
        let start_dir = config
            .extended_assets_dir
            .clone()
            .unwrap_or_else(|| config.base_assets_dir.clone());

        std::thread::spawn(move || {
            let result = rfd::FileDialog::new()
                .set_title("Create New Mob Patch")
                .set_file_name("new_patch.mobpatch")
                .add_filter("Mob patch files", &["mobpatch"])
                .set_directory(&start_dir)
                .save_file();

            let dialog_result = match result {
                Some(path) => FileDialogResult::NewFile {
                    path,
                    is_patch: true,
                },
                None => FileDialogResult::Cancelled,
            };

            let _ = sender.send(dialog_result);
        });
    }

    /// Open a file dialog for opening an existing mob/mobpatch file
    pub fn open_file_dialog(&mut self, config: &EditorConfig) {
        if self.dialog_open {
            return;
        }
        self.dialog_open = true;

        let sender = self.result_sender.clone();
        let start_dir = config.base_assets_dir.clone();

        std::thread::spawn(move || {
            let result = rfd::FileDialog::new()
                .set_title("Open Mob File")
                .add_filter("Mob files", &["mob", "mobpatch"])
                .add_filter("All files", &["*"])
                .set_directory(&start_dir)
                .pick_file();

            let dialog_result = match result {
                Some(path) => FileDialogResult::OpenFile { path },
                None => FileDialogResult::Cancelled,
            };

            let _ = sender.send(dialog_result);
        });
    }

    /// Check for and process dialog results
    pub fn poll_result(&mut self) -> Option<FileDialogResult> {
        let receiver = self.result_receiver.lock().ok()?;
        match receiver.try_recv() {
            Ok(result) => {
                self.dialog_open = false;
                Some(result)
            }
            Err(_) => None,
        }
    }
}

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

/// Render the file browser panel
pub fn file_panel_ui(
    ui: &mut egui::Ui,
    file_tree: &mut FileTreeState,
    session: &EditorSession,
    load_events: &mut MessageWriter<LoadMobEvent>,
    file_dialog: &mut FileDialogState,
    delete_dialog: &mut DeleteDialogState,
    config: &EditorConfig,
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
                    file_dialog,
                    delete_dialog,
                    config,
                );
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
    file_dialog: &mut FileDialogState,
    delete_dialog: &mut DeleteDialogState,
    config: &EditorConfig,
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
                        file_dialog,
                        delete_dialog,
                        config,
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
                file_dialog.open_new_mob_dialog(config);
                ui.close();
            }

            if ui.button("📋 New Patch...").clicked() {
                file_dialog.open_new_patch_dialog(config);
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
            load_events.write(LoadMobEvent {
                path: node.path.clone(),
            });
        }

        // Context menu for files
        response.context_menu(|ui| {
            if ui.button("📂 Open").clicked() {
                load_events.write(LoadMobEvent {
                    path: node.path.clone(),
                });
                ui.close();
            }

            ui.separator();

            // Determine if this is a mob or patch
            let is_mob = node.name.ends_with(".mob");

            if is_mob {
                if ui.button("📋 Create Patch...").clicked() {
                    file_dialog.open_new_patch_dialog(config);
                    ui.close();
                }
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
