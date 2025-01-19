use bevy::app::{Plugin, Startup};
use bevy_alt_ui_navigation_lite::DefaultNavigationPlugins;
use leafwing_abilities::plugin::AbilityPlugin;
use leafwing_input_manager::plugin::InputManagerPlugin;

use super::{systems::setup_input_system, PlayerAbility, PlayerAction};

/// Plugin structure for handling input in the Thetawave game.
pub(crate) struct ThetawaveInputPlugin;

/// Implementation of the Plugin trait for ThetawaveInputPlugin
impl Plugin for ThetawaveInputPlugin {
    /// Builds the plugin by adding navigation plugins and setup systems
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((DefaultNavigationPlugins,))
            .add_plugins(InputManagerPlugin::<PlayerAction>::default())
            .add_plugins(InputManagerPlugin::<PlayerAbility>::default())
            .add_plugins(AbilityPlugin::<PlayerAbility>::default())
            .add_systems(Startup, setup_input_system);
    }
}
