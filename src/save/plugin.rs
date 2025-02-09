use super::systems::setup_save_res;
use bevy::app::{Plugin, Startup};

/// Plugin for managing player save files and save data
pub(crate) struct ThetawaveSavePlugin;

impl Plugin for ThetawaveSavePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, setup_save_res);
    }
}
