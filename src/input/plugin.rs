use bevy::app::{Plugin, Startup};
use bevy_alt_ui_navigation_lite::DefaultNavigationPlugins;

use super::systems::setup;

/// Plugin structure for handling input in the Thetawave game.
pub(crate) struct ThetawaveInputPlugin;

/// Implementation of the Plugin trait for ThetawaveInputPlugin
impl Plugin for ThetawaveInputPlugin {
    /// Builds the plugin by adding navigation plugins and setup systems
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(DefaultNavigationPlugins)
            .add_systems(Startup, setup);
    }
}
