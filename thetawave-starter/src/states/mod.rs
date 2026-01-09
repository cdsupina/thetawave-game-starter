use bevy::{
    app::{Plugin, Update},
    ecs::{message::MessageWriter, schedule::IntoScheduleConfigs, system::Query},
    state::{condition::in_state, state::OnEnter},
};
use leafwing_input_manager::prelude::ActionState;
use thetawave_assets::AssetMergeSet;
use thetawave_core::{
    AppState, ToggleGameStateEvent, enter_playing_state_system, enter_title_menu_state_system,
};
use thetawave_player::{PlayerAction, PlayerNum};

pub(crate) struct ThetawaveStatesPlugin;

impl Plugin for ThetawaveStatesPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(thetawave_core::ThetawaveStatesPlugin)
            // Register state transition systems after assets are merged
            .add_systems(
                OnEnter(AppState::MainMenu),
                enter_title_menu_state_system.after(AssetMergeSet),
            )
            .add_systems(
                OnEnter(AppState::Game),
                enter_playing_state_system.after(AssetMergeSet),
            )
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
