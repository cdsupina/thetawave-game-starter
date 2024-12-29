use super::systems::{
    menu_button_action_system, menu_button_focus_system, options_menu_system,
    setup_options_menu_system, setup_title_menu_system, setup_ui_system,
    website_footer_button_focus_system,
};
use crate::states::{AppState, MainMenuState};
use bevy::{
    app::{Plugin, Update},
    prelude::{in_state, IntoSystemConfigs, OnEnter},
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

        // Add update systems that run every frame:
        // - Handle menu button clicks after navigation
        // - Update menu button focus states after navigation
        // - Handle website footer button focus after navigation
        // - Process options menu logic when in Options state
        app.add_systems(
            Update,
            (
                menu_button_action_system.after(NavRequestSystem),
                menu_button_focus_system.after(NavRequestSystem),
                website_footer_button_focus_system.after(NavRequestSystem),
                options_menu_system.run_if(in_state(MainMenuState::Options)),
            ),
        );
    }
}
