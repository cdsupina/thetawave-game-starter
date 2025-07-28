use bevy::{
    app::{Plugin, Update},
    ecs::schedule::{common_conditions::on_event, Condition, IntoScheduleConfigs},
    state::condition::in_state,
};
use thetawave_states::{AppState, GameState};
use toml::from_slice;

use crate::{
    behavior::MobBehaviorsResource, data::MobAttributesResource, systems::spawn_mob_system,
    SpawnMobEvent,
};

pub struct ThetawaveMobsPlugin;

impl Plugin for ThetawaveMobsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(
            from_slice::<MobAttributesResource>(include_bytes!(
                "../../assets/data/mob_attributes.toml"
            ))
            .expect("Failed to parse MobAttributesResource from `mob_attributes.toml`."),
        );

        app.insert_resource(
            from_slice::<MobBehaviorsResource>(include_bytes!(
                "../../assets/data/mob_behaviors.toml"
            ))
            .expect("Failed to parse MobBehaviorsResource from `mob_behaviors.toml`."),
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
