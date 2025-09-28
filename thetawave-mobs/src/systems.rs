use avian2d::prelude::RevoluteJoint;
use bevy::{
    color::Color,
    ecs::{
        entity::Entity,
        event::{EventReader, EventWriter},
        query::With,
        system::{Commands, Query},
    },
    log::info,
    transform::components::Transform,
};

use crate::MobDeathEvent;
use thetawave_particles::{ActivateParticleEvent, ParticleLifeTimer};

const XHITARA_BLOOD_COLOR: Color = Color::srgba(0.376, 0.820, 0.737, 1.0);

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
    mut particle_effect_event_writer: EventWriter<thetawave_particles::SpawnParticleEffectEvent>,
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

        cmds.entity(event.mob_entity).despawn();
        if let Ok(mob_transform) = mob_q.get(event.mob_entity) {
            particle_effect_event_writer.write(thetawave_particles::SpawnParticleEffectEvent {
                parent_entity: None,
                effect_type: "explosion".to_string(),
                color: thetawave_core::Faction::Enemy.get_color(),
                transform: Transform::from_translation(mob_transform.translation),
                is_active: true,
                key: None,
                needs_position_tracking: false,
                is_one_shot: true,
                scale: None,
                direction: None,
            });
        }
    }
}

/// Detects when mobs with joints are destroyed and spawns blood effects at joint locations
pub(crate) fn detect_destroyed_joints(
    mut mob_death_event_reader: EventReader<MobDeathEvent>,
    mut particle_effect_event_writer: EventWriter<thetawave_particles::SpawnParticleEffectEvent>,
    joint_entities_q: Query<Entity, With<RevoluteJoint>>,
    all_joints_q: Query<&RevoluteJoint>,
    transform_q: Query<&Transform>,
) {
    for event in mob_death_event_reader.read() {
        info!(
            "🔍 Checking mob death event for Entity: {:?}",
            event.mob_entity
        );

        let mut is_jointed_mob = false;

        // Check if this entity is referenced in any existing joints
        for joint_entity in joint_entities_q.iter() {
            if let Ok(joint) = all_joints_q.get(joint_entity) {
                if joint.entity1 == event.mob_entity || joint.entity2 == event.mob_entity {
                    info!(
                        "🔗 JOINTED MOB DESTROYED! Entity: {:?} was connected via joint {:?}",
                        event.mob_entity, joint_entity
                    );

                    // Spawn marker on the remaining entity
                    let remaining_entity = if joint.entity1 == event.mob_entity {
                        joint.entity2
                    } else {
                        joint.entity1
                    };

                    if let Ok(remaining_transform) = transform_q.get(remaining_entity) {
                        // Calculate the marker position based on joint anchor
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
                        particle_effect_event_writer.write(
                            thetawave_particles::SpawnParticleEffectEvent {
                                parent_entity: Some(remaining_entity),
                                effect_type: "blood".to_string(),
                                color: XHITARA_BLOOD_COLOR,
                                transform: Transform::from_translation(anchor_pos.extend(0.0)),
                                is_active: true,
                                key: Some(format!("joint_blood_{}", joint_entity.index())),
                                needs_position_tracking: true,
                                is_one_shot: false,
                                scale: None,
                                direction: Some(spray_direction),
                            },
                        );
                    }

                    is_jointed_mob = true;
                    // Don't break - continue checking for more joints this mob might be part of
                }
            }
        }

        if !is_jointed_mob {
            info!(
                "📝 Mob {:?} was not part of any joint relationship",
                event.mob_entity
            );
        }
    }
}
