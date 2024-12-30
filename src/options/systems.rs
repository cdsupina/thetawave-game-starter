use bevy::{
    prelude::{EventReader, Query, ResMut, With},
    window::{PrimaryWindow, Window, WindowMode},
};

use super::{data::ApplyOptionsEvent, OptionsRes};

/// Initializes the options resource with values from the primary window
/// Updates window mode and resolution settings based on current window state
pub(super) fn sync_options_res_system(
    mut options_res: ResMut<OptionsRes>,
    primary_window_q: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(window) = primary_window_q.get_single() {
        options_res.window_mode = window.mode;
        options_res.window_resolution = window.resolution.clone();
    }
}

/// System that applies window options when an ApplyOptionsEvent is received
/// Takes event reader for ApplyOptionsEvent, mutable access to OptionsRes,
/// and query for the primary window
pub(super) fn apply_options_system(
    mut apply_options_events: EventReader<ApplyOptionsEvent>,
    mut options_res: ResMut<OptionsRes>,
    mut primary_window_q: Query<&mut Window, With<PrimaryWindow>>,
) {
    // Only process if we have received events
    if !apply_options_events.is_empty() {
        // Try to get mutable reference to primary window
        if let Ok(mut window) = primary_window_q.get_single_mut() {
            // If fullscreen is selected, preserve the current resolution
            if matches!(options_res.window_mode, WindowMode::Fullscreen(_))
                && matches!(window.mode, WindowMode::Fullscreen(_))
            {
                options_res.window_resolution = window.resolution.clone();
            }

            // Apply the selected options to the window
            window.mode = options_res.window_mode;
            window.resolution = options_res
                .window_resolution
                .clone()
                .with_scale_factor_override(1.0);
        }

        // Clear the event channel to prevent processing same events multiple times
        apply_options_events.clear();
    }
}
