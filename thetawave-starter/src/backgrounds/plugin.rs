use bevy::{app::Plugin, prelude::OnEnter};
use thetawave_core::AppState;

use crate::backgrounds::spawn::spawn_bg_system;

/// Plugin for managing background elements in Thetawave
pub struct ThetawaveBackgroundsPlugin;

impl Plugin for ThetawaveBackgroundsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(thetawave_backgrounds::ThetawaveBackgroundsPlugin)
            .add_systems(OnEnter(AppState::MainMenu), spawn_bg_system)
            .add_systems(OnEnter(AppState::Game), spawn_bg_system);
    }
}
