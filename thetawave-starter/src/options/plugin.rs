use super::{
    systems::{
        apply_options_system, apply_volume_options_system, setup_options_res,
        sync_options_res_system,
    },
    ApplyOptionsEvent,
};
use bevy::{
    app::{Plugin, Startup, Update},
    prelude::{in_state, Condition, IntoScheduleConfigs, OnEnter},
};
use thetawave_core::{MainMenuState, PauseMenuState};

// Plugin struct for handling Thetawave game options
pub(crate) struct ThetawaveOptionsPlugin;

impl Plugin for ThetawaveOptionsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // Add event for applying options changes
        app.add_event::<ApplyOptionsEvent>()
            // Add system to apply options changes, but only when in Options menu state
            .add_systems(Startup, setup_options_res)
            // Init the options menu to track the current options on startup
            .add_systems(OnEnter(MainMenuState::Options), sync_options_res_system)
            .add_systems(
                Update,
                (apply_options_system, apply_volume_options_system)
                    .run_if(in_state(MainMenuState::Options).or(in_state(PauseMenuState::Options))),
            );
    }
}
