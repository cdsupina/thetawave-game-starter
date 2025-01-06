use bevy::{
    color::Color,
    core::Name,
    core_pipeline::{bloom::Bloom, tonemapping::Tonemapping},
    math::Vec3,
    prelude::{
        Camera, Camera2d, Camera3d, ClearColorConfig, Commands, OrthographicProjection,
        PerspectiveProjection, Projection, Transform,
    },
    render::camera::ScalingMode,
    utils::default,
};

const VIEWPORT_HEIGHT: f32 = 250.0;

// Setup function that spawns a 2D camera
pub(super) fn setup_cameras_system(mut cmd: Commands) {
    // Necessary for viewing 2d sprites
    cmd.spawn((
        Camera2d,
        Camera {
            order: 1,
            hdr: true,
            clear_color: ClearColorConfig::Custom(Color::NONE),
            ..default()
        },
        Tonemapping::TonyMcMapface,
        Bloom::OLD_SCHOOL,
        OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: VIEWPORT_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        },
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
        Tonemapping::TonyMcMapface,
        Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        Projection::Perspective(PerspectiveProjection {
            far: 10000.0,
            ..Default::default()
        }),
        Bloom::OLD_SCHOOL,
        Name::new("3D Camera"),
    ));
}
