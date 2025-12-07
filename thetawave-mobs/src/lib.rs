mod attributes;
mod behavior;
mod spawn;
mod systems;

pub use attributes::{MobDeathEvent, MobMarker};
pub use spawn::MobDebugSettings;

use bevy::{
    app::{Plugin, Update},
    ecs::schedule::{IntoScheduleConfigs, SystemCondition, common_conditions::on_message},
    state::condition::in_state,
};
use thetawave_core::{AppState, GameState};
use thetawave_particles::SpawnerParticleEffectSpawnedEvent;

use crate::{
    attributes::ThetawaveAttributesPlugin,
    behavior::ThetawaveMobBehaviorPlugin,
    spawn::{connect_effect_to_spawner, spawn_mob_system},
    systems::{joint_bleed_system, mob_death_system},
};

pub use spawn::SpawnMobEvent;

pub struct ThetawaveMobsPlugin;

impl Plugin for ThetawaveMobsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        #[cfg(feature = "debug")]
        app.insert_resource(MobDebugSettings::default());

        app.add_plugins((ThetawaveMobBehaviorPlugin, ThetawaveAttributesPlugin));

        app.add_systems(
            Update,
            (
                spawn_mob_system.run_if(on_message::<SpawnMobEvent>),
                connect_effect_to_spawner.run_if(on_message::<SpawnerParticleEffectSpawnedEvent>),
                (joint_bleed_system, mob_death_system)
                    .chain()
                    .run_if(on_message::<MobDeathEvent>),
            )
                .run_if(in_state(AppState::Game).and(in_state(GameState::Playing))),
        )
        .add_message::<SpawnMobEvent>();
    }
}
