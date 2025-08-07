use bevy::{
    app::{Plugin, Update},
    ecs::{
        schedule::{Condition, IntoScheduleConfigs},
        system::Res,
    },
    state::condition::in_state,
};
use bevy_behave::prelude::BehavePlugin;
use thetawave_states::{AppState, GameState};

use crate::{
    MobDebugSettings,
    behavior::{
        MobBehaviorsResource,
        systems::{
            brake_angular_system, brake_horizontal_system, do_for_time_system,
            find_player_target_system, lose_target_system, move_down_system, move_forward_system,
            move_to_system, move_to_target_system, rotate_to_target_system, spawn_mob_system,
        },
    },
};

pub(crate) struct ThetawaveMobBehaviorPlugin;

impl Plugin for ThetawaveMobBehaviorPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(BehavePlugin::default());
        app.insert_resource(MobBehaviorsResource::new());
        app.add_systems(
            Update,
            (
                move_down_system,
                brake_horizontal_system,
                move_to_system,
                find_player_target_system,
                move_to_target_system,
                rotate_to_target_system,
                move_forward_system,
                lose_target_system,
                brake_angular_system,
                spawn_mob_system,
                do_for_time_system,
            )
                .run_if(
                    in_state(AppState::Game)
                        .and(in_state(GameState::Playing))
                        .and(|mob_res: Res<MobDebugSettings>| mob_res.behaviors_enabled),
                ),
        );
    }
}
