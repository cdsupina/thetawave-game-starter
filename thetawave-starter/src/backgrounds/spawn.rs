use bevy::{
    asset::Assets,
    color::{Alpha, Color},
    ecs::error::Result,
    math::Vec3,
    pbr::{MeshMaterial3d, PointLight, StandardMaterial},
    prelude::{
        AlphaMode, ChildSpawnerCommands, Commands, Mesh, Mesh3d, Meshable, Name, Rectangle, Res,
        ResMut, Sphere, Transform, Visibility,
    },
    scene::SceneRoot,
    utils::default,
};
use rand::{Rng, rngs::ThreadRng};
use std::ops::RangeInclusive;
use thetawave_assets::{AssetResolver, BackgroundAssets, ExtendedBackgroundAssets};
use thetawave_backgrounds::PlanetRotationComponent;
use thetawave_core::{AppState, Cleanup};

// Background properties
const BACKGROUND_ALPHA: f32 = 0.1; // Opacity of the background
const BACKGROUND_SCALE: f32 = 250.0; // Scale factor for the background mesh
const BACKGROUND_POS: Vec3 = Vec3::new(0.0, 0.0, -100.0); // Position of the background in 3D space

// Planet position ranges
const PLANET_LOWER_X_RANGE: RangeInclusive<f32> = -12.0..=-6.0; // Left side X coordinate range
const PLANET_UPPER_X_RANGE: RangeInclusive<f32> = 6.0..=12.0; // Right side X coordinate range
const PLANET_LOWER_Y_RANGE: RangeInclusive<f32> = -8.0..=-4.0; // Bottom side Y coordinate range
const PLANET_UPPER_Y_RANGE: RangeInclusive<f32> = 4.0..=8.0; // Top side Y coordinate range
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
    extended_bg_assets: Res<ExtendedBackgroundAssets>,
    bg_assets: Res<BackgroundAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) -> Result {
    // Create a semi-transparent material with a random space background texture
    // that will serve as the backdrop
    let background_texture = AssetResolver::get_random_space_bg(&extended_bg_assets, &bg_assets)?;

    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(background_texture),
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
        Cleanup::<AppState> {
            states: vec![AppState::MainMenu, AppState::Game],
        },
        Name::new("Background Image"),
    ));

    // Initialize random number generator for positioning elements
    let mut rng = rand::rng();

    // Calculate random X position for planet, ensuring it's not in the center
    let planet_x = if rng.random_bool(0.5) {
        rng.random_range(PLANET_LOWER_X_RANGE)
    } else {
        rng.random_range(PLANET_UPPER_X_RANGE)
    };

    // Calculate random Y position for planet, ensuring it's not in the center
    let planet_y = if rng.random_bool(0.5) {
        rng.random_range(PLANET_LOWER_Y_RANGE)
    } else {
        rng.random_range(PLANET_UPPER_Y_RANGE)
    };

    // Spawn a random planet model with rotation behavior
    let planet_scene = AssetResolver::get_random_planet(&extended_bg_assets, &bg_assets)?;

    cmds.spawn((
        SceneRoot(planet_scene),
        Transform::default().with_translation(Vec3::new(
            planet_x,
            planet_y,
            rng.random_range(PLANET_Z_RANGE),
        )),
        PlanetRotationComponent::new(rng.random_range(PLANET_ROTATION_SPEED_RANGE)),
        Cleanup::<AppState> {
            states: vec![AppState::MainMenu, AppState::Game],
        },
        Name::new("Planet"),
    ));

    // Calculate star position on opposite side of screen from planet
    let star_x = if planet_x > 0.0 {
        rng.random_range(STAR_CLUSTER_LOWER_X_RANGE)
    } else {
        rng.random_range(STAR_CLUSTER_UPPER_X_RANGE)
    };

    let star_y = if planet_y > 0.0 {
        rng.random_range(STAR_CLUSTER_LOWER_Y_RANGE)
    } else {
        rng.random_range(STAR_CLUSTER_UPPER_Y_RANGE)
    };

    // Spawn star cluster with potential additional stars
    cmds.spawn((
        Transform::from_xyz(star_x, star_y, rng.random_range(STAR_CLUSTER_Z_RANGE)),
        Visibility::default(),
        Cleanup::<AppState> {
            states: vec![AppState::MainMenu, AppState::Game],
        },
        Name::new("Star Cluster"),
    ))
    .with_children(|parent| {
        // Spawn central star
        spawn_star(parent, Vec3::ZERO, &mut materials, &mut meshes, &mut rng);

        // 15% chance to spawn additional star to the upper right
        if rng.random_bool(ADDITIONAL_STAR_CHANCE) {
            spawn_star(
                parent,
                Vec3::new(
                    rng.random_range(STAR_UPPER_RIGHT_X_RANGE),
                    rng.random_range(STAR_UPPER_RIGHT_Y_RANGE),
                    rng.random_range(STAR_Z_RANGE),
                ),
                &mut materials,
                &mut meshes,
                &mut rng,
            );
        }

        // 15% chance to spawn additional star to the lower left
        if rng.random_bool(ADDITIONAL_STAR_CHANCE) {
            spawn_star(
                parent,
                Vec3::new(
                    rng.random_range(STAR_LOWER_LEFT_X_RANGE),
                    rng.random_range(STAR_LOWER_LEFT_Y_RANGE),
                    rng.random_range(STAR_Z_RANGE),
                ),
                &mut materials,
                &mut meshes,
                &mut rng,
            );
        }
    });

    Ok(())
}

/// Helper function to spawn an individual star with random properties
fn spawn_star(
    csc: &mut ChildSpawnerCommands,
    pos: Vec3,
    materials: &mut Assets<StandardMaterial>,
    meshes: &mut Assets<Mesh>,
    rng: &mut ThreadRng,
) {
    // Generate random bright color for the star
    let star_color = Color::srgb(
        rng.random::<f32>(),
        rng.random::<f32>(),
        rng.random::<f32>(),
    );

    // Spawn star mesh with emissive material
    csc.spawn((
        Mesh3d(
            meshes.add(
                Sphere::new(rng.random_range(STAR_RADIUS_RANGE))
                    .mesh()
                    .uv(14, 9),
            ),
        ),
        Transform::from_translation(pos),
        Visibility::default(),
        MeshMaterial3d(materials.add(StandardMaterial {
            emissive: star_color.into(),
            diffuse_transmission: 1.0,
            ..default()
        })),
        Name::new("Star"),
    ))
    // Add point light to make star glow with a random light intensity
    .with_child((
        PointLight {
            color: star_color,
            intensity: rng.random_range(STAR_POINT_LIGHT_INTENSITY_RANGE),
            range: STAR_POINT_LIGHT_RANGE,
            ..default()
        },
        Name::new("Star Point Light"),
    ));
}
