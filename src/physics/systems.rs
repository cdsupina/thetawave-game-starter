use crate::states::GameState;
use avian2d::prelude::{Physics, PhysicsTime};
use bevy::{
    prelude::{EventReader, ResMut, StateTransitionEvent},
    time::Time,
};

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
            }
        }
    }
}

/// System for resuming physics
pub(super) fn resume_physics_system(mut physics_time: ResMut<Time<Physics>>) {
    physics_time.unpause();
}
