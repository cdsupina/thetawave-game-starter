use bevy::{
    core_pipeline::{core_2d::Camera2d, core_3d::Camera3d},
    ecs::{event::EventReader, query::With, system::Query},
    transform::components::Transform,
};

use crate::data::{Camera2DZoomEvent, Camera3DZoomEvent};

const MAX_2D_CAMERA_ZOOM_SCALE: f32 = 0.9;
const MAX_3D_CAMERA_ZOOM_SCALE: f32 = 80.0;

/// Event for reading zoom events and updating the scale of the 2dCamera
pub(super) fn change_camera2d_zoom_system(
    mut zoom_events: EventReader<Camera2DZoomEvent>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    for event in zoom_events.read() {
        if let Ok(mut transform) = camera_query.single_mut() {
            // Map event value (-100 to 100) to zoom scale
            let zoom_scale = 1.0 + (event.0 as f32 / 100.0) * MAX_2D_CAMERA_ZOOM_SCALE;

            transform.scale.x = zoom_scale;
            transform.scale.y = zoom_scale;
        }
    }
}

/// Event for reading zoom events and updating the translation of the 3dCamera
pub(super) fn change_camera3d_zoom_system(
    mut zoom_events: EventReader<Camera3DZoomEvent>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    for event in zoom_events.read() {
        if let Ok(mut transform) = camera_query.single_mut() {
            // Map event value (-100 to 100) to zoom scale
            let zoom_scale = (event.0 as f32 / 100.0) * MAX_3D_CAMERA_ZOOM_SCALE;

            transform.translation.z = zoom_scale;
        }
    }
}
