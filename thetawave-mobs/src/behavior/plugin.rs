use bevy::app::{Plugin, Update};
use bevy_behave::prelude::BehavePlugin;

use crate::behavior::{MobBehaviorsResource, systems::move_system};

pub(crate) struct ThetawaveMobBehaviorPlugin;

impl Plugin for ThetawaveMobBehaviorPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(BehavePlugin::default());
        app.insert_resource(MobBehaviorsResource::new());
        app.add_systems(Update, move_system);
    }
}
