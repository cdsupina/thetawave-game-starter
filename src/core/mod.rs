use bevy::{
    app::{Plugin, Update},
    ecs::{event::EventWriter, schedule::IntoScheduleConfigs, system::Query},
    state::condition::in_state,
};
use leafwing_input_manager::prelude::ActionState;

use crate::{
    input::PlayerAction,
    player::PlayerNum,
    states::{AppState, ToggleGameStateEvent},
};

pub(crate) struct ThetawaveCorePlugin;

impl Plugin for ThetawaveCorePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(
            Update,
            toggle_game_state_system.run_if(in_state(AppState::Game)),
        );
    }
}

/// System for toggling the paused state only for player
fn toggle_game_state_system(
    player_action_q: Query<(&PlayerNum, &ActionState<PlayerAction>)>,
    mut toggle_game_state_event: EventWriter<ToggleGameStateEvent>,
) {
    for (player_num, player_action) in player_action_q.iter() {
        if matches!(player_num, PlayerNum::One) && player_action.just_released(&PlayerAction::Pause)
        {
            toggle_game_state_event.write(ToggleGameStateEvent);
        }
    }
}
