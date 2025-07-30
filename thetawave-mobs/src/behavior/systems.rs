use avian2d::prelude::LinearVelocity;
use bevy::{
    ecs::{
        entity::Entity,
        event::{EventReader, EventWriter},
        query::With,
        system::Query,
    },
    platform::collections::HashMap,
};

use crate::{
    attributes::MobAttributesComponent,
    behavior::data::{MobBehavior, MobBehaviorEvent, MobBehaviorSequence},
};

/// Create events to activate behaviors
/// TODO: Update timers and change active behavior blocks on behavior sequences
pub(super) fn activate_behaviors_system(
    mob_query: Query<(Entity, &MobBehaviorSequence)>,
    mut behavior_event_writer: EventWriter<MobBehaviorEvent>,
) {
    let mut behavior_events = HashMap::<MobBehavior, Vec<Entity>>::new();

    for (entity, sequence) in mob_query.iter() {
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

/// Accelerate the mob downwards using the mobs acceleration attribute
pub(super) fn move_down_system(
    mut behavior_event_reader: EventReader<MobBehaviorEvent>,
    mut mob_query: Query<(&mut LinearVelocity, &MobAttributesComponent), With<MobBehaviorSequence>>,
) {
    for event in behavior_event_reader.read() {
        for entity in &event.entities {
            if let Ok((mut velocity, attributes)) = mob_query.get_mut(*entity) {
                velocity.y -= attributes.linear_acceleration;
            }
        }
    }
}
