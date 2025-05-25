use bevy::{
    app::{Plugin, Update},
    prelude::{in_state, AppExtStates, IntoScheduleConfigs, OnEnter},
};

use super::{
    data::{AppState, GameState, MainMenuState},
    systems::{
        cleanup_state_system, enter_game_end_system, enter_title_menu_state_system,
        reset_states_on_app_state_transition_system, reset_states_on_game_state_transition_system,
        toggle_game_state_system,
    },
    PauseMenuState, ToggleGameStateEvent,
};

/// Plugin for managing game states and their transitions
pub(crate) struct ThetawaveStatesPlugin;

impl Plugin for ThetawaveStatesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_state::<AppState>() // start game in the main menu state
            .init_state::<GameState>() // start the game in playing state
            .init_state::<MainMenuState>() // start the game in the None state
            .init_state::<PauseMenuState>() // start the game in the pause menu
            .add_event::<ToggleGameStateEvent>()
            .add_systems(OnEnter(AppState::MainMenu), enter_title_menu_state_system)
            // Toggle whether the game is paused
            .add_systems(
                Update,
                (
                    toggle_game_state_system.run_if(in_state(AppState::Game)),
                    enter_game_end_system
                        .run_if(in_state(AppState::Game))
                        .run_if(in_state(GameState::Playing)),
                ),
            )
            .add_systems(Update, reset_states_on_app_state_transition_system)
            .add_systems(Update, reset_states_on_game_state_transition_system)
            .add_systems(Update, cleanup_state_system::<AppState>)
            .add_systems(Update, cleanup_state_system::<MainMenuState>)
            .add_systems(Update, cleanup_state_system::<GameState>)
            .add_systems(Update, cleanup_state_system::<PauseMenuState>);
    }
}
