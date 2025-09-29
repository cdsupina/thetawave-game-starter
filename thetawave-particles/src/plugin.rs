use bevy::{
    app::{Plugin, Update},
    ecs::schedule::{Condition, IntoScheduleConfigs},
    state::condition::in_state,
};
use bevy_enoki::EnokiPlugin;
use thetawave_core::{AppState, GameState};

use crate::{
    SpawnBloodEffectEvent,
    data::{
        ActivateParticleEvent, SpawnParticleEffectEvent, SpawnerParticleEffectSpawnedEvent,
        ToggleActiveParticleEvent,
    },
    spawn::{spawn_blood_effect_system, spawn_particle_effect_system},
    systems::{
        activate_particle_effect_system, blood_effect_management_system,
        particle_lifetime_management_system, particle_position_tracking_system,
        toggle_particle_effect_system,
    },
};

pub struct ThetawaveParticlesPlugin;

impl Plugin for ThetawaveParticlesPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_event::<SpawnParticleEffectEvent>();
        app.add_event::<SpawnerParticleEffectSpawnedEvent>();
        app.add_event::<ActivateParticleEvent>();
        app.add_event::<ToggleActiveParticleEvent>();
        app.add_event::<SpawnBloodEffectEvent>();

        app.add_plugins(EnokiPlugin);

        app.add_systems(
            Update,
            (
                particle_position_tracking_system,
                activate_particle_effect_system,
                toggle_particle_effect_system,
                spawn_particle_effect_system,
                spawn_blood_effect_system,
                particle_lifetime_management_system,
                blood_effect_management_system,
            )
                .run_if(in_state(AppState::Game).and(in_state(GameState::Playing))),
        );
    }
}
