use bevy::{
    asset::Handle,
    ecs::{entity::Entity, name::Name, system::Commands},
    transform::components::Transform,
};
use bevy_enoki::{
    Particle2dEffect, ParticleEffectHandle, ParticleSpawner, prelude::ParticleSpawnerState,
};
use thetawave_assets::GameAssets;
use thetawave_states::{AppState, Cleanup};

use crate::ParticleEffectType;

trait GameAssetsExt {
    fn get_particle_effect(&self, effect_type: &ParticleEffectType) -> Handle<Particle2dEffect>;
}

impl GameAssetsExt for GameAssets {
    fn get_particle_effect(&self, effect_type: &ParticleEffectType) -> Handle<Particle2dEffect> {
        match effect_type {
            ParticleEffectType::SpawnBlast => self.spawn_blast_particle_effect.clone(),
        }
    }
}

pub fn spawn_particle_effect(
    cmds: &mut Commands,
    parent_entity: Option<Entity>,
    effect_type: &ParticleEffectType,
    transform: &Transform,
    assets: &GameAssets,
) -> Entity {
    let particle_entity = cmds
        .spawn((
            Name::new("Particle Effect"),
            Cleanup::<AppState> {
                states: vec![AppState::Game],
            },
            *transform,
            ParticleSpawner::default(),
            ParticleSpawnerState {
                active: false,
                ..Default::default()
            },
            ParticleEffectHandle(assets.get_particle_effect(effect_type)),
        ))
        .id();

    if let Some(parent) = parent_entity {
        cmds.entity(parent).add_child(particle_entity);
    }

    particle_entity
}
