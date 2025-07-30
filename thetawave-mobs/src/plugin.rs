use bevy::{
    app::{Plugin, Update},
    ecs::schedule::{Condition, IntoScheduleConfigs, common_conditions::on_event},
    state::condition::in_state,
};
use thetawave_states::{AppState, GameState};
use toml::from_slice;

use crate::{
    SpawnMobEvent, behavior::ThetawaveMobBehaviorPlugin, data::MobAttributesResource,
    systems::spawn_mob_system,
};

pub struct ThetawaveMobsPlugin;

impl Plugin for ThetawaveMobsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(ThetawaveMobBehaviorPlugin);

        app.insert_resource(
            from_slice::<MobAttributesResource>(include_bytes!(
                "../../assets/data/mob_attributes.toml"
            ))
            .expect("Failed to parse MobAttributesResource from `mob_attributes.toml`."),
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
