use crate::data::{
    ActivateParticleEvent, BloodEffectManager, ParticleLifeTimer, ToggleActiveParticleEvent,
};
use bevy::{
    ecs::{
        entity::Entity,
        message::{MessageReader, MessageWriter},
        query::Without,
        system::{Commands, Query, Res},
    },
    log::warn,
    time::Time,
    transform::components::Transform,
};
use bevy_enoki::prelude::ParticleSpawnerState;

/// Listens for events to activate or deactivate particles
pub(crate) fn activate_particle_effect_system(
    mut particle_spawner_state_q: Query<&mut ParticleSpawnerState>,
    mut activate_particle_event_reader: MessageReader<ActivateParticleEvent>,
) {
    for event in activate_particle_event_reader.read() {
        if let Ok(mut spawner_state) = particle_spawner_state_q.get_mut(event.entity) {
            spawner_state.active = event.active;
        }
    }
}

/// Listens for events to toggle particles active
pub(crate) fn toggle_particle_effect_system(
    mut particle_spawner_state_q: Query<&mut ParticleSpawnerState>,
    mut toggle_active_particle_event_reader: MessageReader<ToggleActiveParticleEvent>,
) {
    for event in toggle_active_particle_event_reader.read() {
        if let Ok(mut spawner_state) = particle_spawner_state_q.get_mut(event.entity) {
            spawner_state.active = !spawner_state.active;
        }
    }
}

/// Updates particle spawner positions to match their parent entity positions
/// This maintains visual consistency while keeping spawners as independent entities
pub(crate) fn particle_position_tracking_system(
    mut particle_q: Query<(&mut Transform, &ParticleLifeTimer)>,
    parent_q: Query<&Transform, (Without<ParticleLifeTimer>,)>,
) {
    for (mut particle_transform, life_timer) in particle_q.iter_mut() {
        if let Some(parent_entity) = life_timer.parent_entity
            && let Ok(parent_transform) = parent_q.get(parent_entity)
        {
            // Update particle spawner position to match parent + offset
            particle_transform.translation =
                parent_transform.translation + (parent_transform.rotation * life_timer.offset);
            particle_transform.rotation = parent_transform.rotation;
        }
        // If parent doesn't exist anymore, the spawner keeps its last known position
    }
}

/// Manages particle spawner lifetime for inactive spawners
/// Ticks the timer and despawns spawners after all particles have completed their lifecycle
pub(crate) fn particle_lifetime_management_system(
    mut cmds: Commands,
    mut particle_q: Query<(Entity, &mut ParticleLifeTimer, &mut ParticleSpawnerState)>,
    parent_q: Query<Entity, (Without<ParticleLifeTimer>,)>,
    time: Res<Time>,
) {
    for (entity, mut life_timer, mut spawner_state) in particle_q.iter_mut() {
        // Check if parent still exists - if not, deactivate this orphaned effect
        if let Some(parent_entity) = life_timer.parent_entity
            && parent_q.get(parent_entity).is_err()
            && spawner_state.active
        {
            // Parent is gone and spawner is still active - deactivate it
            spawner_state.active = false;
        }

        // Only tick timer for inactive spawners (those that have been deactivated)
        // Reset the timer if the effect activates again, good for effects like the blood effect that toggle active and inactive
        if !spawner_state.active && life_timer.timer.tick(time.delta()).just_finished() {
            // Timer expired, despawn the spawner entity
            cmds.entity(entity).despawn();
        } else if spawner_state.active {
            life_timer.timer.reset();
        }
    }
}

/// Manages blood effects with random activation/deactivation intervals for realistic pulsing
pub(crate) fn blood_effect_management_system(
    mut blood_effects_q: Query<(Entity, &mut BloodEffectManager, &ParticleLifeTimer)>,
    parent_q: Query<Entity, (Without<ParticleLifeTimer>,)>,
    mut active_particle_event_writer: MessageWriter<ActivateParticleEvent>,
    time: Res<Time>,
) {
    for (entity, mut blood_manager, life_timer) in blood_effects_q.iter_mut() {
        // Every blood entity should have a parent entity specified due to it spawning from a joint
        if let Some(parent_entity) = life_timer.parent_entity {
            // Deactivate the blood effect if the pulses remaining is 0 or the parent no longer exists
            if blood_manager.pulses_remaining == 0 || parent_q.get(parent_entity).is_err() {
                active_particle_event_writer.write(ActivateParticleEvent {
                    entity,
                    active: false,
                });
            } else if blood_manager.timer.tick(time.delta()).just_finished() {
                // Decrease pulses remaining before switching states
                blood_manager.pulses_remaining -= 1;

                // If there are remaining pulses reset the timer to alternate interval
                if blood_manager.pulses_remaining > 0 {
                    blood_manager.reset_timer_to_random();
                }

                // Set particle effect state based on BloodEffectManager's NEW state (after toggle)
                active_particle_event_writer.write(ActivateParticleEvent {
                    entity,
                    active: blood_manager.is_active,
                });
            }
        } else {
            warn!("No parent entity found for blood effect: {:?}", entity);
        }
    }
}
