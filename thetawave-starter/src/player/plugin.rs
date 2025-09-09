use super::systems::spawn_players_system;
use bevy::{app::Plugin, prelude::OnEnter};
use thetawave_core::AppState;

/// Plugin for managing player entities
pub(crate) struct ThetawavePlayerPlugin;

impl Plugin for ThetawavePlayerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(thetawave_player::ThetawavePlayerPlugin)
            .add_systems(OnEnter(AppState::Game), spawn_players_system);
    }
}
