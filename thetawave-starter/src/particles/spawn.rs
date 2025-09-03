use bevy::ecs::{
    event::EventReader,
    system::{Commands, Res},
};
use thetawave_assets::{ExtendedGameAssets, GameAssets, ParticleMaterials};
use thetawave_particles::{SpawnParticleEffectEvent, spawn_particle_effect};

pub(in crate::particles) fn spawn_particle_effect_system(
    mut cmds: Commands,
    extended_assets: Res<ExtendedGameAssets>,
    assets: Res<GameAssets>,
    materials: Res<ParticleMaterials>,
    mut spawn_particle_effect_event_reader: EventReader<SpawnParticleEffectEvent>,
) {
    for event in spawn_particle_effect_event_reader.read() {
        let _particle_entity = spawn_particle_effect(
            &mut cmds,
            event.parent_entity,
            &event.effect_type,
            &event.faction,
            &event.transform,
            &extended_assets,
            &assets,
            &materials,
        );
    }
}
