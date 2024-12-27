use bevy::{
    app::{Plugin, Startup, Update},
    prelude::{in_state, IntoSystemConfigs},
};

use crate::states::MainMenuState;

use super::{
    data::OptionsRes,
    systems::{apply_options_system, init_options_res_system},
    ApplyOptionsEvent,
};

pub(crate) struct ThetawaveOptionsPlugin;

impl Plugin for ThetawaveOptionsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<OptionsRes>();
        app.add_event::<ApplyOptionsEvent>();
        // Init the options menu to track the current options on startup
        app.add_systems(Startup, init_options_res_system);

        app.add_systems(
            Update,
            apply_options_system.run_if(in_state(MainMenuState::Options)),
        );
    }
}
