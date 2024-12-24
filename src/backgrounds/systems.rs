use crate::assets::BackgroundAssets;
use bevy::{
    asset::Assets,
    color::{Alpha, Color},
    math::Vec3,
    pbr::{MeshMaterial3d, PointLight, StandardMaterial},
    prelude::{
        AlphaMode, BuildChildren, ChildBuild, ChildBuilder, Commands, Mesh, Mesh3d, Meshable,
        Rectangle, Res, ResMut, Sphere, Transform, Visibility,
    },
    scene::SceneRoot,
    utils::default,
};
use rand::{rngs::ThreadRng, Rng};

/// Function to spawn a background element in the game
pub(super) fn spawn_bg_system(
    mut cmds: Commands,
    bg_assets: Res<BackgroundAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create a semi-transparent material with a random space background texture
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(bg_assets.get_random_space_bg()),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        base_color: Color::default().with_alpha(0.1),
        ..default()
    });

    // Spawn a rectangle mesh with material and transform
    cmds.spawn((
        Mesh3d(meshes.add(Rectangle::default())),
        MeshMaterial3d(material_handle),
        Transform::default()
            .with_scale(Vec3::splat(250.0))
            .with_translation(Vec3::new(0.0, 0.0, -100.0)),
    ));

    // spawn a random planet model at a random position
    let mut rng = rand::thread_rng();

    let planet_x = rng.gen_range(-12.0..=12.0);
    let planet_y = rng.gen_range(-8.0..=8.0);

    cmds.spawn((
        SceneRoot(bg_assets.get_random_planet()),
        Transform::default().with_translation(Vec3::new(
            planet_x,
            planet_y,
            rng.gen_range(-30.0..=-20.0),
        )),
    ));

    // Spawn a star with a random color

    let star_x = if planet_x > 0.0 {
        rng.gen_range(-18.0..=-5.0)
    } else {
        rng.gen_range(5.0..=18.0)
    };

    let star_y = if planet_y > 0.0 {
        rng.gen_range(-8.0..=0.0)
    } else {
        rng.gen_range(0.0..=8.0)
    };

    cmds.spawn((
        Transform::from_xyz(star_x, star_y, rng.gen_range(-40.0..=-25.0)),
        Visibility::default(),
    ))
    .with_children(|parent| {
        spawn_star(
            parent,
            Vec3::new(0., 0., 0.),
            &mut materials,
            &mut meshes,
            &mut rng,
        );

        if rng.gen_bool(0.5) {
            spawn_star(
                parent,
                Vec3::new(
                    rng.gen_range(6.0..=10.0),
                    rng.gen_range(6.0..=10.0),
                    rng.gen_range(-10.0..=10.0),
                ),
                &mut materials,
                &mut meshes,
                &mut rng,
            );
        }

        if rng.gen_bool(0.5) {
            spawn_star(
                parent,
                Vec3::new(
                    rng.gen_range(-10.0..=-6.0),
                    rng.gen_range(-10.0..=-6.0),
                    rng.gen_range(-10.0..=10.0),
                ),
                &mut materials,
                &mut meshes,
                &mut rng,
            );
        }
    });
}

fn spawn_star(
    cb: &mut ChildBuilder,
    pos: Vec3,
    materials: &mut Assets<StandardMaterial>,
    meshes: &mut Assets<Mesh>,
    rng: &mut ThreadRng,
) {
    let star_color = Color::srgb(
        1.5 + rng.gen_range(0.0..=1.0),
        1.5 + rng.gen_range(0.0..=1.0),
        1.5 + rng.gen_range(0.0..=1.0),
    );

    cb.spawn((
        Mesh3d(meshes.add(Sphere::new(rng.gen_range(0.5..=2.0)).mesh().uv(32, 18))),
        Transform::from_translation(pos),
        Visibility::default(),
        MeshMaterial3d(materials.add(StandardMaterial {
            emissive: star_color.into(),
            ..default()
        })),
    ))
    .with_child(PointLight {
        color: star_color,
        intensity: 30000000.0,
        range: 100.0,
        ..default()
    });
}
