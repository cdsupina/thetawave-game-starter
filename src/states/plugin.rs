use bevy::{
    app::Plugin,
    prelude::{AppExtStates, OnEnter, OnExit},
};

use super::{
    data::{AppState, GameState, InGameCleanup, MainMenuCleanup, MainMenuState},
    systems::{cleanup_state_system, enter_title_menu_state_system},
    CharacterSelectionCleanup, OptionsMenuCleanup, TitleMenuCleanup,
};

/// Plugin for managing game states and their transitions
pub(crate) struct ThetawaveStatesPlugin;

impl Plugin for ThetawaveStatesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_state::<AppState>() // start game in the main menu state
            .init_state::<GameState>() // start the game in playing state
            .init_state::<MainMenuState>() // start the game in the None state
            .add_systems(OnEnter(AppState::MainMenu), enter_title_menu_state_system)
            // Add cleanup system for when exiting MainMenu state
            .add_systems(
                OnExit(AppState::MainMenu),
                cleanup_state_system::<MainMenuCleanup>,
            )
            // Add cleanup system for when exiting InGame state
            .add_systems(
                OnExit(AppState::Game),
                cleanup_state_system::<InGameCleanup>,
            )
            // Add cleanup system for when exiting OptionsMenu state
            .add_systems(
                OnExit(MainMenuState::Options),
                cleanup_state_system::<OptionsMenuCleanup>,
            )
            // Add cleanup system for when exiting ChracterSelection state
            .add_systems(
                OnExit(MainMenuState::CharacterSelection),
                cleanup_state_system::<CharacterSelectionCleanup>,
            )
            // Add cleanup system for when exiting TitleMenu state
            .add_systems(
                OnExit(MainMenuState::Title),
                cleanup_state_system::<TitleMenuCleanup>,
            );
    }
}
