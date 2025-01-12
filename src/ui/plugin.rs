use super::systems::{
    character_selection::{
        cycle_carousel_system, set_characters_system, setup_character_selection_system,
        update_carousel_ui_system,
    },
    hui::setup_hui_system,
    loading::{setup_loading_ui_system, update_loading_bar_system},
    menu_button_action_system, menu_button_focus_system,
    options::{options_menu_system, persist_options_system, setup_options_menu_system},
    pause::{setup_pause_menu_system, setup_pause_options_system},
    title::{setup_title_menu_system, website_footer_button_focus_system},
};
use crate::states::{AppState, MainMenuState, PauseMenuState};
use bevy::{
    app::{Plugin, Update},
    prelude::{in_state, Condition, IntoSystemConfigs, OnEnter, OnExit},
};
use bevy_alt_ui_navigation_lite::NavRequestSystem;
use bevy_asset_loader::loading_state::LoadingStateSet;
use bevy_egui::EguiPlugin;
use bevy_hui::HuiPlugin;

// Plugin responsible for managing the Thetawave user interface components and systems
pub(crate) struct ThetawaveUiPlugin;

impl Plugin for ThetawaveUiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // Initialize required UI plugins - HuiPlugin for UI components and EguiPlugin for immediate mode GUI
        app.add_plugins((HuiPlugin, EguiPlugin))
            .add_systems(OnEnter(AppState::MainMenuLoading), setup_loading_ui_system)
            // Setup core UI components and main menu systems when entering the MainMenu state
            .add_systems(OnEnter(AppState::MainMenu), setup_hui_system)
            // Initialize and setup the title menu UI components when entering Title state
            .add_systems(OnEnter(MainMenuState::Title), setup_title_menu_system)
            // Initialize and setup the options menu UI components when entering Options state
            .add_systems(OnEnter(MainMenuState::Options), setup_options_menu_system)
            // Initialize and setup the character selection UI components when entering Character Selection state
            .add_systems(
                OnEnter(MainMenuState::CharacterSelection),
                setup_character_selection_system,
            )
            // Initialize and setup the pause menu ui components when entering the paused state
            .add_systems(OnEnter(PauseMenuState::Main), setup_pause_menu_system)
            // Initialize and setup the options pause menu when inetering the paused options state
            .add_systems(OnEnter(PauseMenuState::Options), setup_pause_options_system)
            // Add update systems that run every frame:
            .add_systems(
                Update,
                (
                    menu_button_action_system.after(NavRequestSystem),
                    menu_button_focus_system.after(NavRequestSystem),
                    website_footer_button_focus_system.after(NavRequestSystem),
                    update_loading_bar_system
                        .run_if(
                            in_state(AppState::MainMenuLoading).or(in_state(AppState::GameLoading)),
                        )
                        .after(LoadingStateSet(AppState::MainMenuLoading))
                        .after(LoadingStateSet(AppState::GameLoading)),
                ),
            )
            // Run options systems in main menu and pause menu options states
            .add_systems(
                Update,
                (options_menu_system, persist_options_system)
                    .run_if(in_state(MainMenuState::Options).or(in_state(PauseMenuState::Options))),
            )
            // Run carousel systems in character selection state
            .add_systems(
                Update,
                (cycle_carousel_system, update_carousel_ui_system)
                    .run_if(in_state(MainMenuState::CharacterSelection)),
            )
            // Send the event to set the characters to the active characters in the carousel
            .add_systems(OnExit(AppState::MainMenu), set_characters_system);
    }
}
