use bevy::ecs::message::MessageWriter;
use bevy_egui::egui;

use crate::{data::EditorSession, file::SaveMobEvent};

/// Render the top toolbar with menus and action buttons
#[allow(deprecated)]
pub fn toolbar_ui(
    ctx: &mut egui::Context,
    session: &mut EditorSession,
    save_events: &mut MessageWriter<SaveMobEvent>,
) {
    egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            // File menu
            ui.menu_button("File", |ui| {
                if ui.button("New Mob...").clicked() {
                    // TODO: Open new file dialog
                    ui.close();
                }

                if ui.button("New Patch...").clicked() {
                    // TODO: Open new patch dialog
                    ui.close();
                }

                ui.separator();

                if ui.button("Save").clicked() {
                    if session.current_path.is_some() {
                        save_events.write(SaveMobEvent { path: None });
                    }
                    ui.close();
                }

                if ui.button("Save As...").clicked() {
                    // TODO: Open save as dialog
                    ui.close();
                }

                ui.separator();

                if ui.button("Reload from Disk").clicked() {
                    // TODO: Send reload event
                    ui.close();
                }
            });

            // Edit menu
            ui.menu_button("Edit", |ui| {
                let can_undo = session.history.can_undo();
                let can_redo = session.history.can_redo();

                if ui
                    .add_enabled(can_undo, egui::Button::new("Undo"))
                    .clicked()
                {
                    if let Some(mob) = &session.current_mob {
                        if let Some(prev) = session.history.undo(mob) {
                            session.current_mob = Some(prev);
                            session.is_modified = true;
                        }
                    }
                    ui.close();
                }

                if ui
                    .add_enabled(can_redo, egui::Button::new("Redo"))
                    .clicked()
                {
                    if let Some(mob) = &session.current_mob {
                        if let Some(next) = session.history.redo(mob) {
                            session.current_mob = Some(next);
                            session.is_modified = true;
                        }
                    }
                    ui.close();
                }
            });

            // View menu
            ui.menu_button("View", |ui| {
                if ui.button("Reset Layout").clicked() {
                    // TODO: Reset panel sizes
                    ui.close();
                }
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Quick action buttons on the right
                let save_enabled = session.is_modified && session.current_path.is_some();

                if ui
                    .add_enabled(save_enabled, egui::Button::new("Save"))
                    .clicked()
                {
                    save_events.write(SaveMobEvent { path: None });
                }

                if ui.button("Reload").clicked() {
                    // TODO: Send reload event
                }
            });
        });
    });
}
