use super::{
    systems::{
        disable_additional_players_navigation_system, enable_additional_players_navigation_system,
        setup_input_system,
    },
    PlayerAbility, PlayerAction,
};
use crate::states::MainMenuState;
use bevy::{
    app::{Plugin, Startup, Update},
    prelude::{in_state, IntoSystemConfigs, OnEnter},
};
use bevy_alt_ui_navigation_lite::DefaultNavigationPlugins;
use leafwing_abilities::plugin::AbilityPlugin;
use leafwing_input_manager::plugin::InputManagerPlugin;

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
            .add_systems(Startup, setup_input_system)
            .add_systems(
                Update,
                disable_additional_players_navigation_system
                    .run_if(in_state(MainMenuState::CharacterSelection)),
            )
            .add_systems(
                OnEnter(MainMenuState::Title),
                enable_additional_players_navigation_system,
            );
    }
}
