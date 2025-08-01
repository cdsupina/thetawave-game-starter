use crate::{camera::data::CameraZoomEvent, options::OptionsRes};
use bevy::{
    color::Color,
    core_pipeline::{bloom::Bloom, tonemapping::Tonemapping},
    ecs::{
        event::EventReader,
        query::With,
        system::{Query, Res},
    },
    math::Vec3,
    prelude::{
        Camera, Camera2d, Camera3d, ClearColorConfig, Commands, Name, OrthographicProjection,
        PerspectiveProjection, Projection, Transform,
    },
    render::camera::ScalingMode,
    utils::default,
};
use bevy_egui::PrimaryEguiContext;
use bevy_persistent::Persistent;

const VIEWPORT_HEIGHT: f32 = 250.0;
const MAX_2D_CAMERA_ZOOM_SCALE: f32 = 0.9;

// Setup function that spawns a 2D camera
pub(super) fn setup_cameras_system(mut cmd: Commands, options_res: Res<Persistent<OptionsRes>>) {
    // Necessary for viewing 2d sprites
    cmd.spawn((
        Camera2d,
        PrimaryEguiContext,
        Camera {
            order: 1,
            hdr: true,
            clear_color: ClearColorConfig::Custom(Color::NONE),
            ..default()
        },
        Tonemapping::TonyMcMapface,
        Bloom {
            intensity: if options_res.bloom_enabled {
                Bloom::OLD_SCHOOL.intensity
            } else {
                0.0
            },
            ..Bloom::OLD_SCHOOL
        },
        // Change OrthographicProjection component to Projection component
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: VIEWPORT_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));

    // Necessary for viewing 3D assets
    cmd.spawn((
        Camera3d::default(),
        Camera {
            order: 0,
            hdr: true,
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        Tonemapping::BlenderFilmic,
        Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        Projection::Perspective(PerspectiveProjection {
            far: 10000.0,
            ..Default::default()
        }),
        Bloom {
            intensity: if options_res.bloom_enabled {
                Bloom::OLD_SCHOOL.intensity
            } else {
                0.0
            },
            ..Bloom::OLD_SCHOOL
        },
        Name::new("3D Camera"),
    ));
}

/// Event for reading zoom events and updating the scale of the 2dCamera
pub(super) fn change_camera2d_zoom_system(
    mut zoom_events: EventReader<CameraZoomEvent>,
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
