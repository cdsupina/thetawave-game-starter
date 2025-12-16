use bevy::{
    asset::Assets,
    color::Color,
    ecs::{
        entity::Entity,
        error::{BevyError, Result},
        message::{MessageReader, MessageWriter},
        name::Name,
        system::{Commands, Res, ResMut},
    },
    math::{Vec2, Vec3},
    camera::primitives::Aabb,
    transform::components::Transform,
};
use bevy_enoki::{
    EmissionShape, NoAutoAabb, Particle2dEffect, ParticleEffectHandle, ParticleSpawner,
    prelude::{ColorParticle2dMaterial, OneShot, ParticleSpawnerState},
};
use thetawave_assets::{AssetResolver, ExtendedGameAssets, GameAssets, ParticleMaterials};
use thetawave_core::{AppState, Cleanup};
#[cfg(feature = "debug")]
use thetawave_core::LoggingSettings;

use crate::{
    SpawnBloodEffectEvent, SpawnProjectileTrailEffectEvent,
    data::{
        BloodEffectManager, ParticleLifeTimer, SpawnExplosionEffectEvent,
        SpawnProjectileDespawnEffectEvent, SpawnProjectileHitEffectEvent,
        SpawnSpawnerEffectEvent, SpawnerParticleEffectSpawnedEvent,
    },
};

const MANUAL_AABB_EXTENTS: f32 = 500.0;
const MAX_LIFETIME_FALLBACK: f32 = 3.0;

/// Helper function that spawns a particle entity with common components
fn spawn_particle_entity(
    cmds: &mut Commands,
    particle_effect_handle: ParticleEffectHandle,
    transform: Transform,
    color: &Color,
    materials: &ParticleMaterials,
    color_materials: &mut Assets<ColorParticle2dMaterial>,
    is_active: bool,
    is_one_shot: bool,
) -> Entity {
    let mut entity_cmds = cmds.spawn((
        Name::new("Particle Effect"),
        Cleanup::<AppState> {
            states: vec![AppState::Game],
        },
        transform,
        ParticleSpawner(materials.get_material_for_color(color, color_materials)),
        ParticleSpawnerState {
            active: is_active,
            ..Default::default()
        },
        particle_effect_handle,
        // AABB is used for determining whether something should be rendered.
        // Manual setting so that particles that are in view of the camera, but their spawner is out of view are still rendered.
        NoAutoAabb,
        Aabb::from_min_max(
            Vec2::splat(-MANUAL_AABB_EXTENTS).extend(0.0),
            Vec2::splat(MANUAL_AABB_EXTENTS).extend(0.0),
        ),
    ));

    if is_one_shot {
        entity_cmds.insert(OneShot::Despawn);
    }

    entity_cmds.id()
}

/// Dedicated function for spawning blood effects with simplified parameters
pub fn spawn_blood_effect(
    cmds: &mut Commands,
    parent_entity: Option<Entity>,
    amount: f32,
    color: &Color,
    position: Vec2,
    direction: Vec2,
    extended_assets: &ExtendedGameAssets,
    assets: &GameAssets,
    materials: &ParticleMaterials,
    particle_effects: &mut Assets<Particle2dEffect>,
    color_materials: &mut Assets<ColorParticle2dMaterial>,
    #[cfg(feature = "debug")] logging_settings: &LoggingSettings,
) -> Result<Entity, BevyError> {
    // Get the base blood effect handle and modify it for direction
    let base_handle = AssetResolver::get_game_particle_effect("blood", extended_assets, assets)?;
    let particle_effect_handle = if let Some(base_effect) = particle_effects.get(&base_handle) {
        let mut modified_effect = base_effect.clone();

        // Apply direction override
        if let Some(ref mut current_direction) = modified_effect.direction {
            current_direction.0 = direction; // Update base value only
        } else {
            thetawave_core::log_if!(logging_settings, particles, warn,
                "Trying to set direction on blood effect without existing direction: {:?}",
                direction
            );
        }

        // Add modified effect to assets and return new handle
        particle_effects.add(modified_effect)
    } else {
        base_handle
    };

    // Create transform from Vec2 position
    let transform = Transform::from_translation(position.extend(0.0));

    // Spawn the particle entity with blood-specific defaults
    let particle_entity = spawn_particle_entity(
        cmds,
        ParticleEffectHandle(particle_effect_handle.clone()),
        transform,
        color,
        materials,
        color_materials,
        true,  // Blood effects start active
        false, // Blood effects are not one-shot
    );

    // Add ParticleLifeTimer for position tracking
    cmds.entity(particle_entity)
        .insert(ParticleLifeTimer::new_with_offset(
            MAX_LIFETIME_FALLBACK,
            parent_entity,
            transform.translation,
        ));

    // Add blood effect manager with the specified amount
    cmds.entity(particle_entity)
        .insert(BloodEffectManager::new(amount));

    Ok(particle_entity)
}

