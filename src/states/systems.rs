use super::{data::Cleanup, GameState, MainMenuState, PauseMenuState};
use bevy::{
    input::ButtonInput,
    prelude::{
        Commands, DespawnRecursiveExt, Entity, EventReader, KeyCode, NextState, Query, Res, ResMut,
        State, StateTransitionEvent, States,
    },
};

/// A system that cleans up entities after exiting states
pub(super) fn cleanup_state_system<S: States>(
    mut cmds: Commands,
    mut state_trans_event: EventReader<StateTransitionEvent<S>>,
    cleanup_entities_q: Query<(Entity, &Cleanup<S>)>,
) {
    for event in state_trans_event.read() {
        if let Some(exited_state) = &event.exited {
            for (entity, cleanup) in cleanup_entities_q.iter() {
                if cleanup.states.contains(exited_state) {
                    cmds.entity(entity).despawn_recursive();
                }
            }
        }
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
