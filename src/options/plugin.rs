use bevy::{
    app::{Plugin, Startup, Update},
    prelude::{in_state, Condition, IntoSystemConfigs, OnEnter},
};

use crate::states::{MainMenuState, PauseMenuState};

use super::{
    systems::{
        apply_volume_options_system, apply_window_options_system, setup_options_res,
        sync_options_res_system, update_ui_scale_system,
    },
    ApplyOptionsEvent,
};

// Plugin struct for handling Thetawave game options
pub(crate) struct ThetawaveOptionsPlugin;

impl Plugin for ThetawaveOptionsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // Add event for applying options changes
        app.add_event::<ApplyOptionsEvent>()
            // Init the options menu to track the current options on startup
            .add_systems(OnEnter(MainMenuState::Options), sync_options_res_system)
            // Add system to apply options changes, but only when in Options menu state
            .add_systems(Startup, setup_options_res)
            .add_systems(
                Update,
                (
                    apply_window_options_system,
                    apply_volume_options_system,
                    update_ui_scale_system,
                )
                    .run_if(in_state(MainMenuState::Options).or(in_state(PauseMenuState::Options))),
            );
    }
}