/// Dedicated function for spawning projectile trail effects
pub fn spawn_projectile_trail(
    cmds: &mut Commands,
    parent_entity: Entity,
    color: &Color,
    scale: f32,
    extended_assets: &ExtendedGameAssets,
    assets: &GameAssets,
    materials: &ParticleMaterials,
    particle_effects: &mut Assets<Particle2dEffect>,
    color_materials: &mut Assets<ColorParticle2dMaterial>,
    #[cfg(feature = "debug")] logging_settings: &LoggingSettings,
) -> Result<Entity, BevyError> {
    // Get the projectile trail effect handle and apply scaling
    let particle_effect_handle = {
        let base_handle =
            AssetResolver::get_game_particle_effect("projectile_trail", extended_assets, assets)?;
        if let Some(base_effect) = particle_effects.get(&base_handle) {
            let mut modified_effect = base_effect.clone();

            // Apply scaling
            // Scale emission shape
            match &mut modified_effect.emission_shape {
                EmissionShape::Circle(radius) => *radius *= scale,
                EmissionShape::Point => {} // Point doesn't need scaling
            }

            modified_effect.spawn_amount *= scale as u32;

            // Add modified effect to assets and return new handle
            particle_effects.add(modified_effect)
        } else {
            base_handle
        }
    };

    // Use default transform (origin) - position tracking will handle positioning
    let transform = Transform::default();

    // Spawn the particle entity with projectile trail specific defaults
    let particle_entity = spawn_particle_entity(
        cmds,
        ParticleEffectHandle(particle_effect_handle.clone()),
        transform,
        color,
        materials,
        color_materials,
        true,  // Projectile trails start active
        false, // Projectile trails are not one-shot
    );

    // Calculate lifetime from the particle effect
    let max_lifetime = if let Some(particle_effect) = particle_effects.get(&particle_effect_handle)
    {
        let base_lifetime = particle_effect.lifetime.0;
        let randomness = particle_effect.lifetime.1;
        base_lifetime + (base_lifetime * randomness)
    } else {
        thetawave_core::log_if!(logging_settings, particles, warn,
            "Projectile trail effect not loaded yet, using fallback lifetime: {}",
            MAX_LIFETIME_FALLBACK
        );
        MAX_LIFETIME_FALLBACK
    };

    // Add ParticleLifeTimer for position tracking (following the projectile)
    // Use Vec3::ZERO offset so trail follows projectile exactly
    cmds.entity(particle_entity)
        .insert(ParticleLifeTimer::new_with_offset(
            max_lifetime,
            Some(parent_entity),
            Vec3::ZERO,
        ));

    Ok(particle_entity)
}

