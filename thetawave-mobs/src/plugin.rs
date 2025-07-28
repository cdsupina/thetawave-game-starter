use bevy::{
    app::{Plugin, Update},
    ecs::schedule::{common_conditions::on_event, Condition, IntoScheduleConfigs},
    state::condition::in_state,
};
use thetawave_states::{AppState, GameState};
use toml::from_slice;

use crate::{systems::spawn_mob_system, MobResource, SpawnMobEvent};

pub struct ThetawaveMobsPlugin;

impl Plugin for ThetawaveMobsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(
            from_slice::<MobResource>(include_bytes!("../../assets/data/mobs.toml"))
                .expect("Failed to parse MobResource from `mobs.toml`."),
        );

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
