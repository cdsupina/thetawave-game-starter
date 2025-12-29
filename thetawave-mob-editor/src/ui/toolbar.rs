use bevy::ecs::message::MessageWriter;
use bevy_egui::egui;

use crate::{
    data::EditorSession,
    file::{ReloadMobEvent, SaveMobEvent},
};

/// Render the top toolbar with action buttons
pub fn toolbar_ui(
    ctx: &mut egui::Context,
    session: &EditorSession,
    save_events: &mut MessageWriter<SaveMobEvent>,
    reload_events: &mut MessageWriter<ReloadMobEvent>,
) {
    egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            // Action buttons
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
}
