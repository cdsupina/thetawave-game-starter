use super::{data::ApplyOptionsEvent, OptionsRes};
use crate::audio::ChangeVolumeEvent;
use bevy::{
    core_pipeline::{bloom::Bloom, core_2d::Camera2d, core_3d::Camera3d},
    ecs::query::Without,
    prelude::{Commands, EventReader, EventWriter, Local, Query, Res, ResMut, With},
    window::{PrimaryWindow, Window, WindowMode},
};
use bevy_persistent::{Persistent, StorageFormat};
use std::path::Path;

/// Setup OptionsRes as a persistent resource
pub(super) fn setup_options_res(mut cmds: Commands) {
    //let config_dir = dirs::config_dir().unwrap().join("thetawave_game_starter");
    let config_dir = dirs::config_dir()
        .map(|native_config_dir| native_config_dir.join("thetawave_game_starter"))
        .unwrap_or(Path::new("local").join("configuration"));
    cmds.insert_resource(
        Persistent::<OptionsRes>::builder()
            .name("options")
            .format(StorageFormat::Toml)
            .path(config_dir.join("options.toml"))
            .default(OptionsRes::default())
            .build()
            .expect("failed to initialize options"),
    )
}

/// Initializes the options resource with values from the primary window
/// Updates window mode and resolution settings based on current window state
pub(super) fn sync_options_res_system(
    mut options_res: ResMut<Persistent<OptionsRes>>,
    primary_window_q: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(window) = primary_window_q.get_single() {
        options_res.window_mode = window.mode;
        options_res.window_resolution = window.resolution.clone();
    }
}

/// Applies window options when an ApplyOptionsEvent is received
pub(super) fn apply_options_system(
    mut apply_options_events: EventReader<ApplyOptionsEvent>,
    mut options_res: ResMut<Persistent<OptionsRes>>,
    mut camera_2d_q: Query<&mut Bloom, (With<Camera2d>, Without<Camera3d>)>,
    mut camera_3d_q: Query<&mut Bloom, (With<Camera3d>, Without<Camera2d>)>,
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

        // Enable or disable bloom
        if options_res.bloom_enabled {
            if let Ok(mut bloom) = camera_2d_q.get_single_mut() {
                bloom.intensity = Bloom::OLD_SCHOOL.intensity;
            }
            if let Ok(mut bloom) = camera_3d_q.get_single_mut() {
                bloom.intensity = Bloom::OLD_SCHOOL.intensity;
            }
        } else {
            if let Ok(mut bloom) = camera_2d_q.get_single_mut() {
                bloom.intensity = 0.0;
            }
            if let Ok(mut bloom) = camera_3d_q.get_single_mut() {
                bloom.intensity = 0.0
            }
        }

        // Clear the event channel to prevent processing same events multiple times
        apply_options_events.clear();
    }
}

/// Applies volume from OptionsRes to all audio channels
/// Applies instantly when changed rather than, when the apply button is pressed
pub(super) fn apply_volume_options_system(
    options_res: Res<Persistent<OptionsRes>>,
    mut event_writer: EventWriter<ChangeVolumeEvent>,
    mut previous_options_res: Local<OptionsRes>,
) {
    // Check if any of the volume options have changed since the previous frame
    if (options_res.master_volume != previous_options_res.master_volume)
        || (options_res.music_volume != previous_options_res.music_volume)
        || (options_res.effects_volume != previous_options_res.effects_volume)
        || (options_res.ui_volume != previous_options_res.ui_volume)
    {
        // Send event to change volumes of all audio channels
        event_writer.send(ChangeVolumeEvent {
            music_volume: options_res.master_volume * options_res.music_volume,
            effects_volume: options_res.master_volume * options_res.effects_volume,
            ui_volume: options_res.master_volume * options_res.ui_volume,
        });

        // Save the OptionsRes to a file
        options_res.persist().expect("failed to save new options");
    }

    // Save OptionsRes from this frame to local variable
    *previous_options_res = options_res.clone();
}
