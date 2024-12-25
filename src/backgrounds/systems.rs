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
use std::ops::RangeInclusive;

// Background properties
const BACKGROUND_ALPHA: f32 = 0.1; // Opacity of the background
const BACKGROUND_SCALE: f32 = 250.0; // Scale factor for the background mesh
const BACKGROUND_POS: Vec3 = Vec3::new(0.0, 0.0, -100.0); // Position of the background in 3D space

// Planet position ranges
const PLANET_LOWER_X_RANGE: RangeInclusive<f32> = -12.0..=-5.0; // Left side X coordinate range
const PLANET_UPPER_X_RANGE: RangeInclusive<f32> = 5.0..=12.0; // Right side X coordinate range
const PLANET_LOWER_Y_RANGE: RangeInclusive<f32> = -8.0..=-2.0; // Bottom side Y coordinate range
const PLANET_UPPER_Y_RANGE: RangeInclusive<f32> = 2.0..=8.0; // Top side Y coordinate range
const PLANET_Z_RANGE: RangeInclusive<f32> = -50.0..=-20.0; // Depth range for planets
const PLANET_ROTATION_SPEED_RANGE: RangeInclusive<f32> = 0.01..=0.03; // Range of rotation speeds

// Star cluster position ranges
const STAR_CLUSTER_LOWER_X_RANGE: RangeInclusive<f32> = -18.0..=-5.0; // Left side X coordinate range for clusters
const STAR_CLUSTER_UPPER_X_RANGE: RangeInclusive<f32> = 5.0..=18.0; // Right side X coordinate range for clusters
const STAR_CLUSTER_LOWER_Y_RANGE: RangeInclusive<f32> = -8.0..=-2.0; // Bottom side Y coordinate range for clusters
const STAR_CLUSTER_UPPER_Y_RANGE: RangeInclusive<f32> = 2.0..=8.0; // Top side Y coordinate range for clusters
const STAR_CLUSTER_Z_RANGE: RangeInclusive<f32> = -80.0..=-40.0; // Depth range for star clusters
const ADDITIONAL_STAR_CHANCE: f64 = 0.15; // Probability of spawning additional stars

// Individual star properties
const STAR_UPPER_RIGHT_X_RANGE: RangeInclusive<f32> = 6.0..=15.0; // X range for upper right additional stars
const STAR_UPPER_RIGHT_Y_RANGE: RangeInclusive<f32> = 6.0..=15.0; // Y range for upper right additional stars
const STAR_LOWER_LEFT_X_RANGE: RangeInclusive<f32> = -15.0..=6.0; // X range for lower left additional stars
const STAR_LOWER_LEFT_Y_RANGE: RangeInclusive<f32> = -15.0..=6.0; // Y range for lower left additional stars
const STAR_Z_RANGE: RangeInclusive<f32> = -15.0..=15.0; // Depth range for individual stars
const STAR_RADIUS_RANGE: RangeInclusive<f32> = 1.0..=3.0; // Range of star sizes
const STAR_POINT_LIGHT_INTENSITY_RANGE: RangeInclusive<f32> = 8000000.0..=50000000.0; // Range of star light intensities
const STAR_POINT_LIGHT_RANGE: f32 = 100.0; // Maximum distance of star light effect

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
        base_color: Color::default().with_alpha(BACKGROUND_ALPHA),
        ..default()
    });

    // Spawn a large rectangle mesh that will display the space background texture
    cmds.spawn((
        Mesh3d(meshes.add(Rectangle::default())),
        MeshMaterial3d(material_handle),
        Transform::default()
            .with_scale(Vec3::splat(BACKGROUND_SCALE))
            .with_translation(BACKGROUND_POS),
    ));

    // Initialize random number generator for positioning elements
    let mut rng = rand::thread_rng();

    // Calculate random X position for planet, ensuring it's not in the center
    let planet_x = if rng.gen_bool(0.5) {
        rng.gen_range(PLANET_LOWER_X_RANGE)
    } else {
        rng.gen_range(PLANET_UPPER_X_RANGE)
    };

    // Calculate random Y position for planet, ensuring it's not in the center
    let planet_y = if rng.gen_bool(0.5) {
        rng.gen_range(PLANET_LOWER_Y_RANGE)
    } else {
        rng.gen_range(PLANET_UPPER_Y_RANGE)
    };

    // Spawn a random planet model with rotation behavior
    cmds.spawn((
        SceneRoot(bg_assets.get_random_planet()),
        Transform::default().with_translation(Vec3::new(
            planet_x,
            planet_y,
            rng.gen_range(PLANET_Z_RANGE),
        )),
        PlanetRotationComponent::new(rng.gen_range(PLANET_ROTATION_SPEED_RANGE)),
    ));

    // Calculate star position on opposite side of screen from planet
    let star_x = if planet_x > 0.0 {
        rng.gen_range(STAR_CLUSTER_LOWER_X_RANGE)
    } else {
        rng.gen_range(STAR_CLUSTER_UPPER_X_RANGE)
    };

    let star_y = if planet_y > 0.0 {
        rng.gen_range(STAR_CLUSTER_LOWER_Y_RANGE)
    } else {
        rng.gen_range(STAR_CLUSTER_UPPER_Y_RANGE)
    };

    // Spawn star cluster with potential additional stars
    cmds.spawn((
        Transform::from_xyz(star_x, star_y, rng.gen_range(STAR_CLUSTER_Z_RANGE)),
        Visibility::default(),
    ))
    .with_children(|parent| {
        // Spawn central star
        spawn_star(parent, Vec3::ZERO, &mut materials, &mut meshes, &mut rng);

        // 15% chance to spawn additional star to the upper right
        if rng.gen_bool(ADDITIONAL_STAR_CHANCE) {
            spawn_star(
                parent,
                Vec3::new(
                    rng.gen_range(STAR_UPPER_RIGHT_X_RANGE),
                    rng.gen_range(STAR_UPPER_RIGHT_Y_RANGE),
                    rng.gen_range(STAR_Z_RANGE),
                ),
                &mut materials,
                &mut meshes,
                &mut rng,
            );
        }

        // 15% chance to spawn additional star to the lower left
        if rng.gen_bool(ADDITIONAL_STAR_CHANCE) {
            spawn_star(
                parent,
                Vec3::new(
                    rng.gen_range(STAR_LOWER_LEFT_X_RANGE),
                    rng.gen_range(STAR_LOWER_LEFT_Y_RANGE),
                    rng.gen_range(STAR_Z_RANGE),
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
    let star_color = Color::srgb(rng.gen::<f32>(), rng.gen::<f32>(), rng.gen::<f32>());

    // Spawn star mesh with emissive material
    cb.spawn((
        Mesh3d(
            meshes.add(
                Sphere::new(rng.gen_range(STAR_RADIUS_RANGE))
                    .mesh()
                    .uv(32, 18),
            ),
        ),
        Transform::from_translation(pos),
        Visibility::default(),
        MeshMaterial3d(materials.add(StandardMaterial {
            emissive: star_color.into(),
            diffuse_transmission: 1.0,
            ..default()
        })),
    ))
    // Add point light to make star glow with a random light intensity
    .with_child(PointLight {
        color: star_color,
        intensity: rng.gen_range(STAR_POINT_LIGHT_INTENSITY_RANGE),
        range: STAR_POINT_LIGHT_RANGE,
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
