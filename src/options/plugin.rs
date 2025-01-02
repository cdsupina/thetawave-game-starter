use bevy::{
    app::{Plugin, Update},
    prelude::{in_state, Condition, IntoSystemConfigs, OnEnter},
};

use crate::states::{MainMenuState, PauseMenuState};

use super::{
    data::OptionsRes,
    systems::{apply_options_system, sync_options_res_system, update_ui_scale_system},
    ApplyOptionsEvent,
};

// Plugin struct for handling Thetawave game options
pub(crate) struct ThetawaveOptionsPlugin;

impl Plugin for ThetawaveOptionsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // Initialize options resource
        app.init_resource::<OptionsRes>();
        // Add event for applying options changes
        app.add_event::<ApplyOptionsEvent>();
        // Init the options menu to track the current options on startup
        app.add_systems(OnEnter(MainMenuState::Options), sync_options_res_system);

        // Add system to apply options changes, but only when in Options menu state
        app.add_systems(
            Update,
            (
                apply_options_system
                    .run_if(in_state(MainMenuState::Options).or(in_state(PauseMenuState::Options))),
                update_ui_scale_system
                    .run_if(in_state(MainMenuState::Options).or(in_state(PauseMenuState::Options))),
            ),
        );
    }
}
