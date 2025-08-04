use bevy::{
    app::{Plugin, Update},
    ecs::{schedule::IntoScheduleConfigs, system::Res},
};
use bevy_behave::prelude::BehavePlugin;

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
            move_system.run_if(|mob_res: Res<MobDebugSettings>| mob_res.behaviors_enabled),
        );
    }
}
