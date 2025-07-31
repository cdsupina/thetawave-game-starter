use super::{
    data::{CharactersResource, ChosenCharactersResource},
    systems::{
        player_ability_system, player_move_system, reset_chosen_characters_resource_system,
        spawn_players_system,
    },
};
use bevy::{
    app::{Plugin, Update},
    ecs::schedule::common_conditions::not,
    prelude::{Condition, IntoScheduleConfigs, OnEnter, in_state},
};
use thetawave_states::{AppState, DebugState, GameState, MainMenuState};
use toml::from_slice;

/// Plugin for managing player entities
pub(crate) struct ThetawavePlayerPlugin;

impl Plugin for ThetawavePlayerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(
            from_slice::<CharactersResource>(include_bytes!(
                "../../../assets/data/character_attributes.toml"
            ))
            .expect("Failed to parse CharactersResource from `character_attributes.toml`."),
        )
        .init_resource::<ChosenCharactersResource>()
        .add_systems(OnEnter(AppState::Game), spawn_players_system)
        .add_systems(
            Update,
            (
                player_move_system,
                player_ability_system.run_if(not(in_state(DebugState::Debug))),
            )
                .run_if(in_state(AppState::Game).and(in_state(GameState::Playing))),
        )
        .add_systems(
            OnEnter(MainMenuState::Title),
            reset_chosen_characters_resource_system,
        );
    }
}
