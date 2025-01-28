use bevy::app::{Plugin, Startup};

use super::systems::set_window_icon_system;

pub(crate) struct ThetawaveWindowPlugin;

impl Plugin for ThetawaveWindowPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, set_window_icon_system);
    }
}
