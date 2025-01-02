use bevy::{
    app::{Plugin, Update},
    prelude::{in_state, AppExtStates, IntoSystemConfigs, OnEnter, OnExit},
};

use super::{
    data::{AppState, GameCleanup, GameState, MainMenuCleanup, MainMenuState, PauseOptionsCleanup},
    systems::{cleanup_state_system, enter_title_menu_state_system, toggle_game_state},
    CharacterSelectionCleanup, OptionsMenuCleanup, PauseCleanup, PauseMainCleanup, PauseMenuState,
    TitleMenuCleanup,
};

/// Plugin for managing game states and their transitions
pub(crate) struct ThetawaveStatesPlugin;

impl Plugin for ThetawaveStatesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_state::<AppState>() // start game in the main menu state
            .init_state::<GameState>() // start the game in playing state
            .init_state::<MainMenuState>() // start the game in the None state
            .init_state::<PauseMenuState>() // start the game in the pause menu
            .add_systems(OnEnter(AppState::MainMenu), enter_title_menu_state_system)
            // Add cleanup system for when exiting MainMenu state
            .add_systems(
                OnExit(AppState::MainMenu),
                cleanup_state_system::<MainMenuCleanup>,
            )
            // Add cleanup system for when exiting Game state
            .add_systems(OnExit(AppState::Game), cleanup_state_system::<GameCleanup>)
            // Add cleanup system for when exiting Paused state
            .add_systems(
                OnExit(GameState::Paused),
                cleanup_state_system::<PauseCleanup>,
            )
            // Add cleanup system for the main pause menu
            .add_systems(
                OnExit(PauseMenuState::Main),
                cleanup_state_system::<PauseMainCleanup>,
            )
            // Add cleanup system for the options pause menu
            .add_systems(
                OnExit(PauseMenuState::Options),
                cleanup_state_system::<PauseOptionsCleanup>,
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
            )
            // Toggle whether the game is paused
            .add_systems(Update, toggle_game_state.run_if(in_state(AppState::Game)));
    }
}
