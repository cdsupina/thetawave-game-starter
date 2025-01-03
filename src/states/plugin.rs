use bevy::{
    app::{Plugin, Update},
    prelude::{in_state, AppExtStates, IntoSystemConfigs, OnEnter},
};

use super::{
    data::{AppState, GameState, MainMenuState},
    systems::{cleanup_state_system, enter_title_menu_state_system, toggle_game_state},
    PauseMenuState,
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
            // Toggle whether the game is paused
            .add_systems(Update, toggle_game_state.run_if(in_state(AppState::Game)))
            .add_systems(Update, cleanup_state_system::<AppState>)
            .add_systems(Update, cleanup_state_system::<MainMenuState>)
            .add_systems(Update, cleanup_state_system::<GameState>)
            .add_systems(Update, cleanup_state_system::<PauseMenuState>);
    }
}
