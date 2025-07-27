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
    prelude::{in_state, Condition, IntoScheduleConfigs, OnEnter},
};
use thetawave_states::{AppState, DebugState, GameState, MainMenuState};

/// Plugin for managing player entities
pub(crate) struct ThetawavePlayerPlugin;

impl Plugin for ThetawavePlayerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<CharactersResource>()
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
