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
    behavior::{MobBehaviorsResource, systems::move_system},
};

pub(crate) struct ThetawaveMobBehaviorPlugin;

impl Plugin for ThetawaveMobBehaviorPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(BehavePlugin::default());
        app.insert_resource(MobBehaviorsResource::new());
        app.add_systems(
            Update,
            move_system.run_if(
                in_state(AppState::Game)
                    .and(in_state(GameState::Playing))
                    .and(|mob_res: Res<MobDebugSettings>| mob_res.behaviors_enabled),
            ),
        );
    }
}
