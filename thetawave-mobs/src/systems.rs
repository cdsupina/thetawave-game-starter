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
    prelude::Name,
    sprite::Sprite,
    transform::components::Transform,
};

use crate::{MobDeathEvent, attributes::JointsComponent};

/// Checks for mob death events, despawns the entity and spawns an explosion
pub(crate) fn mob_death_system(
    mut cmds: Commands,
    mut mob_death_event_reader: EventReader<MobDeathEvent>,
    mut particle_effect_event_writer: EventWriter<thetawave_particles::SpawnParticleEffectEvent>,
    mob_q: Query<&Transform>,
) {
    for event in mob_death_event_reader.read() {
        cmds.entity(event.mob_entity).despawn();
        if let Ok(mob_transform) = mob_q.get(event.mob_entity) {
            particle_effect_event_writer.write(thetawave_particles::SpawnParticleEffectEvent {
                parent_entity: None,
                effect_type: "explosion".to_string(),
                faction: thetawave_core::Faction::Enemy,
                transform: Transform::from_translation(mob_transform.translation),
                is_active: true,
                key: None,
                needs_position_tracking: false,
                is_one_shot: true,
                scale: None,
            });
        }
    }
}

/// Detects when mobs with joints are destroyed and spawns green markers and blood effects at joint locations
pub(crate) fn detect_destroyed_joints(
    mut cmds: Commands,
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

                        let marker_position = remaining_transform.translation
                            + (remaining_transform.rotation * anchor_pos.extend(0.0));

                        // Spawn a green debug marker as a child of the remaining entity
                        cmds.entity(remaining_entity).with_children(|parent| {
                            parent.spawn((
                                Transform::from_translation(anchor_pos.extend(0.0)),
                                Name::new(format!("JointMarker_{}", joint_entity.index())),
                                Sprite {
                                    color: Color::srgb(0.0, 1.0, 0.0), // Green color
                                    custom_size: Some(bevy::math::Vec2::new(6.0, 6.0)),
                                    ..Default::default()
                                },
                            ));
                        });

                        // Spawn blood particle effect at the joint location using world position
                        particle_effect_event_writer.write(
                            thetawave_particles::SpawnParticleEffectEvent {
                                parent_entity: Some(remaining_entity),
                                effect_type: "blood".to_string(),
                                faction: thetawave_core::Faction::Enemy,
                                transform: Transform::from_translation(anchor_pos.extend(0.0)),
                                is_active: true,
                                key: Some(format!("joint_blood_{}", joint_entity.index())),
                                needs_position_tracking: false,
                                is_one_shot: false,
                                scale: None,
                            },
                        );

                        info!(
                            "  📍 Spawned joint marker at {:?} on remaining entity {:?}",
                            marker_position, remaining_entity
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
