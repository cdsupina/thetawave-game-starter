use avian2d::prelude::CollisionStarted;
use bevy::{
    ecs::{
        entity::Entity,
        event::{EventReader, EventWriter},
        query::With,
        system::{Commands, Query, Res},
    },
    time::Time,
    transform::components::Transform,
};
use bevy_aseprite_ultra::prelude::AnimationEvents;
use thetawave_core::Faction;
use thetawave_particles::{ActivateParticleEvent, ParticleLifeTimer, SpawnParticleEffectEvent};

use crate::{
    ProjectileType,
    attributes::{DespawnAfterAnimationComponent, ProjectileRangeComponent},
};

fn get_despawn_particle_effect(projectile_type: &ProjectileType) -> &str {
    // keys are the file stem of the desired asset
    match projectile_type {
        ProjectileType::Bullet => "despawn_bullet",
        ProjectileType::Blast => "despawn_blast",
    }
}

fn get_hit_particle_effect(projectile_type: &ProjectileType) -> &str {
    // keys are the file stem of the desired asset
    match projectile_type {
        ProjectileType::Bullet => "hit_bullet",
        ProjectileType::Blast => "hit_blast",
    }
}

/// Helper function to deactivate particle spawners associated with a projectile entity
fn deactivate_projectile_particle_spawners(
    projectile_entity: Entity,
    particle_spawner_q: &Query<(Entity, &ParticleLifeTimer)>,
    activate_particle_event_writer: &mut EventWriter<ActivateParticleEvent>,
) {
    for (spawner_entity, life_timer) in particle_spawner_q.iter() {
        if life_timer.parent_entity == Some(projectile_entity) {
            activate_particle_event_writer.write(ActivateParticleEvent {
                entity: spawner_entity,
                active: false,
            });
        }
    }
}

/// Despawns projectiles after a set amount of time passes
pub(crate) fn timed_range_system(
    mut cmds: Commands,
    mut projectile_q: Query<(
        Entity,
        &ProjectileType,
        &Faction,
        &Transform,
        &mut ProjectileRangeComponent,
    )>,
    particle_spawner_q: Query<(Entity, &ParticleLifeTimer)>,
    time: Res<Time>,
    mut activate_particle_event_writer: EventWriter<ActivateParticleEvent>,
    mut spawn_particle_effect_event_writer: EventWriter<SpawnParticleEffectEvent>,
) {
    for (entity, projectile_type, faction, transform, mut range) in projectile_q.iter_mut() {
        if range.timer.tick(time.delta()).just_finished() {
            // Deactivate any particle spawners associated with this projectile
            deactivate_projectile_particle_spawners(
                entity,
                &particle_spawner_q,
                &mut activate_particle_event_writer,
            );

            spawn_particle_effect_event_writer.write(SpawnParticleEffectEvent {
                parent_entity: None,
                effect_type: get_despawn_particle_effect(projectile_type).to_string(),
                faction: faction.clone(),
                transform: *transform,
                is_active: true,
                key: None,
                needs_position_tracking: false,
                is_one_shot: true,
            });

            cmds.entity(entity).despawn();
        }
    }
}

pub(crate) fn projectile_hit_system(
    mut cmds: Commands,
    projectile_q: Query<(Entity, &ProjectileType, &Faction, &Transform)>,
    particle_spawner_q: Query<(Entity, &ParticleLifeTimer)>,
    mut activate_particle_event_writer: EventWriter<ActivateParticleEvent>,
    mut collision_start_event: EventReader<CollisionStarted>,
    mut spawn_particle_effect_event_writer: EventWriter<SpawnParticleEffectEvent>,
) {
    for event in collision_start_event.read() {
        // Get the two entities involved in the collision
        let entity1 = event.0;
        let entity2 = event.1;

        // Find which entity is the projectile
        let projectile_data = projectile_q
            .get(entity1)
            .or_else(|_| projectile_q.get(entity2));

        if let Ok((entity, projectile_type, faction, transform)) = projectile_data {
            // Deactivate any particle spawners associated with this projectile
            deactivate_projectile_particle_spawners(
                entity,
                &particle_spawner_q,
                &mut activate_particle_event_writer,
            );

            spawn_particle_effect_event_writer.write(SpawnParticleEffectEvent {
                parent_entity: None,
                effect_type: get_hit_particle_effect(projectile_type).to_string(),
                faction: faction.clone(),
                transform: *transform,
                is_active: true,
                key: None,
                needs_position_tracking: false,
                is_one_shot: true,
            });

            cmds.entity(entity).despawn();
        }
    }
}

/// Despawns entities with despawn after animation component
/// After one animation cycle
pub(crate) fn despawn_after_animation_system(
    mut cmds: Commands,
    mut animation_event_reader: EventReader<AnimationEvents>,
    despawn_q: Query<Entity, With<DespawnAfterAnimationComponent>>,
) {
    for event in animation_event_reader.read() {
        if let AnimationEvents::LoopCycleFinished(event_entity) = event
            && let Ok(entity) = despawn_q.get(*event_entity)
        {
            cmds.entity(entity).despawn();
        }
    }
}
