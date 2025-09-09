use bevy::{
    app::{Plugin, Update},
    ecs::schedule::{Condition, IntoScheduleConfigs},
    state::condition::in_state,
};
use thetawave_core::{AppState, GameState};

use crate::particles::spawn::spawn_particle_effect_system;

/// Plugin for managing player entities
pub(crate) struct ThetawaveParticlesPlugin;

impl Plugin for ThetawaveParticlesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(thetawave_particles::ThetawaveParticlesPlugin)
            .add_systems(
                Update,
                spawn_particle_effect_system
                    .run_if(in_state(AppState::Game).and(in_state(GameState::Playing))),
            );
    }
}
