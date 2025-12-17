use bevy::{
    app::{Plugin, Update},
    ecs::schedule::{common_conditions::on_message, IntoScheduleConfigs},
    state::condition::in_state,
};
use thetawave_core::AppState;

use super::{
    behavior::collect_behavior_tree_display_system,
    camera::{mob_view_camera_follow_system, mob_view_camera_zoom_system},
    data::{BehaviorTreeDisplays, MobGroupDisplayStats, MobGroupRegistry, MobViewWindowState},
    groups::update_mob_groups_system,
    selection::{handle_cycle_mob_selection, tab_cycle_mob_system, CycleMobSelectionEvent},
    stats::collect_mob_stats_system,
    ui::mob_view_ui_system,
    window::{
        handle_mob_view_window_close, toggle_mob_view_window, MobViewContextPass,
        ToggleMobViewWindowEvent,
    },
};

pub struct MobViewPlugin;

impl Plugin for MobViewPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // Resources
        app.init_resource::<MobGroupRegistry>()
            .init_resource::<MobViewWindowState>()
            .init_resource::<MobGroupDisplayStats>()
            .init_resource::<BehaviorTreeDisplays>();

        // Messages
        app.add_message::<CycleMobSelectionEvent>()
            .add_message::<ToggleMobViewWindowEvent>();

        // Window management - always runs
        app.add_systems(
            Update,
            (
                toggle_mob_view_window.run_if(on_message::<ToggleMobViewWindowEvent>),
                handle_mob_view_window_close,
            ),
        );

        // Core systems - run when window is open and in game
        app.add_systems(
            Update,
            (
                update_mob_groups_system,
                tab_cycle_mob_system,
                handle_cycle_mob_selection.run_if(on_message::<CycleMobSelectionEvent>),
                collect_mob_stats_system,
                collect_behavior_tree_display_system,
                mob_view_camera_follow_system,
                mob_view_camera_zoom_system,
            )
                .chain()
                .run_if(|state: bevy::ecs::system::Res<MobViewWindowState>| state.is_open)
                .run_if(in_state(AppState::Game)),
        );

        // UI system runs in the mob view window's egui context pass
        app.add_systems(MobViewContextPass, mob_view_ui_system);
    }
}