/// Dedicated function for spawning explosion effects
pub fn spawn_explosion(
    cmds: &mut Commands,
    color: &Color,
    position: Vec2,
    scale: f32,
    extended_assets: &ExtendedGameAssets,
    assets: &GameAssets,
    materials: &ParticleMaterials,
    particle_effects: &mut Assets<Particle2dEffect>,
    color_materials: &mut Assets<ColorParticle2dMaterial>,
) -> Result<Entity, BevyError> {
    // Get the explosion effect handle and apply scaling
    let particle_effect_handle = {
        let base_handle =
            AssetResolver::get_game_particle_effect("explosion", extended_assets, assets)?;
        if let Some(base_effect) = particle_effects.get(&base_handle) {
            let mut modified_effect = base_effect.clone();

            // Apply scaling
            // Scale emission shape
            match &mut modified_effect.emission_shape {
                EmissionShape::Circle(radius) => *radius *= scale,
                EmissionShape::Point => {} // Point doesn't need scaling
            }

            modified_effect.spawn_amount *= scale as u32;

            // Add modified effect to assets and return new handle
            particle_effects.add(modified_effect)
        } else {
            base_handle
        }
    };

    // Create transform from Vec2 position
    let transform = Transform::from_translation(position.extend(0.0));

    // Spawn the particle entity with explosion-specific defaults
    let particle_entity = spawn_particle_entity(
        cmds,
        ParticleEffectHandle(particle_effect_handle.clone()),
        transform,
        color,
        materials,
        color_materials,
        true, // Explosions start active
        true, // Explosions are one-shot effects
    );

    Ok(particle_entity)
}

/// Dedicated function for spawning projectile despawn effects
pub fn spawn_projectile_despawn_effect(
    cmds: &mut Commands,
    effect_type: &str,
    color: &Color,
    position: Vec2,
    scale: f32,
    extended_assets: &ExtendedGameAssets,
    assets: &GameAssets,
    materials: &ParticleMaterials,
    particle_effects: &mut Assets<Particle2dEffect>,
    color_materials: &mut Assets<ColorParticle2dMaterial>,
) -> Result<Entity, BevyError> {
    // Get the despawn effect handle and apply scaling
    let particle_effect_handle = {
        let base_handle =
            AssetResolver::get_game_particle_effect(effect_type, extended_assets, assets)?;
        if let Some(base_effect) = particle_effects.get(&base_handle) {
            let mut modified_effect = base_effect.clone();

            // Apply scaling
            // Scale emission shape
            match &mut modified_effect.emission_shape {
                EmissionShape::Circle(radius) => *radius *= scale,
                EmissionShape::Point => {} // Point doesn't need scaling
            }

            modified_effect.spawn_amount *= scale as u32;

            // Add modified effect to assets and return new handle
            particle_effects.add(modified_effect)
        } else {
            base_handle
        }
    };

    // Create transform from Vec2 position
    let transform = Transform::from_translation(position.extend(0.0));

    // Spawn the particle entity with despawn-specific defaults
    let particle_entity = spawn_particle_entity(
        cmds,
        ParticleEffectHandle(particle_effect_handle.clone()),
        transform,
        color,
        materials,
        color_materials,
        true, // Despawn effects start active
        true, // Despawn effects are one-shot effects
    );

    Ok(particle_entity)
}

/// Dedicated function for spawning projectile hit effects
pub fn spawn_projectile_hit_effect(
    cmds: &mut Commands,
    effect_type: &str,
    color: &Color,
    position: Vec2,
    scale: f32,
    extended_assets: &ExtendedGameAssets,
    assets: &GameAssets,
    materials: &ParticleMaterials,
    particle_effects: &mut Assets<Particle2dEffect>,
    color_materials: &mut Assets<ColorParticle2dMaterial>,
) -> Result<Entity, BevyError> {
    // Get the hit effect handle and apply scaling
    let particle_effect_handle = {
        let base_handle =
            AssetResolver::get_game_particle_effect(effect_type, extended_assets, assets)?;
        if let Some(base_effect) = particle_effects.get(&base_handle) {
            let mut modified_effect = base_effect.clone();

            // Apply scaling
            // Scale emission shape
            match &mut modified_effect.emission_shape {
                EmissionShape::Circle(radius) => *radius *= scale,
                EmissionShape::Point => {} // Point doesn't need scaling
            }

            modified_effect.spawn_amount *= scale as u32;

            // Add modified effect to assets and return new handle
            particle_effects.add(modified_effect)
        } else {
            base_handle
        }
    };

    // Create transform from Vec2 position
    let transform = Transform::from_translation(position.extend(0.0));

    // Spawn the particle entity with hit-specific defaults
    let particle_entity = spawn_particle_entity(
        cmds,
        ParticleEffectHandle(particle_effect_handle.clone()),
        transform,
        color,
        materials,
        color_materials,
        true, // Hit effects start active
        true, // Hit effects are one-shot effects
    );

    Ok(particle_entity)
}

