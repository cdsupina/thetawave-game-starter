use bevy::{
    app::{Plugin, Update},
    prelude::{AppExtStates, IntoScheduleConfigs, in_state},
};

use super::{
    data::{
        AppState, DebugState, GameState, MainMenuState, PauseMenuState, ToggleDebugStateEvent,
        ToggleGameStateEvent,
    },
    systems::{
        cleanup_state_system, enter_game_end_system,
        reset_states_on_app_state_transition_system, reset_states_on_game_state_transition_system,
        toggle_debug_state_system, toggle_game_state_system,
    },
};

/// Plugin for managing game states and their transitions
pub struct ThetawaveStatesPlugin;

impl Plugin for ThetawaveStatesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_state::<AppState>() // starts in MainMenuLoading
            .init_state::<GameState>() // starts in Initializing, transitions to Playing after assets merge
            .init_state::<MainMenuState>() // start the game in the None state
            .init_state::<PauseMenuState>() // start the game in the pause menu
            .init_state::<DebugState>() // start the game in the None state
            .add_message::<ToggleGameStateEvent>()
            .add_message::<ToggleDebugStateEvent>()
            // Note: enter_title_menu_state_system and enter_playing_state_system
            // are registered in thetawave-starter with proper ordering after AssetMergeSet
            // Toggle whether the game is paused
            .add_systems(
                Update,
                (
                    toggle_game_state_system.run_if(in_state(AppState::Game)),
                    enter_game_end_system
                        .run_if(in_state(AppState::Game))
                        .run_if(in_state(GameState::Playing)),
                    toggle_debug_state_system,
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
