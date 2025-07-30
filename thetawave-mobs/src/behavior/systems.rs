use avian2d::prelude::LinearVelocity;
use bevy::{
    ecs::{
        entity::Entity,
        event::{EventReader, EventWriter},
        query::With,
        system::{Query, Res},
    },
    platform::collections::HashMap,
    time::Time,
};

use crate::{
    attributes::MobAttributesComponent,
    behavior::data::{MobBehavior, MobBehaviorEvent, MobBehaviorSequence},
};

/// Create events to activate behaviors
/// TODO: Update timers and change active behavior blocks on behavior sequences
pub(super) fn activate_behaviors_system(
    mut mob_query: Query<(Entity, &mut MobBehaviorSequence)>,
    mut behavior_event_writer: EventWriter<MobBehaviorEvent>,
    time: Res<Time>,
) {
    let mut behavior_events = HashMap::<MobBehavior, Vec<Entity>>::new();

    for (entity, mut sequence) in mob_query.iter_mut() {
        // update the timer and move to next block when it finshes
        sequence.update_timer(time.delta_secs());

        // Get the active block for the MobBehaviorSequence
        if let Some(active_block) = sequence.get_active_block() {
            // For each behavior in the active block, add the entity to the corresponding vector in the HashMap
            for behavior in active_block.behaviors.iter() {
                if let Some(entity_vec) = behavior_events.get_mut(behavior) {
                    entity_vec.push(entity);
                } else {
                    behavior_events.insert(behavior.clone(), vec![entity]);
                }
            }
        }
    }

    // Convert the hashmap into events to send
    for (behavior, entities) in behavior_events {
        behavior_event_writer.write(MobBehaviorEvent { behavior, entities });
    }
}

/// Accelerate the mob downwards using the mob's acceleration attribute
pub(super) fn move_down_system(
    mut behavior_event_reader: EventReader<MobBehaviorEvent>,
    mut mob_query: Query<(&mut LinearVelocity, &MobAttributesComponent), With<MobBehaviorSequence>>,
) {
    for event in behavior_event_reader.read() {
        if matches!(event.behavior, MobBehavior::MoveDown) {
            for entity in &event.entities {
                if let Ok((mut velocity, attributes)) = mob_query.get_mut(*entity) {
                    velocity.y -= attributes.linear_acceleration.y;
                    velocity.y = velocity.y.max(-attributes.max_linear_speed.y);
                }
            }
        }
    }
}

/// Accelerate the mob to the right using the mob's acceleration attribute
pub(super) fn move_right_system(
    mut behavior_event_reader: EventReader<MobBehaviorEvent>,
    mut mob_query: Query<(&mut LinearVelocity, &MobAttributesComponent), With<MobBehaviorSequence>>,
) {
    for event in behavior_event_reader.read() {
        if matches!(event.behavior, MobBehavior::MoveRight) {
            for entity in &event.entities {
                if let Ok((mut velocity, attributes)) = mob_query.get_mut(*entity) {
                    velocity.x += attributes.linear_acceleration.x;
                    velocity.x = velocity.x.min(attributes.max_linear_speed.x);
                }
            }
        }
    }
}

/// Accelerate the mob to the left using the mob's acceleration attribute
pub(super) fn move_left_system(
    mut behavior_event_reader: EventReader<MobBehaviorEvent>,
    mut mob_query: Query<(&mut LinearVelocity, &MobAttributesComponent), With<MobBehaviorSequence>>,
) {
    for event in behavior_event_reader.read() {
        if matches!(event.behavior, MobBehavior::MoveLeft) {
            for entity in &event.entities {
                if let Ok((mut velocity, attributes)) = mob_query.get_mut(*entity) {
                    velocity.x -= attributes.linear_acceleration.x;
                    velocity.x = velocity.x.max(-attributes.max_linear_speed.x);
                }
            }
        }
    }
}

/// Decelerate the mob horizontally using the mob's deceleration attribute
pub(super) fn brake_horizontal_system(
    mut behavior_event_reader: EventReader<MobBehaviorEvent>,
    mut mob_query: Query<(&mut LinearVelocity, &MobAttributesComponent), With<MobBehaviorSequence>>,
) {
    for event in behavior_event_reader.read() {
        if matches!(event.behavior, MobBehavior::BrakeHorizontal) {
            for entity in &event.entities {
                if let Ok((mut velocity, attributes)) = mob_query.get_mut(*entity) {
                    if velocity.x > 0.0 {
                        velocity.x = (velocity.x - attributes.linear_deceleration.x).max(0.0);
                    } else if velocity.x < 0.0 {
                        velocity.x = (velocity.x + attributes.linear_deceleration.x).min(0.0);
                    }
                }
            }
        }
    }
}

/// Decelerate the mob vertically using the mob's deceleration attribute
pub(super) fn brake_vertical_system(
    mut behavior_event_reader: EventReader<MobBehaviorEvent>,
    mut mob_query: Query<(&mut LinearVelocity, &MobAttributesComponent), With<MobBehaviorSequence>>,
) {
    for event in behavior_event_reader.read() {
        if matches!(event.behavior, MobBehavior::BrakeVertical) {
            for entity in &event.entities {
                if let Ok((mut velocity, attributes)) = mob_query.get_mut(*entity) {
                    if velocity.y > 0.0 {
                        velocity.y = (velocity.y - attributes.linear_deceleration.y).max(0.0);
                    } else if velocity.y < 0.0 {
                        velocity.y = (velocity.y + attributes.linear_deceleration.y).min(0.0);
                    }
                }
            }
        }
    }
}
