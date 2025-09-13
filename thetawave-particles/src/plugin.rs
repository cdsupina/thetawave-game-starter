use bevy::{
    app::{Plugin, Update},
    ecs::schedule::{Condition, IntoScheduleConfigs},
    state::condition::in_state,
};
use bevy_enoki::EnokiPlugin;
use thetawave_core::{AppState, GameState};

use crate::{
    data::{ActivateParticleEvent, SpawnParticleEffectEvent},
    spawn::spawn_particle_effect_system,
    systems::activate_particle_effect_system,
};

pub struct ThetawaveParticlesPlugin;

impl Plugin for ThetawaveParticlesPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_event::<SpawnParticleEffectEvent>();
        app.add_event::<ActivateParticleEvent>();

        app.add_plugins(EnokiPlugin);

        app.add_systems(
            Update,
            (
                activate_particle_effect_system,
                spawn_particle_effect_system,
            )
                .run_if(in_state(AppState::Game).and(in_state(GameState::Playing))),
        );
    }
}
