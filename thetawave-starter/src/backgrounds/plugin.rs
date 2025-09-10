use bevy::{
    app::{Plugin, Update},
    ecs::schedule::common_conditions::not,
    prelude::{Condition, IntoScheduleConfigs, OnEnter, in_state},
};
use thetawave_core::{AppState, GameState};

use crate::backgrounds::{spawn::spawn_bg_system, systems::rotate_planet_system};

/// Plugin for managing background elements in Thetawave
pub struct ThetawaveBackgroundsPlugin;

impl Plugin for ThetawaveBackgroundsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(OnEnter(AppState::MainMenu), spawn_bg_system)
            .add_systems(OnEnter(AppState::Game), spawn_bg_system)
            .add_systems(
                Update,
                rotate_planet_system
                    // rotate the planets if the game is not paused or the app is in the main menu state
                    .run_if(not(in_state(GameState::Paused)).or(in_state(AppState::MainMenu))),
            );
    }
}
