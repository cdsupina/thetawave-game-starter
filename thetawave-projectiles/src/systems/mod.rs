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
        DespawnAfterAnimationComponent, ProjectileRangeComponent, SpawnProjectileEffectEvent,
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
