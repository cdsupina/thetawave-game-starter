use bevy::{
    ecs::{
        query::With,
        system::{Query, Res, ResMut},
    },
    log::warn,
    prelude::NonSend,
    ui::UiScale,
    window::{PrimaryWindow, Window},
    winit::WinitWindows,
};
use bevy_persistent::Persistent;
use winit::window::Icon;

use crate::options::OptionsRes;

/// Set the image for the window icon
pub(super) fn set_window_icon_system(
    // we have to use `NonSend` here - wrap in Option since it may not be available yet
    windows: Option<NonSend<WinitWindows>>,
) {
    let Some(windows) = windows else {
        return;
    };
    // Try multiple possible locations for the window icon
    let possible_paths = [
        "assets/window_icon.png",    // When running from workspace root
        "../assets/window_icon.png", // When running from test-game directory
    ];

    let mut icon_path = None;
    for path in &possible_paths {
        if std::path::Path::new(path).exists() {
            icon_path = Some(path);
            break;
        }
    }

    let icon_path = match icon_path {
        Some(path) => path,
        None => {
            warn!(
                "Window icon not found at any expected location: {:?}",
                possible_paths
            );
            return; // Skip setting icon if not found
        }
    };

    // here we use the `image` crate to load our icon data from a png file
    // this is not a very bevy-native solution, but it will do
    let (icon_rgba, icon_width, icon_height) = {
        let image = match image::open(icon_path) {
            Ok(img) => img,
            Err(e) => {
                warn!("Failed to open icon at {}: {}", icon_path, e);
                return; // Skip setting icon if loading fails
            }
        }
        .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height)
        .expect("Failed to create window icon from RGBA data");

    // do it for all windows
    for window in windows.windows.values() {
        window.set_window_icon(Some(icon.clone()));
    }
}

// Sets up the window with the window options in OptionsRes
pub(super) fn setup_window_system(
    options_res: Res<Persistent<OptionsRes>>,
    mut primary_window_q: Query<&mut Window, With<PrimaryWindow>>,
) {
    // Try to get mutable reference to primary window
    if let Ok(mut window) = primary_window_q.single_mut() {
        // Apply the selected options to the window
        window.mode = options_res.window_mode;
        window.resolution = options_res
            .window_resolution
            .clone()
            .with_scale_factor_override(1.0);
    }
}

/// System that updates UI scale based on window height
pub(super) fn update_ui_scale_system(
    mut ui_scale: ResMut<UiScale>,
    primary_window_q: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(window) = primary_window_q.single() {
        // Calculate UI scale based on physical window height relative to 720p baseline
        ui_scale.0 = (1. / 720.) * (window.resolution.physical_height() as f32);
    }
}
