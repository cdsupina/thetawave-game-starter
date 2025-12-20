use bevy::{
    app::{Plugin, Update},
    ecs::{message::MessageWriter, schedule::IntoScheduleConfigs, system::Query},
    state::condition::in_state,
};
use leafwing_input_manager::prelude::ActionState;
use thetawave_core::{AppState, ToggleGameStateEvent};
use thetawave_player::{PlayerAction, PlayerNum};

pub(crate) struct ThetawaveStatesPlugin;

impl Plugin for ThetawaveStatesPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(thetawave_core::ThetawaveStatesPlugin)
            .add_systems(
                Update,
                toggle_game_state_system.run_if(in_state(AppState::Game)),
            );
    }
}

/// System for toggling the paused state only for player
fn toggle_game_state_system(
    player_action_q: Query<(&PlayerNum, &ActionState<PlayerAction>)>,
    mut toggle_game_state_event: MessageWriter<ToggleGameStateEvent>,
) {
    for (player_num, player_action) in player_action_q.iter() {
        if matches!(player_num, PlayerNum::One) && player_action.just_released(&PlayerAction::Pause)
        {
            toggle_game_state_event.write(ToggleGameStateEvent);
        }
    }
}
