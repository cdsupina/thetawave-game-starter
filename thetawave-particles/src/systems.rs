use crate::data::{ActivateParticleEvent, ParticleLifeTimer};
use bevy::{
    ecs::{
        entity::Entity,
        event::EventReader,
        query::Without,
        system::{Commands, Query, Res},
    },
    time::Time,
    transform::components::Transform,
};
use bevy_enoki::prelude::ParticleSpawnerState;

/// Listens for events to activate or deactivate particles
pub(crate) fn activate_particle_effect_system(
    mut particle_spawner_state_q: Query<&mut ParticleSpawnerState>,
    mut activate_particle_event_reader: EventReader<ActivateParticleEvent>,
) {
    for event in activate_particle_event_reader.read() {
        if let Ok(mut spawner_state) = particle_spawner_state_q.get_mut(event.entity) {
            spawner_state.active = event.active;
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
            particle_transform.translation = parent_transform.translation + (parent_transform.rotation * life_timer.offset);
            particle_transform.rotation = parent_transform.rotation;
        }
        // If parent doesn't exist anymore, the spawner keeps its last known position
    }
}

/// Manages particle spawner lifetime for inactive spawners
/// Ticks the timer and despawns spawners after all particles have completed their lifecycle
pub(crate) fn particle_lifetime_management_system(
    mut cmds: Commands,
    mut particle_q: Query<(Entity, &mut ParticleLifeTimer, &ParticleSpawnerState)>,
    time: Res<Time>,
) {
    for (entity, mut life_timer, spawner_state) in particle_q.iter_mut() {
        // Only tick timer for inactive spawners (those that have been deactivated)
        if !spawner_state.active && life_timer.timer.tick(time.delta()).just_finished() {
            // Timer expired, despawn the spawner entity
            cmds.entity(entity).despawn();
        }
    }
}
