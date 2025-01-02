use super::systems::{
    menu_button_action_system, menu_button_focus_system, options_menu_system,
    setup_character_selection_system, setup_options_menu_system, setup_pause_menu_system,
    setup_pause_options_system, setup_title_menu_system, setup_ui_system,
    website_footer_button_focus_system,
};
use crate::states::{AppState, MainMenuState, PauseMenuState};
use bevy::{
    app::{Plugin, Update},
    prelude::{in_state, Condition, IntoSystemConfigs, OnEnter},
};
use bevy_alt_ui_navigation_lite::NavRequestSystem;
use bevy_egui::EguiPlugin;
use bevy_hui::HuiPlugin;

// Plugin responsible for managing the Thetawave user interface components and systems
pub(crate) struct ThetawaveUiPlugin;

impl Plugin for ThetawaveUiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // Initialize required UI plugins - HuiPlugin for UI components and EguiPlugin for immediate mode GUI
        app.add_plugins((HuiPlugin, EguiPlugin));

        // Setup core UI components and main menu systems when entering the MainMenu state
        app.add_systems(OnEnter(AppState::MainMenu), setup_ui_system);

        // Initialize and setup the title menu UI components when entering Title state
        app.add_systems(OnEnter(MainMenuState::Title), setup_title_menu_system);

        // Initialize and setup the options menu UI components when entering Options state
        app.add_systems(OnEnter(MainMenuState::Options), setup_options_menu_system);

        // Initialize and setup the character selection UI components when entering Character Selection state
        app.add_systems(
            OnEnter(MainMenuState::CharacterSelection),
            setup_character_selection_system,
        );

        // Initialize and setup the pause menu ui components when entering the paused state
        app.add_systems(OnEnter(PauseMenuState::Main), setup_pause_menu_system);

        // Initialize and setup the options pause menu when inetering the paused options state
        app.add_systems(OnEnter(PauseMenuState::Options), setup_pause_options_system);

        // Add update systems that run every frame:
        app.add_systems(
            Update,
            (
                menu_button_action_system.after(NavRequestSystem),
                menu_button_focus_system.after(NavRequestSystem),
                website_footer_button_focus_system.after(NavRequestSystem),
                options_menu_system
                    .run_if(in_state(MainMenuState::Options).or(in_state(PauseMenuState::Options))),
            ),
        );
    }
}
