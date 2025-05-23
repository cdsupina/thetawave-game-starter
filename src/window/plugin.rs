use super::systems::{set_window_icon_system, setup_window_system, update_ui_scale_system};
use crate::states::{MainMenuState, PauseMenuState};
use bevy::{
    app::{Plugin, PostStartup, Startup, Update},
    ecs::schedule::{Condition, IntoScheduleConfigs},
    state::condition::in_state,
};

pub(crate) struct ThetawaveWindowPlugin;

impl Plugin for ThetawaveWindowPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, set_window_icon_system)
            .add_systems(
                PostStartup,
                (setup_window_system, update_ui_scale_system).chain(),
            )
            .add_systems(
                Update,
                update_ui_scale_system
                    .run_if(in_state(MainMenuState::Options).or(in_state(PauseMenuState::Options))),
            );
    }
}
