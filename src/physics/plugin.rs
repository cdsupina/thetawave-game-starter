use super::systems::pause_physics_system;
use crate::states::AppState;
use avian2d::{prelude::PhysicsDebugPlugin, PhysicsPlugins};
use bevy::{
    app::{Plugin, Update},
    prelude::{in_state, IntoSystemConfigs},
};

pub(crate) struct ThetawavePhysicsPlugin;

impl Plugin for ThetawavePhysicsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(PhysicsPlugins::default()).add_systems(
            Update,
            pause_physics_system.run_if(in_state(AppState::Game)),
        );

        if cfg!(feature = "physics_debug") {
            app.add_plugins(PhysicsDebugPlugin::default());
        }
    }
}
