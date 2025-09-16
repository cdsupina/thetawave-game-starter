use bevy::{
    ecs::{
        entity::Entity,
        error::{BevyError, Result},
        event::{EventReader, EventWriter},
        name::Name,
        system::{Commands, Res},
    },
    math::Vec2,
    render::primitives::Aabb,
    transform::components::Transform,
};
use bevy_enoki::{
    NoAutoAabb, ParticleEffectHandle, ParticleSpawner, prelude::ParticleSpawnerState,
};
use thetawave_assets::{AssetResolver, ExtendedGameAssets, GameAssets, ParticleMaterials};
use thetawave_core::Faction;
use thetawave_core::{AppState, Cleanup};

use crate::data::{SpawnParticleEffectEvent, SpawnerParticleEffectSpawnedEvent};

const MANUAL_AABB_EXTENTS: f32 = 500.0;

pub fn spawn_particle_effect(
    cmds: &mut Commands,
    parent_entity: Option<Entity>,
    effect_type: &str,
    key: &Option<String>,
    faction: &Faction,
    transform: &Transform,
    extended_assets: &ExtendedGameAssets,
    assets: &GameAssets,
    materials: &ParticleMaterials,
    is_active: bool,
    particle_effect_spawned_event_writer: &mut EventWriter<SpawnerParticleEffectSpawnedEvent>,
) -> Result<Entity, BevyError> {
    let particle_effect_handle =
        AssetResolver::get_game_particle_effect(effect_type, extended_assets, assets)?;

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
                active: is_active, // Start inactive, will be activated by behavior system when needed
                ..Default::default()
            },
            ParticleEffectHandle(particle_effect_handle),
            // AABB is used for determining whether something should be rendered.
            // Manual setting  so that particles that are in view of the camera, but their spawner is out of view are still rendered.
            NoAutoAabb,
            Aabb::from_min_max(
                Vec2::splat(MANUAL_AABB_EXTENTS).extend(0.0),
                Vec2::splat(MANUAL_AABB_EXTENTS).extend(0.0),
            ),
        ))
        .id();

    if let Some(parent) = parent_entity {
        cmds.entity(parent).add_child(particle_entity);

        if let Some(key) = key {
            particle_effect_spawned_event_writer.write(SpawnerParticleEffectSpawnedEvent {
                key: key.clone(),
                effect_entity: particle_entity,
                parent_entity: parent,
            });
        }
    }

    Ok(particle_entity)
}

pub(crate) fn spawn_particle_effect_system(
    mut cmds: Commands,
    extended_assets: Res<ExtendedGameAssets>,
    assets: Res<GameAssets>,
    materials: Res<ParticleMaterials>,
    mut spawn_particle_effect_event_reader: EventReader<SpawnParticleEffectEvent>,
    mut particle_effect_spawned_event_writer: EventWriter<SpawnerParticleEffectSpawnedEvent>,
) -> Result {
    for event in spawn_particle_effect_event_reader.read() {
        let _particle_entity = spawn_particle_effect(
            &mut cmds,
            event.parent_entity,
            &event.effect_type,
            &event.key,
            &event.faction,
            &event.transform,
            &extended_assets,
            &assets,
            &materials,
            event.is_active,
            &mut particle_effect_spawned_event_writer,
        )?;
    }

    Ok(())
}
