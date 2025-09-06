use bevy::{
    app::{Plugin, Startup, Update},
    ecs::schedule::{Condition, IntoScheduleConfigs, common_conditions::not},
    state::{
        condition::in_state,
        state::{OnEnter, OnExit},
    },
};
use bevy_alt_ui_navigation_lite::DefaultNavigationPlugins;
use leafwing_abilities::plugin::AbilityPlugin;
use leafwing_input_manager::plugin::InputManagerPlugin;
use thetawave_states::{AppState, DebugState, GameState, MainMenuState};

mod character;
mod input;
mod player;

pub use character::{
    CharacterType, CharactersResource, ChosenCharacterData, ChosenCharactersResource,
};
pub use input::{
    CharacterCarouselAction, DummyGamepad, InputType, PlayerAbility, PlayerAction, PlayerJoinEvent,
    PlayerNum,
};
pub use player::{PlayerDeathEvent, PlayerStats};
use thetawave_assets::load_with_extended;

use crate::{
    character::reset_chosen_characters_resource_system,
    input::{
        disable_additional_players_navigation_system, disable_horizontal_navigation_system,
        enable_additional_players_navigation_system, enable_horizontal_navigation_system,
        setup_input_system,
    },
    player::{player_ability_system, player_death_system, player_move_system},
};

/// Plugin structure for handling input in the Thetawave game.
pub struct ThetawavePlayerPlugin;

/// Implementation of the Plugin trait for ThetawaveInputPlugin
impl Plugin for ThetawavePlayerPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.insert_resource(
            load_with_extended::<CharactersResource>(
                include_bytes!("../../assets/data/character_attributes.toml"),
                "character_attributes.toml"
            )
        )
        .init_resource::<ChosenCharactersResource>()
        .add_plugins(DefaultNavigationPlugins)
        .add_plugins(InputManagerPlugin::<PlayerAction>::default())
        .add_plugins(InputManagerPlugin::<PlayerAbility>::default())
        .add_plugins(InputManagerPlugin::<CharacterCarouselAction>::default())
        .add_plugins(AbilityPlugin::<PlayerAbility>::default())
        .add_event::<PlayerJoinEvent>()
        .add_event::<PlayerDeathEvent>()
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
        )
        .add_systems(
            Update,
            (
                player_move_system,
                player_death_system,
                player_ability_system.run_if(not(in_state(DebugState::Debug))), // to prevent abilities from activating on mouse clicks
            )
                .run_if(in_state(AppState::Game).and(in_state(GameState::Playing))),
        )
        .add_systems(
            OnEnter(MainMenuState::Title),
            reset_chosen_characters_resource_system,
        );
    }
}
