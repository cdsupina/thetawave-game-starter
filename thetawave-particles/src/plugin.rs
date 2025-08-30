use bevy::{
    app::{Plugin, Update},
    ecs::schedule::{Condition, IntoScheduleConfigs},
    state::condition::in_state,
};
use bevy_enoki::EnokiPlugin;
use thetawave_states::{AppState, GameState};

use crate::{
    ParticleEffectType,
    data::{ActivateParticleEvent, SpawnParticleEffectEvent},
    systems::activate_particle_effect_system,
};

pub struct ThetawaveParticlesPlugin;

impl Plugin for ThetawaveParticlesPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.register_type::<ParticleEffectType>();

        app.add_event::<SpawnParticleEffectEvent>();
        app.add_event::<ActivateParticleEvent>();

        app.add_plugins(EnokiPlugin);

        app.add_systems(
            Update,
            activate_particle_effect_system
                .run_if(in_state(AppState::Game).and(in_state(GameState::Playing))),
        );
    }
}
