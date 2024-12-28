use super::systems::{
    button_system, menu_button_action_system, options_menu_system, setup_options_menu_system,
    setup_title_menu_system, setup_ui_system,
};
use crate::states::{AppState, MainMenuState};
use bevy::{
    app::{Plugin, Update},
    prelude::{in_state, IntoSystemConfigs, OnEnter},
};
use bevy_alt_ui_navigation_lite::NavRequestSystem;
use bevy_egui::EguiPlugin;
use bevy_hui::HuiPlugin;

// Plugin for managing the Thetawave UI
pub(crate) struct ThetawaveUiPlugin;

impl Plugin for ThetawaveUiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // Add HuiPlugin and HuiAutoLoadPlugin with UI components path
        app.add_plugins((HuiPlugin, EguiPlugin));

        // Add systems to setup UI and main menu when entering MainMenu state
        app.add_systems(OnEnter(AppState::MainMenu), setup_ui_system);

        // Setup the title menu ui
        app.add_systems(OnEnter(MainMenuState::Title), setup_title_menu_system);

        // Add system to setup options menu when entering OptionsMenu state
        app.add_systems(OnEnter(MainMenuState::Options), setup_options_menu_system);

        // Add UI systems that run after navigation system:
        // - Button system for handling button interactions
        // - Print system for logging navigation events
        app.add_systems(
            Update,
            (
                button_system.after(NavRequestSystem),
                menu_button_action_system.after(NavRequestSystem),
                options_menu_system.run_if(in_state(MainMenuState::Options)),
            ),
        );
    }
}
