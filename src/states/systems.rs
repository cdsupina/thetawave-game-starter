use super::{GameState, MainMenuState};
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
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_released(KeyCode::Escape) {
        next_state.set(match **current_state {
            GameState::Playing => GameState::Paused,
            GameState::Paused => GameState::Playing,
        });
    }
}
