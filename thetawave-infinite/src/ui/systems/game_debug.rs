use bevy::ecs::{error::Result, system::ResMut};
use bevy_egui::{
    egui::{menu, TopBottomPanel},
    EguiContexts,
};

#[cfg(feature = "debug")]
use thetawave_starter::{InspectorDebugSettings, PhysicsDebugSettings};

#[cfg(feature = "debug")]
/// System that handles the egui debug menu
pub(in crate::ui) fn game_debug_menu_system(
    mut contexts: EguiContexts,
    mut physics_debug_settings: ResMut<PhysicsDebugSettings>,
    mut inspector_debug_settings: ResMut<InspectorDebugSettings>,
) -> Result {
    TopBottomPanel::top("menu_bar").show(contexts.ctx_mut()?, |ui| {
        menu::bar(ui, |ui| {
            ui.menu_button("Inspector", |ui| {
                ui.checkbox(
                    &mut inspector_debug_settings.inspector_enabled,
                    "World Inspector",
                );
            });

            ui.menu_button("Physics", |ui| {
                ui.checkbox(&mut physics_debug_settings.gizmos_enabled, "Gizmos");
                ui.checkbox(
                    &mut physics_debug_settings.diagnostics_enabled,
                    "Diagnostics",
                );
            });

            /*
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
            */
        })
    });

    Ok(())
}
