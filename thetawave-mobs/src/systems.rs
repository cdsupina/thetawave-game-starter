use avian2d::prelude::RevoluteJoint;
use bevy::{
    ecs::{
        entity::Entity,
        event::{EventReader, EventWriter},
        query::With,
        system::{Commands, Query},
    },
    transform::components::Transform,
};
use thetawave_core::{XHITARA_BLOOD_COLOR, with_bloom};

use crate::MobDeathEvent;
use thetawave_particles::{
    ActivateParticleEvent, ParticleLifeTimer, SpawnBloodEffectEvent, SpawnExplosionEffectEvent,
};

/// Helper function to deactivate particle spawners associated with a mob entity
fn deactivate_mob_particle_spawners(
    mob_entity: Entity,
    particle_spawner_q: &Query<(Entity, &ParticleLifeTimer)>,
    activate_particle_event_writer: &mut EventWriter<ActivateParticleEvent>,
) {
    for (spawner_entity, life_timer) in particle_spawner_q.iter() {
        if life_timer.parent_entity == Some(mob_entity) {
            activate_particle_event_writer.write(ActivateParticleEvent {
                entity: spawner_entity,
                active: false,
            });
        }
    }
}

/// Checks for mob death events, despawns the entity and spawns an explosion
pub(crate) fn mob_death_system(
    mut cmds: Commands,
    mut mob_death_event_reader: EventReader<MobDeathEvent>,
    mut explosion_event_writer: EventWriter<SpawnExplosionEffectEvent>,
    mut activate_particle_event_writer: EventWriter<ActivateParticleEvent>,
    particle_spawner_q: Query<(Entity, &ParticleLifeTimer)>,
    mob_q: Query<&Transform>,
) {
    for event in mob_death_event_reader.read() {
        // Deactivate any particle spawners associated with this mob
        deactivate_mob_particle_spawners(
            event.mob_entity,
            &particle_spawner_q,
            &mut activate_particle_event_writer,
        );

        // Despawn the mob
        cmds.entity(event.mob_entity).despawn();

        // Spawn an explosion particle effect
        if let Ok(mob_transform) = mob_q.get(event.mob_entity) {
            explosion_event_writer.write(SpawnExplosionEffectEvent {
                color: with_bloom(XHITARA_BLOOD_COLOR, 2.0),
                position: mob_transform.translation.truncate(),
                scale: 1.0,
            });
        }
    }
}

/// Detects when mobs with joints are destroyed and spawns blood effects at joint locations
pub(crate) fn joint_bleed_system(
    mut mob_death_event_reader: EventReader<MobDeathEvent>,
    mut blood_effect_event_writer: EventWriter<SpawnBloodEffectEvent>,
    joint_entities_q: Query<Entity, With<RevoluteJoint>>,
    all_joints_q: Query<&RevoluteJoint>,
    transform_q: Query<&Transform>,
) {
    for event in mob_death_event_reader.read() {
        // Check if this entity is referenced in any existing joints
        for joint_entity in joint_entities_q.iter() {
            if let Ok(joint) = all_joints_q.get(joint_entity)
                && (joint.entity1 == event.mob_entity || joint.entity2 == event.mob_entity)
            {
                // Spawn blood on the remaining entity
                let remaining_entity = if joint.entity1 == event.mob_entity {
                    joint.entity2
                } else {
                    joint.entity1
                };

                if let Ok(remaining_transform) = transform_q.get(remaining_entity) {
                    // Calculate the blood position based on joint anchor
                    let anchor_pos = if joint.entity1 == event.mob_entity {
                        joint.local_anchor2
                    } else {
                        joint.local_anchor1
                    };

                    // Calculate direction from joint to mob center for particle spray direction
                    // Joint world position = mob center + (mob rotation * anchor offset)
                    let joint_world_pos = remaining_transform.translation.truncate()
                        + remaining_transform
                            .rotation
                            .mul_vec3(anchor_pos.extend(0.0))
                            .truncate();
                    let direction_to_center =
                        remaining_transform.translation.truncate() - joint_world_pos;
                    let spray_direction = -direction_to_center.normalize(); // Opposite direction (away from center)

                    // Spawn blood particle effect at the joint location
                    // Set parent to remaining entity and use local anchor position for offset tracking
                    // Rotate the effect to spray away from the mob center
                    blood_effect_event_writer.write(SpawnBloodEffectEvent {
                        amount: 0.2,
                        color: XHITARA_BLOOD_COLOR,
                        parent_entity: remaining_entity,
                        position: anchor_pos,
                        direction: spray_direction,
                    });
                }
            }
        }
    }
}
