use bevy::app::{Plugin, Startup};

use super::systems::setup;

/// A plugin for managing the Thetawave game's camera systems
pub(crate) struct ThetawaveCameraPlugin;

impl Plugin for ThetawaveCameraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, setup);
    }
}
