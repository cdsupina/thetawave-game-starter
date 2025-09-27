use avian2d::prelude::RevoluteJoint;
use bevy::{
    ecs::{
        event::{EventReader, EventWriter},
        system::{Commands, Query},
        query::With,
        entity::Entity,
    },
    transform::components::Transform,
    log::info,
    prelude::{Name},
    sprite::Sprite,
    color::Color,
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

/// Detects when mobs with joints are destroyed and logs the information
pub(crate) fn detect_destroyed_joints(
    mut cmds: Commands,
    mut mob_death_event_reader: EventReader<MobDeathEvent>,
    joints_q: Query<&JointsComponent>,
    joint_entities_q: Query<Entity, With<RevoluteJoint>>,
    all_joints_q: Query<&RevoluteJoint>,
    transform_q: Query<&Transform>,
) {
    for event in mob_death_event_reader.read() {
        info!("🔍 Checking mob death event for Entity: {:?}", event.mob_entity);

        let mut is_jointed_mob = false;

        // Check if this entity is an anchor (has JointsComponent)
        if let Ok(joints_component) = joints_q.get(event.mob_entity) {
            if !joints_component.joints.is_empty() {
                info!(
                    "🔗 ANCHOR MOB DESTROYED! Entity: {:?}, Connected joints: {}",
                    event.mob_entity,
                    joints_component.joints.len()
                );

                for (joint_key, joint_entity) in &joints_component.joints {
                    info!(
                        "  ├─ Joint '{}' (Entity: {:?}) was connected to destroyed anchor",
                        joint_key,
                        joint_entity
                    );

                    // Get the actual joint data to find the connected entity
                    if let Ok(joint) = all_joints_q.get(*joint_entity) {
                        let connected_entity = if joint.entity1 == event.mob_entity {
                            joint.entity2
                        } else {
                            joint.entity1
                        };

                        if let Ok(connected_transform) = transform_q.get(connected_entity) {
                            // Calculate the marker position on the connected entity
                            let anchor_pos = if joint.entity1 == event.mob_entity {
                                joint.local_anchor2
                            } else {
                                joint.local_anchor1
                            };

                            let marker_position = connected_transform.translation +
                                (connected_transform.rotation * anchor_pos.extend(0.0));

                            // Spawn a debug marker entity at the joint location
                            cmds.spawn((
                                Transform::from_translation(marker_position),
                                Name::new(format!("JointMarker_Anchor_{}_{}", joint_key, joint_entity.index())),
                                Sprite {
                                    color: Color::srgb(1.0, 0.0, 0.0), // Red color for anchor markers
                                    custom_size: Some(bevy::math::Vec2::new(6.0, 6.0)),
                                    ..Default::default()
                                },
                            ));

                            info!(
                                "  📍 Spawned anchor joint marker at {:?} on entity {:?}",
                                marker_position,
                                connected_entity
                            );
                        }
                    }
                }
                is_jointed_mob = true;
            }
        }

        // Check if this entity is referenced in any existing joints
        if !is_jointed_mob {
            for joint_entity in joint_entities_q.iter() {
                if let Ok(joint) = all_joints_q.get(joint_entity) {
                    if joint.entity1 == event.mob_entity || joint.entity2 == event.mob_entity {
                        info!(
                            "🔗 JOINTED MOB DESTROYED! Entity: {:?} was connected via joint {:?}",
                            event.mob_entity,
                            joint_entity
                        );
                        info!(
                            "  ├─ Joint connected Entity {:?} ⟷ Entity {:?}",
                            joint.entity1,
                            joint.entity2
                        );

                        // Spawn markers on the remaining entity
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

                            // Transform the local anchor to world position
                            let marker_position = remaining_transform.translation +
                                (remaining_transform.rotation * anchor_pos.extend(0.0));

                            // Spawn a debug marker entity at the joint location
                            cmds.spawn((
                                Transform::from_translation(marker_position),
                                Name::new(format!("JointMarker_Piece_{}", joint_entity.index())),
                                Sprite {
                                    color: Color::srgb(0.0, 1.0, 0.0), // Green color for piece markers
                                    custom_size: Some(bevy::math::Vec2::new(6.0, 6.0)),
                                    ..Default::default()
                                },
                            ));

                            info!(
                                "  📍 Spawned joint marker at {:?} on remaining entity {:?}",
                                marker_position,
                                remaining_entity
                            );
                        }

                        is_jointed_mob = true;
                        // Don't break - continue checking for more joints this mob might be part of
                    }
                }
            }
        }

        if !is_jointed_mob {
            info!("📝 Mob {:?} was not part of any joint relationship", event.mob_entity);
        }
    }
}