/// Dedicated function for spawning projectile spawner effects on mobs
pub fn spawn_spawner_effect(
    cmds: &mut Commands,
    parent_entity: Entity,
    effect_type: &str,
    color: &Color,
    position: Vec2,
    key: &str,
    extended_assets: &ExtendedGameAssets,
    assets: &GameAssets,
    materials: &ParticleMaterials,
    _particle_effects: &mut Assets<Particle2dEffect>,
    color_materials: &mut Assets<ColorParticle2dMaterial>,
    spawner_event_writer: &mut MessageWriter<SpawnerParticleEffectSpawnedEvent>,
) -> Result<Entity, BevyError> {
    // Get the spawner effect handle (no scaling needed)
    let particle_effect_handle =
        AssetResolver::get_game_particle_effect(effect_type, extended_assets, assets)?;

    // Create transform from Vec2 position
    let transform = Transform::from_translation(position.extend(0.0));

    // Spawn the particle entity with spawner-specific defaults
    let particle_entity = spawn_particle_entity(
        cmds,
        ParticleEffectHandle(particle_effect_handle.clone()),
        transform,
        color,
        materials,
        color_materials,
        false, // Spawner effects start inactive
        false, // Spawner effects are not one-shot
    );

    // Add particle as child of parent entity (uses parent-child relationship, not position tracking)
    cmds.entity(parent_entity).add_child(particle_entity);

    // Emit event to associate this particle effect with the spawner
    spawner_event_writer.write(SpawnerParticleEffectSpawnedEvent {
        key: key.to_string(),
        effect_entity: particle_entity,
        parent_entity,
    });

    Ok(particle_entity)
}

pub(crate) fn spawn_blood_effect_system(
    mut cmds: Commands,
    extended_assets: Res<ExtendedGameAssets>,
    assets: Res<GameAssets>,
    materials: Res<ParticleMaterials>,
    mut particle_effects: ResMut<Assets<Particle2dEffect>>,
    mut color_materials: ResMut<Assets<ColorParticle2dMaterial>>,
    #[cfg(feature = "debug")] logging_settings: Res<LoggingSettings>,
    mut blood_effect_event_reader: MessageReader<SpawnBloodEffectEvent>,
) -> Result {
    for event in blood_effect_event_reader.read() {
        let _particle_entity = spawn_blood_effect(
            &mut cmds,
            Some(event.parent_entity),
            event.amount,
            &event.color,
            event.position,
            event.direction,
            &extended_assets,
            &assets,
            &materials,
            &mut particle_effects,
            &mut color_materials,
            #[cfg(feature = "debug")]
            &logging_settings,
        )?;
    }

    Ok(())
}

pub(crate) fn spawn_projectile_trail_system(
    mut cmds: Commands,
    extended_assets: Res<ExtendedGameAssets>,
    assets: Res<GameAssets>,
    materials: Res<ParticleMaterials>,
    mut particle_effects: ResMut<Assets<Particle2dEffect>>,
    mut color_materials: ResMut<Assets<ColorParticle2dMaterial>>,
    #[cfg(feature = "debug")] logging_settings: Res<LoggingSettings>,
    mut projectile_trail_event_reader: MessageReader<SpawnProjectileTrailEffectEvent>,
) -> Result {
    for event in projectile_trail_event_reader.read() {
        let _particle_entity = spawn_projectile_trail(
            &mut cmds,
            event.parent_entity,
            &event.color,
            event.scale,
            &extended_assets,
            &assets,
            &materials,
            &mut particle_effects,
            &mut color_materials,
            #[cfg(feature = "debug")]
            &logging_settings,
        )?;
    }

    Ok(())
}

