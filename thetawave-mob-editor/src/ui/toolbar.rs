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
