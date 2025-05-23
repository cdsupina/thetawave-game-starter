use crate::options::OptionsRes;
use bevy::{
    color::Color,
    core_pipeline::{bloom::Bloom, tonemapping::Tonemapping},
    ecs::system::Res,
    math::Vec3,
    prelude::{
        Camera, Camera2d, Camera3d, ClearColorConfig, Commands, Name, OrthographicProjection,
        PerspectiveProjection, Projection, Transform,
    },
    render::camera::ScalingMode,
    utils::default,
};
use bevy_persistent::Persistent;

const VIEWPORT_HEIGHT: f32 = 250.0;

// Setup function that spawns a 2D camera
pub(super) fn setup_cameras_system(mut cmd: Commands, options_res: Res<Persistent<OptionsRes>>) {
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
