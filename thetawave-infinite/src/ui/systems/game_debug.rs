use bevy::{
    ecs::{
        error::Result,
        system::{Local, Res},
    },
    input::{keyboard::KeyCode, ButtonInput},
};
use bevy_egui::{
    egui::{menu, TopBottomPanel},
    EguiContexts,
};

/// This function is a system that handles the egui options menu
pub(in crate::ui) fn game_debug_menu_system(
    mut contexts: EguiContexts,
    mut is_active: Local<bool>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) -> Result {
    if keyboard_input.just_released(KeyCode::Backquote) {
        *is_active = !*is_active;
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
