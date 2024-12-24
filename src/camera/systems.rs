use bevy::{
    color::Color,
    core_pipeline::{bloom::Bloom, tonemapping::Tonemapping},
    math::Vec3,
    prelude::{
        Camera, Camera3d, ClearColorConfig, Commands, PerspectiveProjection, Projection, Transform,
    },
    utils::default,
};

// Setup function that spawns a 2D camera
pub(super) fn setup_cameras_system(mut cmd: Commands) {
    // Necessary for viewing 2d sprites
    // Both cameras can view UI indepently
    /*
    cmd.spawn((
        Camera2d,
        Camera {
            order: 1,
            hdr: true,
            clear_color: ClearColorConfig::Custom(Color::NONE),
            ..default()
        },
        Tonemapping::TonyMcMapface,
        Bloom {
            prefilter: BloomPrefilter {
                threshold: 1.0,
                threshold_softness: 0.2,
            },
            ..Bloom::OLD_SCHOOL
        },
    ));
    */

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
        Bloom::NATURAL,
    ));
}
