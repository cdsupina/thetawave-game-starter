use super::{
    data::PlayerAction,
    systems::{player_move_system, spawn_players_system},
};
use crate::states::{AppState, GameState};
use bevy::{
    app::{Plugin, Update},
    prelude::{in_state, Condition, IntoSystemConfigs, OnEnter},
};
use leafwing_input_manager::plugin::InputManagerPlugin;

/// Plugin for managing player entities
pub(crate) struct ThetawavePlayerPlugin;

impl Plugin for ThetawavePlayerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default())
            .add_systems(OnEnter(AppState::Game), spawn_players_system)
            .add_systems(
                Update,
                player_move_system
                    .run_if(in_state(AppState::Game).and(in_state(GameState::Playing))),
            );
    }
}
