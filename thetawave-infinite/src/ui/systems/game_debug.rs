use bevy::ecs::{error::Result, system::ResMut};
use bevy_egui::{
    egui::{menu, TopBottomPanel},
    EguiContexts,
};

#[cfg(feature = "debug")]
use thetawave_starter::PhysicsDebugSettings;

#[cfg(feature = "debug")]
/// This function is a system that handles the egui options menu
pub(in crate::ui) fn game_debug_menu_system(
    mut contexts: EguiContexts,
    mut physics_debug_settings: ResMut<PhysicsDebugSettings>,
) -> Result {
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

            ui.menu_button("Physics", |ui| {
                ui.checkbox(&mut physics_debug_settings.gizmos_enabled, "Gizmos");
                ui.checkbox(
                    &mut physics_debug_settings.diagnostics_enabled,
                    "Diagnostics",
                );
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

    Ok(())
}
