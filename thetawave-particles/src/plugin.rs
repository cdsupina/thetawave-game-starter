use bevy::{
    app::{Plugin, Update},
    ecs::schedule::{IntoScheduleConfigs, SystemCondition},
    state::condition::in_state,
};
use bevy_enoki::EnokiPlugin;
use thetawave_core::{AppState, GameState};

use crate::{
    SpawnBloodEffectEvent,
    data::{
        ActivateParticleEvent, SpawnExplosionEffectEvent, SpawnProjectileDespawnEffectEvent,
        SpawnProjectileHitEffectEvent, SpawnProjectileTrailEffectEvent, SpawnSpawnerEffectEvent,
        SpawnerParticleEffectSpawnedEvent, ToggleActiveParticleEvent,
    },
    spawn::{
        spawn_blood_effect_system, spawn_explosion_system,
        spawn_projectile_despawn_effect_system, spawn_projectile_hit_effect_system,
        spawn_projectile_trail_system, spawn_spawner_effect_system,
    },
    systems::{
        activate_particle_effect_system, blood_effect_management_system,
        particle_lifetime_management_system, particle_position_tracking_system,
        toggle_particle_effect_system,
    },
};

pub struct ThetawaveParticlesPlugin;

impl Plugin for ThetawaveParticlesPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_message::<SpawnerParticleEffectSpawnedEvent>();
        app.add_message::<ActivateParticleEvent>();
        app.add_message::<ToggleActiveParticleEvent>();
        app.add_message::<SpawnBloodEffectEvent>();
        app.add_message::<SpawnProjectileTrailEffectEvent>();
        app.add_message::<SpawnExplosionEffectEvent>();
        app.add_message::<SpawnProjectileDespawnEffectEvent>();
        app.add_message::<SpawnProjectileHitEffectEvent>();
        app.add_message::<SpawnSpawnerEffectEvent>();

        app.add_plugins(EnokiPlugin);

        app.add_systems(
            Update,
            (
                particle_position_tracking_system,
                activate_particle_effect_system,
                toggle_particle_effect_system,
                spawn_blood_effect_system,
                spawn_projectile_trail_system,
                spawn_explosion_system,
                spawn_projectile_despawn_effect_system,
                spawn_projectile_hit_effect_system,
                spawn_spawner_effect_system,
            )
                .run_if(in_state(AppState::Game).and(in_state(GameState::Playing))),
        );

        app.add_systems(
            Update,
            (
                particle_lifetime_management_system,
                blood_effect_management_system,
            )
                .run_if(in_state(AppState::Game).and(in_state(GameState::Playing))),
        );
    }
}
