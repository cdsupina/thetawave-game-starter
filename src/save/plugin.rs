use super::systems::{increment_run_count_system, increment_win_loss_count_system, setup_save_res};
use crate::states::{AppState, GameState};
use bevy::{
    app::{Plugin, Startup},
    state::state::OnEnter,
};

/// Plugin for managing player save files and save data
pub(crate) struct ThetawaveSavePlugin;

impl Plugin for ThetawaveSavePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, setup_save_res)
            .add_systems(OnEnter(AppState::Game), increment_run_count_system)
            .add_systems(OnEnter(GameState::End), increment_win_loss_count_system);
    }
}
