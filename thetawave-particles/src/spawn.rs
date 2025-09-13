use bevy::{
    ecs::{
        entity::Entity,
        error::{BevyError, Result},
        event::EventReader,
        name::Name,
        system::{Commands, Res},
    },
    transform::components::Transform,
};
use bevy_enoki::{ParticleEffectHandle, ParticleSpawner, prelude::ParticleSpawnerState};
use thetawave_assets::{AssetResolver, ExtendedGameAssets, GameAssets, ParticleMaterials};
use thetawave_core::Faction;
use thetawave_core::{AppState, Cleanup};

use crate::data::SpawnParticleEffectEvent;

pub fn spawn_particle_effect(
    cmds: &mut Commands,
    parent_entity: Option<Entity>,
    effect_type: &str,
    faction: &Faction,
    transform: &Transform,
    extended_assets: &ExtendedGameAssets,
    assets: &GameAssets,
    materials: &ParticleMaterials,
    is_active: bool,
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
        ))
        .id();

    if let Some(parent) = parent_entity {
        cmds.entity(parent).add_child(particle_entity);
    }

    Ok(particle_entity)
}

pub(crate) fn spawn_particle_effect_system(
    mut cmds: Commands,
    extended_assets: Res<ExtendedGameAssets>,
    assets: Res<GameAssets>,
    materials: Res<ParticleMaterials>,
    mut spawn_particle_effect_event_reader: EventReader<SpawnParticleEffectEvent>,
) -> Result {
    for event in spawn_particle_effect_event_reader.read() {
        let _particle_entity = spawn_particle_effect(
            &mut cmds,
            event.parent_entity,
            &event.effect_type,
            &event.faction,
            &event.transform,
            &extended_assets,
            &assets,
            &materials,
            event.is_active,
        )?;
    }

    Ok(())
}
