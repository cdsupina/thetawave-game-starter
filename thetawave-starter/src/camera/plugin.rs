use super::systems::setup_cameras_system;
use bevy::app::{Plugin, PostStartup};

/// A plugin for managing the Thetawave game's camera systems
pub(crate) struct ThetawaveCameraPlugin;

impl Plugin for ThetawaveCameraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(PostStartup, setup_cameras_system);
    }
}
