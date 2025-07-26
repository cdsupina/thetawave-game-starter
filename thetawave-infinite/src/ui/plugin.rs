use bevy::{app::Plugin, ecs::schedule::IntoScheduleConfigs, state::condition::in_state};
use bevy_egui::EguiPrimaryContextPass;
use thetawave_starter::{ui::update_egui_scale_system, DebugState};

use crate::ui::systems::game_debug::game_debug_menu_system;

pub(crate) struct ThetawaveInfiniteUiPlugin;

impl Plugin for ThetawaveInfiniteUiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            EguiPrimaryContextPass,
            (game_debug_menu_system, update_egui_scale_system).run_if(in_state(DebugState::Debug)),
        );
    }
}
