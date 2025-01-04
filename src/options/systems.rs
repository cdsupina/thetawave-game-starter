use bevy::{
    prelude::{EventReader, EventWriter, Query, Res, ResMut, With},
    ui::UiScale,
    window::{PrimaryWindow, Window, WindowMode},
};

use crate::audio::ChangeVolumeEvent;

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

/// Applies window options when an ApplyOptionsEvent is received
pub(super) fn apply_window_options_system(
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

/// Applies volume options when an ApplyOptionsEvent is received
pub(super) fn apply_volume_options_system(
    mut apply_options_events: EventReader<ApplyOptionsEvent>,
    options_res: Res<OptionsRes>,
    mut event_writer: EventWriter<ChangeVolumeEvent>,
) {
    // Only process if we have received events
    if !apply_options_events.is_empty() {
        event_writer.send(ChangeVolumeEvent {
            music_volume: options_res.master_volume * options_res.music_volume,
            effects_volume: options_res.master_volume * options_res.effects_volume,
            ui_volume: options_res.master_volume * options_res.ui_volume,
        });

        // Clear the event channel to prevent processing same events multiple times
        apply_options_events.clear();
    }
}

/// System that updates UI scale based on window height
pub(super) fn update_ui_scale_system(
    mut ui_scale: ResMut<UiScale>,
    primary_window_q: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(window) = primary_window_q.get_single() {
        // Calculate UI scale based on physical window height relative to 720p baseline
        ui_scale.0 = (1. / 720.) * (window.resolution.physical_height() as f32);
    }
}
