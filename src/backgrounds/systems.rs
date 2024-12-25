use super::data::PlanetRotationComponent;
use crate::assets::BackgroundAssets;
use bevy::{
    asset::Assets,
    color::{Alpha, Color},
    math::Vec3,
    pbr::{MeshMaterial3d, PointLight, StandardMaterial},
    prelude::{
        AlphaMode, BuildChildren, ChildBuild, ChildBuilder, Commands, Mesh, Mesh3d, Meshable,
        Query, Rectangle, Res, ResMut, Sphere, Transform, Visibility,
    },
    scene::SceneRoot,
    time::Time,
    utils::default,
};
use rand::{rngs::ThreadRng, Rng};

/// Function to spawn background elements in the game world, including a space background,
/// a random planet, and star clusters
pub(super) fn spawn_bg_system(
    mut cmds: Commands,
    bg_assets: Res<BackgroundAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create a semi-transparent material with a random space background texture
    // that will serve as the backdrop
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(bg_assets.get_random_space_bg()),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        base_color: Color::default().with_alpha(0.1),
        ..default()
    });

    // Spawn a large rectangle mesh that will display the space background texture
    cmds.spawn((
        Mesh3d(meshes.add(Rectangle::default())),
        MeshMaterial3d(material_handle),
        Transform::default()
            .with_scale(Vec3::splat(250.0))
            .with_translation(Vec3::new(0.0, 0.0, -100.0)),
    ));

    // Initialize random number generator for positioning elements
    let mut rng = rand::thread_rng();

    // Calculate random X position for planet, ensuring it's not in the center
    let planet_x = if rng.gen_bool(0.5) {
        rng.gen_range(-12.0..=-5.0)
    } else {
        rng.gen_range(5.0..=12.0)
    };

    // Calculate random Y position for planet, ensuring it's not in the center
    let planet_y = if rng.gen_bool(0.5) {
        rng.gen_range(-8.0..=-2.0)
    } else {
        rng.gen_range(2.0..=8.0)
    };

    // Spawn a random planet model with rotation behavior
    cmds.spawn((
        SceneRoot(bg_assets.get_random_planet()),
        Transform::default().with_translation(Vec3::new(
            planet_x,
            planet_y,
            rng.gen_range(-50.0..=-20.0),
        )),
        PlanetRotationComponent {
            rotation_speed: rng.gen_range(0.01..=0.03),
        },
    ));

    // Calculate star position on opposite side of screen from planet
    let star_x = if planet_x > 0.0 {
        rng.gen_range(-18.0..=-5.0)
    } else {
        rng.gen_range(5.0..=18.0)
    };

    let star_y = if planet_y > 0.0 {
        rng.gen_range(-8.0..=-2.0)
    } else {
        rng.gen_range(2.0..=8.0)
    };

    // Spawn star cluster with potential additional stars
    cmds.spawn((
        Transform::from_xyz(star_x, star_y, rng.gen_range(-80.0..=-40.0)),
        Visibility::default(),
    ))
    .with_children(|parent| {
        // Spawn central star
        spawn_star(
            parent,
            Vec3::new(0., 0., 0.),
            &mut materials,
            &mut meshes,
            &mut rng,
        );

        // 15% chance to spawn additional star to the upper right
        if rng.gen_bool(0.15) {
            spawn_star(
                parent,
                Vec3::new(
                    rng.gen_range(6.0..=15.0),
                    rng.gen_range(6.0..=15.0),
                    rng.gen_range(-15.0..=15.0),
                ),
                &mut materials,
                &mut meshes,
                &mut rng,
            );
        }

        // 15% chance to spawn additional star to the lower left
        if rng.gen_bool(0.15) {
            spawn_star(
                parent,
                Vec3::new(
                    rng.gen_range(-15.0..=-6.0),
                    rng.gen_range(-15.0..=-6.0),
                    rng.gen_range(-15.0..=15.0),
                ),
                &mut materials,
                &mut meshes,
                &mut rng,
            );
        }
    });
}

/// Helper function to spawn an individual star with random properties
fn spawn_star(
    cb: &mut ChildBuilder,
    pos: Vec3,
    materials: &mut Assets<StandardMaterial>,
    meshes: &mut Assets<Mesh>,
    rng: &mut ThreadRng,
) {
    // Generate random bright color for the star
    let star_color = Color::srgb(
        rng.gen_range(0.0..=1.0),
        rng.gen_range(0.0..=1.0),
        rng.gen_range(0.0..=1.0),
    );

    // Spawn star mesh with emissive material
    cb.spawn((
        Mesh3d(meshes.add(Sphere::new(rng.gen_range(1.0..=3.0)).mesh().uv(32, 18))),
        Transform::from_translation(pos),
        Visibility::default(),
        MeshMaterial3d(materials.add(StandardMaterial {
            emissive: star_color.into(),
            diffuse_transmission: 1.0,
            ..default()
        })),
    ))
    // Add point light to make star glow
    .with_child(PointLight {
        color: star_color,
        intensity: rng.gen_range(8000000.0..=50000000.0),
        range: 100.0,
        ..default()
    });
}

/// System to handle planet rotation animation
pub(super) fn rotate_planet_system(
    mut planet_q: Query<(&mut Transform, &PlanetRotationComponent)>,
    time: Res<Time>,
) {
    for (mut transform, planet_rotation) in planet_q.iter_mut() {
        transform.rotate_y(planet_rotation.rotation_speed * time.delta_secs());
    }
}
