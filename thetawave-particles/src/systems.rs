use crate::data::ActivateParticleEvent;
use bevy::ecs::{event::EventReader, system::Query};
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
