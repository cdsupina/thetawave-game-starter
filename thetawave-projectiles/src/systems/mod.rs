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

use crate::{
    ProjectileType,
    attributes::{
        DespawnAfterAnimationComponent, ProjectileEffectType, ProjectileRangeComponent,
        SpawnProjectileEffectEvent,
    },
};

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
    time: Res<Time>,
    mut spawn_effect_event_writer: EventWriter<SpawnProjectileEffectEvent>,
) {
    for (entity, projectile_type, faction, transform, mut range) in projectile_q.iter_mut() {
        if range.timer.tick(time.delta()).just_finished() {
            // Spawn the despawn effect
            spawn_effect_event_writer.write(SpawnProjectileEffectEvent {
                projectile_type: projectile_type.clone(),
                effect_type: ProjectileEffectType::Despawn,
                faction: faction.clone(),
                transform: *transform,
            });

            cmds.entity(entity).despawn();
        }
    }
}

pub(crate) fn projectile_hit_system(
    mut cmds: Commands,
    projectile_q: Query<(Entity, &ProjectileType, &Faction, &Transform)>,
    mut spawn_effect_event_writer: EventWriter<SpawnProjectileEffectEvent>,
    mut collision_start_event: EventReader<CollisionStarted>,
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
            // Spawn the hit effect
            spawn_effect_event_writer.write(SpawnProjectileEffectEvent {
                projectile_type: projectile_type.clone(),
                effect_type: ProjectileEffectType::Hit,
                faction: faction.clone(),
                transform: *transform,
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
