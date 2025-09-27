use bevy::{
    asset::Assets,
    ecs::{
        entity::Entity,
        error::{BevyError, Result},
        event::{EventReader, EventWriter},
        name::Name,
        system::{Commands, Res, ResMut},
    },
    log::warn,
    math::Vec2,
    render::primitives::Aabb,
    transform::components::Transform,
};
use bevy_enoki::{
    EmissionShape, NoAutoAabb, Particle2dEffect, ParticleEffectHandle, ParticleSpawner,
    prelude::{OneShot, ParticleSpawnerState},
};
use thetawave_assets::{AssetResolver, ExtendedGameAssets, GameAssets, ParticleMaterials};
use thetawave_core::Faction;
use thetawave_core::{AppState, Cleanup};

use crate::data::{ParticleLifeTimer, SpawnParticleEffectEvent, SpawnerParticleEffectSpawnedEvent};

const MANUAL_AABB_EXTENTS: f32 = 500.0;
const MAX_LIFETIME_FALLBACK: f32 = 5.0;

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
    particle_effects: &mut Assets<Particle2dEffect>,
    is_active: bool,
    is_one_shot: bool,
    needs_position_tracking: bool,
    scale: Option<f32>,
    particle_effect_spawned_event_writer: &mut EventWriter<SpawnerParticleEffectSpawnedEvent>,
) -> Result<Entity, BevyError> {
    let particle_effect_handle = if let Some(scale_value) = scale {
        let base_handle =
            AssetResolver::get_game_particle_effect(effect_type, extended_assets, assets)?;
        if let Some(base_effect) = particle_effects.get(&base_handle) {
            let mut scaled_effect = base_effect.clone();

            // Scale emission shape
            match &mut scaled_effect.emission_shape {
                EmissionShape::Circle(radius) => *radius *= scale_value,
                EmissionShape::Point => {} // Point doesn't need scaling
            }

            // Scale the scale property if present
            if let Some(ref mut scale_rval) = scaled_effect.scale {
                scale_rval.0 *= scale_value; // Scale the base scale value
            }

            if let Some(ref mut scale_curve) = scaled_effect.scale_curve
                && let Some(first_point) = scale_curve.points.first_mut()
            {
                first_point.0 *= scale_value;
            }

            scaled_effect.spawn_amount *= scale_value as u32;

            // Add scaled effect to assets and return new handle
            particle_effects.add(scaled_effect)
        } else {
            base_handle
        }
    } else {
        AssetResolver::get_game_particle_effect(effect_type, extended_assets, assets)?
    };

    let mut entity_cmds = cmds.spawn((
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
        ParticleEffectHandle(particle_effect_handle.clone()),
        // AABB is used for determining whether something should be rendered.
        // Manual setting  so that particles that are in view of the camera, but their spawner is out of view are still rendered.
        NoAutoAabb,
        Aabb::from_min_max(
            Vec2::splat(-MANUAL_AABB_EXTENTS).extend(0.0),
            Vec2::splat(MANUAL_AABB_EXTENTS).extend(0.0),
        ),
    ));

    if is_one_shot {
        entity_cmds.insert(OneShot::Despawn);
    }

    let particle_entity = entity_cmds.id();

    // Only add ParticleLifeTimer for effects that need position tracking (projectile trails)
    if needs_position_tracking {
        let max_lifetime = if let Some(particle_effect) =
            particle_effects.get(&particle_effect_handle)
        {
            // Calculate max possible lifetime: base_value + (base_value * randomness)
            let base_lifetime = particle_effect.lifetime.0;
            let randomness = particle_effect.lifetime.1;
            base_lifetime + (base_lifetime * randomness)
        } else {
            // Fallback if effect not loaded yet
            warn!(
                "Particle effect was not yet loaded, so no lifetime was found. Falling back to {}.",
                MAX_LIFETIME_FALLBACK
            );
            MAX_LIFETIME_FALLBACK
        };

        entity_cmds.insert(ParticleLifeTimer::new_with_offset(max_lifetime, parent_entity, transform.translation));
    } else {
        // For spawner effects (spawn_bullet, spawn_blast), maintain parent-child relationship
        // For projectile trails, keep them independent for lifetime management
        if let Some(parent) = parent_entity {
            cmds.entity(parent).add_child(particle_entity);
        }
    }

    // Send spawned event if key is provided (for associating with spawners)
    if let (Some(key), Some(parent)) = (key, parent_entity) {
        particle_effect_spawned_event_writer.write(SpawnerParticleEffectSpawnedEvent {
            key: key.clone(),
            effect_entity: particle_entity,
            parent_entity: parent,
        });
    }

    Ok(particle_entity)
}

pub(crate) fn spawn_particle_effect_system(
    mut cmds: Commands,
    extended_assets: Res<ExtendedGameAssets>,
    assets: Res<GameAssets>,
    materials: Res<ParticleMaterials>,
    mut particle_effects: ResMut<Assets<Particle2dEffect>>,
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
            &mut particle_effects,
            event.is_active,
            event.is_one_shot,
            event.needs_position_tracking,
            event.scale,
            &mut particle_effect_spawned_event_writer,
        )?;
    }

    Ok(())
}
