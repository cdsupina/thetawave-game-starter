mod attributes;
mod behavior;
mod spawn;

pub use attributes::{MobType, SpawnMobEvent};
pub use spawn::MobDebugSettings;

use bevy::{
    app::{Plugin, Update},
    ecs::schedule::{Condition, IntoScheduleConfigs, common_conditions::on_event},
    state::condition::in_state,
};
use thetawave_states::{AppState, GameState};

use crate::{
    attributes::ThetawaveAttributesPlugin, behavior::ThetawaveMobBehaviorPlugin,
    spawn::spawn_mob_system,
};

pub struct ThetawaveMobsPlugin;

impl Plugin for ThetawaveMobsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        #[cfg(feature = "debug")]
        app.insert_resource(MobDebugSettings::default());

        app.add_plugins((ThetawaveMobBehaviorPlugin, ThetawaveAttributesPlugin));

        app.add_systems(
            Update,
            spawn_mob_system.run_if(
                in_state(AppState::Game)
                    .and(in_state(GameState::Playing).and(on_event::<SpawnMobEvent>)),
            ),
        )
        .add_event::<SpawnMobEvent>();
    }
}
