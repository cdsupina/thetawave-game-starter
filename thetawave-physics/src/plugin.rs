use super::systems::{pause_physics_system, resume_physics_system};
use avian2d::{prelude::Gravity, PhysicsPlugins};
use bevy::{
    app::{Plugin, Update},
    math::Vec2,
    prelude::{in_state, IntoScheduleConfigs, OnEnter},
};
use thetawave_states::AppState;

pub struct ThetawavePhysicsPlugin;

impl Plugin for ThetawavePhysicsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(PhysicsPlugins::default())
            .insert_resource(Gravity(Vec2::ZERO))
            .add_systems(
                Update,
                pause_physics_system.run_if(in_state(AppState::Game)),
            )
            .add_systems(OnEnter(AppState::GameLoading), resume_physics_system);

        #[cfg(feature = "physics_debug")]
        {
            use avian2d::prelude::{
                PhysicsDebugPlugin, PhysicsDiagnosticsPlugin, PhysicsDiagnosticsUiPlugin,
            };
            use bevy::ecs::schedule::common_conditions::resource_changed;

            use crate::PhysicsDebugSettings;

            app.insert_resource(PhysicsDebugSettings::default());

            app.add_plugins((
                PhysicsDebugPlugin::default(),
                PhysicsDiagnosticsPlugin,
                PhysicsDiagnosticsUiPlugin,
            ));
            app.add_systems(
                Update,
                crate::systems::toggle_physics_debug_system
                    .run_if(resource_changed::<PhysicsDebugSettings>),
            );
        }
    }
}
