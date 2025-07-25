use bevy::ecs::{error::Result, event::EventReader, system::Local};
use bevy_egui::{
    egui::{menu, TopBottomPanel},
    EguiContexts,
};
use thetawave_starter::ToggleDebugModeEvent;

/// This function is a system that handles the egui options menu
pub(in crate::ui) fn game_debug_menu_system(
    mut contexts: EguiContexts,
    mut is_active: Local<bool>,
    mut toggle_debug_event_reader: EventReader<ToggleDebugModeEvent>,
) -> Result {
    if let Some(event) = toggle_debug_event_reader.read().next() {
        *is_active = event.0;
    }

    if *is_active {
        TopBottomPanel::top("menu_bar").show(contexts.ctx_mut()?, |ui| {
            menu::bar(ui, |ui| {
                ui.menu_button("Spawn", |ui| {
                    ui.menu_button("Mob", |ui| {
                        // Add contents to the menu
                    });
                    ui.menu_button("Consumable", |ui| {
                        // Add contents to the menu
                    });
                    ui.menu_button("Item", |ui| {
                        // Add contents to the menu
                    });
                });

                ui.menu_button("Lines", |ui| {
                    ui.menu_button("Mob", |ui| {
                        // Add contents to the menu
                    });
                    ui.menu_button("Consumable", |ui| {
                        // Add contents to the menu
                    });
                    ui.menu_button("Item", |ui| {
                        // Add contents to the menu
                    });
                });
            })
        });
    }

    Ok(())
}
