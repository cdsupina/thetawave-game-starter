use bevy::{
    app::Plugin,
    prelude::{AppExtStates, OnExit},
};

use super::{
    data::{AppState, CharacterSelectionCleanup, GameState, InGameCleanup, MainMenuCleanup},
    systems::cleanup_state_system,
};

/// Plugin for managing game states and their transitions
pub(crate) struct ThetawaveStatesPlugin;

impl Plugin for ThetawaveStatesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_state::<AppState>() // start game in the main menu state
            .init_state::<GameState>() // start the game in playing state
            // Add cleanup system for when exiting MainMenu state
            .add_systems(
                OnExit(AppState::MainMenu),
                cleanup_state_system::<MainMenuCleanup>,
            )
            // Add cleanup system for when exiting CharacterSelection state
            .add_systems(
                OnExit(AppState::CharacterSelectionMenu),
                cleanup_state_system::<CharacterSelectionCleanup>,
            )
            // Add cleanup system for when exiting InGame state
            .add_systems(
                OnExit(AppState::InGame),
                cleanup_state_system::<InGameCleanup>,
            );
    }
}
