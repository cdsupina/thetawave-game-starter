use super::systems::{button_system, print_nav_events, setup};
use crate::states::AppState;
use bevy::{
    app::{Plugin, Update},
    prelude::{IntoSystemConfigs, OnEnter},
};
use bevy_alt_ui_navigation_lite::NavRequestSystem;
use bevy_hui::HuiPlugin;

// Plugin for managing the Thetawave UI
pub(crate) struct ThetawaveUiPlugin;

impl Plugin for ThetawaveUiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // Add HuiPlugin and HuiAutoLoadPlugin with UI components path
        app.add_plugins(HuiPlugin);

        // Initialize UI setup when asset loading is finished
        app.add_systems(OnEnter(AppState::MainMenu), setup);

        // Add UI systems that run after navigation system:
        // - Button system for handling button interactions
        // - Print system for logging navigation events
        app.add_systems(
            Update,
            (
                button_system.after(NavRequestSystem),
                print_nav_events.after(NavRequestSystem),
            ),
        );
    }
}
