use crate::camera::{
    Camera2DZoomEvent, Camera3DZoomEvent,
    systems::{change_camera2d_zoom_system, change_camera3d_zoom_system},
};

use super::systems::setup_cameras_system;
use bevy::app::{Plugin, PostStartup, Update};

/// A plugin for managing the Thetawave game's camera systems
pub(crate) struct ThetawaveCameraPlugin;

impl Plugin for ThetawaveCameraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<Camera2DZoomEvent>()
            .add_event::<Camera3DZoomEvent>();

        app.add_systems(PostStartup, setup_cameras_system)
            .add_systems(
                Update,
                (change_camera2d_zoom_system, change_camera3d_zoom_system),
            );
    }
}
