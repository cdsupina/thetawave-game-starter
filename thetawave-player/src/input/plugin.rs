use bevy::{
    app::{Plugin, Startup, Update},
    ecs::schedule::IntoScheduleConfigs,
    state::{
        condition::in_state,
        state::{OnEnter, OnExit},
    },
};
use bevy_alt_ui_navigation_lite::DefaultNavigationPlugins;
use leafwing_abilities::plugin::AbilityPlugin;
use leafwing_input_manager::plugin::InputManagerPlugin;
use thetawave_states::MainMenuState;

use crate::input::{
    CharacterCarouselAction, PlayerAbility, PlayerAction, PlayerJoinEvent,
    systems::{
        disable_additional_players_navigation_system, disable_horizontal_navigation_system,
        enable_additional_players_navigation_system, enable_horizontal_navigation_system,
        setup_input_system,
    },
};

/// Plugin structure for handling input in the Thetawave game.
pub struct ThetawaveInputPlugin;

/// Implementation of the Plugin trait for ThetawaveInputPlugin
impl Plugin for ThetawaveInputPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(DefaultNavigationPlugins)
            .add_plugins(InputManagerPlugin::<PlayerAction>::default())
            .add_plugins(InputManagerPlugin::<PlayerAbility>::default())
            .add_plugins(InputManagerPlugin::<CharacterCarouselAction>::default())
            .add_plugins(AbilityPlugin::<PlayerAbility>::default())
            .add_event::<PlayerJoinEvent>()
            .add_systems(Startup, setup_input_system)
            .add_systems(
                OnEnter(MainMenuState::Title),
                enable_additional_players_navigation_system,
            )
            .add_systems(
                OnEnter(MainMenuState::CharacterSelection),
                disable_horizontal_navigation_system,
            )
            .add_systems(
                OnExit(MainMenuState::CharacterSelection),
                enable_horizontal_navigation_system,
            )
            .add_systems(
                Update,
                disable_additional_players_navigation_system
                    .run_if(in_state(MainMenuState::CharacterSelection)),
            );
    }
}
