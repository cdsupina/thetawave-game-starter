use avian2d::prelude::{Physics, PhysicsGizmos, PhysicsTime};
use bevy::{
    ecs::system::Res,
    gizmos::config::GizmoConfigStore,
    input::{keyboard::KeyCode, ButtonInput},
    prelude::{EventReader, ResMut, StateTransitionEvent},
    time::Time,
};
use thetawave_states::GameState;

/// Pause and resume physics on GameState change
pub(super) fn pause_physics_system(
    mut physics_time: ResMut<Time<Physics>>,
    mut game_state_trans: EventReader<StateTransitionEvent<GameState>>,
) {
    for event in game_state_trans.read() {
        if let Some(entered_state) = event.entered {
            match entered_state {
                GameState::Playing => {
                    physics_time.unpause();
                }
                GameState::Paused => {
                    physics_time.pause();
                }
                GameState::End => {}
            }
        }
    }
}

/// System for resuming physics
pub(super) fn resume_physics_system(mut physics_time: ResMut<Time<Physics>>) {
    physics_time.unpause();
}

#[cfg(feature = "physics_debug")]
pub(super) fn toggle_physics_debug_system(
    mut config_store: ResMut<GizmoConfigStore>,
    mut physics_diagnostics: ResMut<avian2d::prelude::PhysicsDiagnosticsUiSettings>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_released(KeyCode::KeyP) {
        let config = config_store.config_mut::<PhysicsGizmos>().0;
        config.enabled = !config.enabled;
        physics_diagnostics.enabled = !physics_diagnostics.enabled;
    }
}
