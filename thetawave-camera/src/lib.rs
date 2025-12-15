use bevy::app::{Plugin, Update};

use crate::systems::{change_camera2d_zoom_system, change_camera3d_zoom_system};

pub use data::{Camera2DZoomEvent, Camera3DZoomEvent};

mod data;
mod systems;

/// A plugin for managing the Thetawave game's camera systems
pub struct ThetawaveCameraPlugin;

impl Plugin for ThetawaveCameraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_message::<Camera2DZoomEvent>()
            .add_message::<Camera3DZoomEvent>();

        app.add_systems(
            Update,
            (change_camera2d_zoom_system, change_camera3d_zoom_system),
        );
    }
}
