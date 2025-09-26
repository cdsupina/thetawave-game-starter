use bevy::{
    ecs::{
        event::{EventReader, EventWriter},
        system::{Commands, Query},
    },
    transform::components::Transform,
};
use thetawave_core::Faction;
use thetawave_particles::SpawnParticleEffectEvent;

use crate::MobDeathEvent;

/// Checks for mob death events, despawns the entity and spawns an explosion
pub(crate) fn mob_death_system(
    mut cmds: Commands,
    mut mob_death_event_reader: EventReader<MobDeathEvent>,
    mut particle_effect_event_writer: EventWriter<SpawnParticleEffectEvent>,
    mob_q: Query<&Transform>,
) {
    for event in mob_death_event_reader.read() {
        cmds.entity(event.mob_entity).despawn();
        if let Ok(mob_transform) = mob_q.get(event.mob_entity) {
            particle_effect_event_writer.write(SpawnParticleEffectEvent {
                parent_entity: None,
                effect_type: "explosion".to_string(),
                faction: Faction::Enemy,
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
