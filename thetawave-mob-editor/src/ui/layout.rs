use bevy::{ecs::message::MessageWriter, prelude::*};
use bevy_egui::{EguiContexts, egui};

use crate::{
    data::EditorSession,
    file::{DeleteMobEvent, FileTreeState, LoadMobEvent, NewMobEvent, ReloadMobEvent, SaveMobEvent},
    plugin::EditorConfig,
    states::EditorState,
};

use super::{
    file_panel_ui, properties_panel_ui, render_delete_dialog, toolbar_ui,
    DeleteDialogState, FileDialogState, FileDialogResult,
};

/// Main UI layout system that renders all egui panels
pub fn main_ui_system(
    mut contexts: EguiContexts,
    mut session: ResMut<EditorSession>,
    mut file_tree: ResMut<FileTreeState>,
    state: Res<State<EditorState>>,
    config: Res<EditorConfig>,
    time: Res<Time>,
    mut load_events: MessageWriter<LoadMobEvent>,
    mut save_events: MessageWriter<SaveMobEvent>,
    mut reload_events: MessageWriter<ReloadMobEvent>,
    mut new_events: MessageWriter<NewMobEvent>,
    mut delete_events: MessageWriter<DeleteMobEvent>,
    mut file_dialog: ResMut<FileDialogState>,
    mut delete_dialog: ResMut<DeleteDialogState>,
    mut frames_passed: Local<u8>,
) {
    // Skip first two frames to let egui initialize properly
    if *frames_passed < 2 {
        *frames_passed += 1;
        return;
    }

    // Poll for file dialog results
    if let Some(result) = file_dialog.poll_result() {
        match result {
            FileDialogResult::NewFile { path, is_patch } => {
                // Extract name from filename (without extension)
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unnamed")
                    .to_string();

                new_events.write(NewMobEvent {
                    path,
                    name,
                    is_patch,
                });
            }
            FileDialogResult::OpenFile { path } => {
                // Open the selected file
                load_events.write(LoadMobEvent { path });
            }
            FileDialogResult::Cancelled => {
                // User cancelled, nothing to do
            }
        }
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    // Update status message expiry
    session.update_status(&time);

    // Top toolbar
    toolbar_ui(
        ctx,
        &mut session,
        &mut save_events,
        &mut reload_events,
        &mut *file_dialog,
        &config,
    );

    // Left panel - File browser
    egui::SidePanel::left("file_panel")
        .default_width(250.0)
        .min_width(200.0)
        .resizable(true)
        .show(ctx, |ui| {
            file_panel_ui(
                ui,
                &mut file_tree,
                &session,
                &mut load_events,
                &mut *file_dialog,
                &mut *delete_dialog,
                &config,
            );
        });

    // Right panel - Properties (only when editing)
    if *state.get() == EditorState::Editing {
        egui::SidePanel::right("properties_panel")
            .default_width(300.0)
            .min_width(250.0)
            .resizable(true)
            .show(ctx, |ui| {
                properties_panel_ui(ui, &mut session);
            });
    }

    // Bottom status bar
    egui::TopBottomPanel::bottom("status_bar")
        .exact_height(24.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // File path
                if let Some(path) = &session.current_path {
                    ui.label(format!("File: {}", path.display()));
                } else {
                    ui.label("No file loaded");
                }

                ui.separator();

                // Modified indicator
                if session.is_modified {
                    ui.colored_label(egui::Color32::YELLOW, "Modified");
                } else {
                    ui.label("Saved");
                }

                // Status message
                if let Some((msg, _)) = &session.status_message {
                    ui.separator();
                    ui.label(msg);
                }
            });
        });

    // Central panel - Preview area (placeholder for now)
    egui::CentralPanel::default().show(ctx, |ui| {
        match *state.get() {
            EditorState::Loading => {
                ui.centered_and_justified(|ui| {
                    ui.heading("Loading...");
                });
            }
            EditorState::Browsing => {
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Mob Editor");
                        ui.add_space(20.0);
                        ui.label("Select a .mob or .mobpatch file from the file browser");
                        ui.label("or use File → New Mob/Patch to create one.");
                    });
                });
            }
            EditorState::Editing => {
                // Preview area - will be replaced with actual sprite preview later
                ui.heading("Preview");
                ui.separator();

                if let Some(name) = session.get_mob_name() {
                    ui.label(format!("Editing: {}", name));
                }

                // Placeholder for mob preview
                let available = ui.available_size();
                let (rect, _response) = ui.allocate_exact_size(available, egui::Sense::hover());

                // Draw a simple grid background
                let painter = ui.painter();
                let grid_color = egui::Color32::from_gray(40);
                let grid_spacing = 20.0;

                let min = rect.min;
                let max = rect.max;

                // Vertical lines
                let mut x = min.x;
                while x < max.x {
                    painter.line_segment(
                        [egui::pos2(x, min.y), egui::pos2(x, max.y)],
                        egui::Stroke::new(1.0, grid_color),
                    );
                    x += grid_spacing;
                }

                // Horizontal lines
                let mut y = min.y;
                while y < max.y {
                    painter.line_segment(
                        [egui::pos2(min.x, y), egui::pos2(max.x, y)],
                        egui::Stroke::new(1.0, grid_color),
                    );
                    y += grid_spacing;
                }

                // Center crosshair
                let center = rect.center();
                painter.line_segment(
                    [
                        egui::pos2(center.x - 20.0, center.y),
                        egui::pos2(center.x + 20.0, center.y),
                    ],
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 100, 100)),
                );
                painter.line_segment(
                    [
                        egui::pos2(center.x, center.y - 20.0),
                        egui::pos2(center.x, center.y + 20.0),
                    ],
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 100, 100)),
                );
            }
        }
    });

    // Render delete dialog window
    render_delete_dialog(ctx, &mut *delete_dialog, &mut delete_events);
}
