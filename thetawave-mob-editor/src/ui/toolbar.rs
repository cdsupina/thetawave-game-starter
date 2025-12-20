use bevy::ecs::message::MessageWriter;
use bevy_egui::egui;

use crate::{
    data::EditorSession,
    file::{ReloadMobEvent, SaveMobEvent},
    plugin::EditorConfig,
};

use super::FileDialogState;

/// Render the top toolbar with menus and action buttons
pub fn toolbar_ui(
    ctx: &mut egui::Context,
    session: &mut EditorSession,
    save_events: &mut MessageWriter<SaveMobEvent>,
    reload_events: &mut MessageWriter<ReloadMobEvent>,
    file_dialog: &mut FileDialogState,
    config: &EditorConfig,
) {
    egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            // File menu
            ui.menu_button("File", |ui| {
                let dialog_busy = file_dialog.dialog_open;

                if ui
                    .add_enabled(!dialog_busy, egui::Button::new("New Mob..."))
                    .clicked()
                {
                    file_dialog.open_new_mob_dialog(config);
                    ui.close();
                }

                if ui
                    .add_enabled(!dialog_busy, egui::Button::new("New Patch..."))
                    .clicked()
                {
                    file_dialog.open_new_patch_dialog(config);
                    ui.close();
                }

                ui.separator();

                if ui
                    .add_enabled(!dialog_busy, egui::Button::new("Open..."))
                    .clicked()
                {
                    file_dialog.open_file_dialog(config);
                    ui.close();
                }

                ui.separator();

                let save_enabled = session.is_modified && session.current_path.is_some();
                if ui
                    .add_enabled(save_enabled, egui::Button::new("Save"))
                    .clicked()
                {
                    save_events.write(SaveMobEvent { path: None });
                    ui.close();
                }

                ui.separator();

                let reload_enabled = session.current_path.is_some();
                if ui
                    .add_enabled(reload_enabled, egui::Button::new("Reload from Disk"))
                    .clicked()
                {
                    reload_events.write(ReloadMobEvent);
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
                            session.check_modified();
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
                            session.check_modified();
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
                    .add_enabled(save_enabled, egui::Button::new("💾 Save"))
                    .clicked()
                {
                    save_events.write(SaveMobEvent { path: None });
                }

                let reload_enabled = session.current_path.is_some();
                if ui
                    .add_enabled(reload_enabled, egui::Button::new("🔄 Reload"))
                    .clicked()
                {
                    reload_events.write(ReloadMobEvent);
                }
            });
        });
    });
}
