use super::{data::PlayerAction, systems::spawn_players_system};
use crate::states::AppState;
use bevy::{app::Plugin, prelude::OnEnter};
use leafwing_input_manager::plugin::InputManagerPlugin;

/// Plugin for managing player entities
pub(crate) struct ThetawavePlayerPlugin;

impl Plugin for ThetawavePlayerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default())
            .add_systems(OnEnter(AppState::Game), spawn_players_system);
    }
}