pub(crate) fn spawn_explosion_system(
    mut cmds: Commands,
    extended_assets: Res<ExtendedGameAssets>,
    assets: Res<GameAssets>,
    materials: Res<ParticleMaterials>,
    mut particle_effects: ResMut<Assets<Particle2dEffect>>,
    mut color_materials: ResMut<Assets<ColorParticle2dMaterial>>,
    mut explosion_event_reader: MessageReader<SpawnExplosionEffectEvent>,
) -> Result {
    for event in explosion_event_reader.read() {
        let _particle_entity = spawn_explosion(
            &mut cmds,
            &event.color,
            event.position,
            event.scale,
            &extended_assets,
            &assets,
            &materials,
            &mut particle_effects,
            &mut color_materials,
        )?;
    }

    Ok(())
}

pub(crate) fn spawn_projectile_despawn_effect_system(
    mut cmds: Commands,
    extended_assets: Res<ExtendedGameAssets>,
    assets: Res<GameAssets>,
    materials: Res<ParticleMaterials>,
    mut particle_effects: ResMut<Assets<Particle2dEffect>>,
    mut color_materials: ResMut<Assets<ColorParticle2dMaterial>>,
    mut despawn_event_reader: MessageReader<SpawnProjectileDespawnEffectEvent>,
) -> Result {
    for event in despawn_event_reader.read() {
        let _particle_entity = spawn_projectile_despawn_effect(
            &mut cmds,
            &event.effect_type,
            &event.color,
            event.position,
            event.scale,
            &extended_assets,
            &assets,
            &materials,
            &mut particle_effects,
            &mut color_materials,
        )?;
    }

    Ok(())
}

pub(crate) fn spawn_projectile_hit_effect_system(
    mut cmds: Commands,
    extended_assets: Res<ExtendedGameAssets>,
    assets: Res<GameAssets>,
    materials: Res<ParticleMaterials>,
    mut particle_effects: ResMut<Assets<Particle2dEffect>>,
    mut color_materials: ResMut<Assets<ColorParticle2dMaterial>>,
    mut hit_event_reader: MessageReader<SpawnProjectileHitEffectEvent>,
) -> Result {
    for event in hit_event_reader.read() {
        let _particle_entity = spawn_projectile_hit_effect(
            &mut cmds,
            &event.effect_type,
            &event.color,
            event.position,
            event.scale,
            &extended_assets,
            &assets,
            &materials,
            &mut particle_effects,
            &mut color_materials,
        )?;
    }

    Ok(())
}

pub(crate) fn spawn_spawner_effect_system(
    mut cmds: Commands,
    extended_assets: Res<ExtendedGameAssets>,
    assets: Res<GameAssets>,
    materials: Res<ParticleMaterials>,
    mut particle_effects: ResMut<Assets<Particle2dEffect>>,
    mut color_materials: ResMut<Assets<ColorParticle2dMaterial>>,
    mut spawner_event_reader: MessageReader<SpawnSpawnerEffectEvent>,
    mut spawner_particle_event_writer: MessageWriter<SpawnerParticleEffectSpawnedEvent>,
) -> Result {
    for event in spawner_event_reader.read() {
        let _particle_entity = spawn_spawner_effect(
            &mut cmds,
            event.parent_entity,
            &event.effect_type,
            &event.color,
            event.position,
            &event.key,
            &extended_assets,
            &assets,
            &materials,
            &mut particle_effects,
            &mut color_materials,
            &mut spawner_particle_event_writer,
        )?;
    }

    Ok(())
}
