use avian2d::{prelude::PhysicsDebugPlugin, PhysicsPlugins};
use bevy::app::Plugin;

pub(crate) struct ThetawavePhysicsPlugin;

impl Plugin for ThetawavePhysicsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((PhysicsPlugins::default(), PhysicsDebugPlugin::default()));
    }
}
