use bevy::{
    app::{Plugin, Update},
    ecs::{
        schedule::{Condition, IntoScheduleConfigs},
        system::Res,
    },
    state::condition::in_state,
};
use thetawave_states::{AppState, GameState};
use toml::from_slice;

use crate::{
    MobDebugSettings,
    behavior::{
        MobBehaviorsResource,
        data::MobBehaviorEvent,
        systems::{
            activate_behaviors_system, brake_horizontal_system, brake_vertical_system,
            move_down_system, move_left_system, move_right_system,
        },
    },
};

pub(crate) struct ThetawaveMobBehaviorPlugin;

impl Plugin for ThetawaveMobBehaviorPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_event::<MobBehaviorEvent>();

        app.insert_resource(
            from_slice::<MobBehaviorsResource>(include_bytes!(
                "../../../assets/data/mob_behaviors.toml"
            ))
            .expect("Failed to parse MobBehaviorsResource from `mob_behaviors.toml`."),
        );

        app.add_systems(
            Update,
            (
                activate_behaviors_system
                    .run_if(|mob_res: Res<MobDebugSettings>| mob_res.behaviors_enabled),
                move_down_system,
                move_left_system,
                move_right_system,
                brake_horizontal_system,
                brake_vertical_system,
            )
                .run_if(in_state(AppState::Game).and(in_state(GameState::Playing))),
        );
    }
}
