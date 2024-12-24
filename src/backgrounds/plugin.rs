use super::systems::spawn_bg_system;
use crate::states::AppState;
use bevy::{app::Plugin, prelude::OnEnter};

/// Plugin for managing background elements in Thetawave
pub(crate) struct ThetawaveBackgroundsPlugin;

impl Plugin for ThetawaveBackgroundsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(OnEnter(AppState::MainMenu), spawn_bg_system);
    }
}
