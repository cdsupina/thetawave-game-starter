use super::{
    data::OptionsRes,
    systems::{
        button_system, init_options_res_system, options_menu_system, print_nav_events,
        setup_main_menu_system, setup_options_menu_system, setup_ui_system,
    },
};
use crate::states::AppState;
use bevy::{
    app::{Plugin, Startup, Update},
    prelude::{in_state, IntoSystemConfigs, OnEnter},
};
use bevy_alt_ui_navigation_lite::NavRequestSystem;
use bevy_egui::EguiPlugin;
use bevy_hui::HuiPlugin;

// Plugin for managing the Thetawave UI
pub(crate) struct ThetawaveUiPlugin;

impl Plugin for ThetawaveUiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<OptionsRes>();

        // Add HuiPlugin and HuiAutoLoadPlugin with UI components path
        app.add_plugins((HuiPlugin, EguiPlugin));

        // Init the options menu to track the current options on startup
        app.add_systems(Startup, init_options_res_system);

        app.add_systems(
            OnEnter(AppState::MainMenu),
            (setup_ui_system, setup_main_menu_system).chain(),
        );
        app.add_systems(OnEnter(AppState::OptionsMenu), setup_options_menu_system);

        // Add UI systems that run after navigation system:
        // - Button system for handling button interactions
        // - Print system for logging navigation events
        app.add_systems(
            Update,
            (
                button_system.after(NavRequestSystem),
                print_nav_events.after(NavRequestSystem),
                options_menu_system.run_if(in_state(AppState::OptionsMenu)),
            ),
        );
    }
}
