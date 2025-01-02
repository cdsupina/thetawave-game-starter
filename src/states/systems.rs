use super::{GameState, MainMenuState, PauseMenuState};
use bevy::{
    input::ButtonInput,
    prelude::{
        Commands, Component, DespawnRecursiveExt, Entity, KeyCode, NextState, Query, Res, ResMut,
        State, With,
    },
};

/// A system that cleans up entities marked with a specific component type
pub(super) fn cleanup_state_system<T: Component>(
    mut cmds: Commands,
    cleanup_entities_q: Query<Entity, With<T>>,
) {
    // Iterate through all entities with component T and despawn them
    for e in cleanup_entities_q.iter() {
        cmds.entity(e).despawn_recursive();
    }
}

/// System to enter the title menu state
pub(super) fn enter_title_menu_state_system(mut next_state: ResMut<NextState<MainMenuState>>) {
    next_state.set(MainMenuState::Title);
}

/// Toggle weather the game is paused or playing
pub(super) fn toggle_game_state(
    mut next_game_state: ResMut<NextState<GameState>>,
    mut next_pause_state: ResMut<NextState<PauseMenuState>>,
    current_state: Res<State<GameState>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_released(KeyCode::Escape) {
        match **current_state {
            GameState::Playing => {
                next_game_state.set(GameState::Paused);
                next_pause_state.set(PauseMenuState::Main);
            }
            GameState::Paused => {
                next_game_state.set(GameState::Playing);
                next_pause_state.set(PauseMenuState::None);
            }
        };
    }
}
