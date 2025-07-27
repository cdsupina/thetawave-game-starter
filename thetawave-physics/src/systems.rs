use avian2d::prelude::{Physics, PhysicsTime};
use bevy::{
    prelude::{EventReader, ResMut, StateTransitionEvent},
    time::Time,
};
use thetawave_states::GameState;

#[cfg(feature = "physics_debug")]
use crate::PhysicsDebugSettings;

#[cfg(feature = "physics_debug")]
use avian2d::prelude::PhysicsGizmos;

#[cfg(feature = "physics_debug")]
use bevy::{ecs::system::Res, gizmos::config::GizmoConfigStore};

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

// Toggle physics debug settings when PhysicsDebugSettings resource is changed
#[cfg(feature = "physics_debug")]
pub(super) fn toggle_physics_debug_system(
    mut config_store: ResMut<GizmoConfigStore>,
    mut physics_diagnostics: ResMut<avian2d::prelude::PhysicsDiagnosticsUiSettings>,
    physics_debug_settings: Res<PhysicsDebugSettings>,
) {
    let config = config_store.config_mut::<PhysicsGizmos>().0;
    config.enabled = physics_debug_settings.gizmos_enabled;

    physics_diagnostics.enabled = physics_debug_settings.diagnostics_enabled;
}
