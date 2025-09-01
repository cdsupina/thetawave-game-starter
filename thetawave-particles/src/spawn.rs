use bevy::{
    asset::Handle,
    ecs::{entity::Entity, name::Name, system::Commands},
    transform::components::Transform,
};
use bevy_enoki::{
    Particle2dEffect, ParticleEffectHandle, ParticleSpawner, prelude::ParticleSpawnerState,
};
use thetawave_assets::{AssetResolver, ExtendedGameAssets, GameAssets, ParticleMaterials};
use thetawave_core::Faction;
use thetawave_states::{AppState, Cleanup};

use crate::ParticleEffectType;

/// Get the particle effect handle from a given ParticleEffectType using asset resolver
fn get_particle_effect(
    effect_type: &ParticleEffectType,
    extended_game_assets: &ExtendedGameAssets,
    game_assets: &GameAssets,
) -> Handle<Particle2dEffect> {
    // keys are the file stem of the desired asset
    let key = match effect_type {
        ParticleEffectType::SpawnBlast => "spawn_blast",
        ParticleEffectType::SpawnBullet => "spawn_bullet",
    };

    AssetResolver::get_particle_effect(key, extended_game_assets, game_assets)
        .unwrap_or_else(|| panic!("Missing particle effect asset for type: {:?}", effect_type))
}

pub fn spawn_particle_effect(
    cmds: &mut Commands,
    parent_entity: Option<Entity>,
    effect_type: &ParticleEffectType,
    faction: &Faction,
    transform: &Transform,
    extended_assets: &ExtendedGameAssets,
    assets: &GameAssets,
    materials: &ParticleMaterials,
) -> Entity {
    let particle_entity = cmds
        .spawn((
            Name::new("Particle Effect"),
            faction.clone(),
            Cleanup::<AppState> {
                states: vec![AppState::Game],
            },
            *transform,
            ParticleSpawner(materials.get_material_for_faction(faction)),
            ParticleSpawnerState {
                active: false, // Start inactive, will be activated by behavior system when needed
                ..Default::default()
            },
            ParticleEffectHandle(get_particle_effect(effect_type, extended_assets, assets)),
        ))
        .id();

    if let Some(parent) = parent_entity {
        cmds.entity(parent).add_child(particle_entity);
    }

    particle_entity
}
