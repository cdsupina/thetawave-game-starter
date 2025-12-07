use bevy::{
    camera::ScalingMode,
    color::Color,
    core_pipeline::tonemapping::Tonemapping,
    math::Vec3,
    post_process::bloom::Bloom,
    prelude::{
        Camera, Camera2d, Camera3d, ClearColorConfig, Commands, Name, OrthographicProjection,
        PerspectiveProjection, Projection, Transform,
    },
    render::view::Hdr,
    utils::default,
};
use bevy_egui::PrimaryEguiContext;

const VIEWPORT_HEIGHT: f32 = 250.0;

// Setup function that spawns a 2D camera
pub(super) fn setup_cameras_system(mut cmd: Commands) {
    // Necessary for viewing 2d sprites
    cmd.spawn((
        Camera2d,
        PrimaryEguiContext,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::Custom(Color::NONE),
            ..default()
        },
        Hdr,
        Tonemapping::TonyMcMapface,
        Bloom::default(),
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
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        Hdr,
        Tonemapping::BlenderFilmic,
        Bloom::default(),
        Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        Projection::Perspective(PerspectiveProjection {
            far: 10000.0,
            ..Default::default()
        }),
        Name::new("3D Camera"),
    ));
}
