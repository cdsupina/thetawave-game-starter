use crate::assets::BackgroundAssets;
use bevy::{
    asset::Assets,
    color::{Alpha, Color},
    math::Vec3,
    pbr::{MeshMaterial3d, StandardMaterial},
    prelude::{AlphaMode, Commands, Mesh, Mesh3d, Rectangle, Res, ResMut, Transform},
    scene::SceneRoot,
    utils::default,
};
use rand::Rng;

/// Function to spawn a background element in the game
pub(super) fn spawn_bg(
    mut cmds: Commands,
    bg_assets: Res<BackgroundAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create a semi-transparent material with the blue space background texture
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

    // give planet a random position
    let mut rng = rand::thread_rng();
    cmds.spawn((
        SceneRoot(bg_assets.get_random_planet()),
        Transform::default().with_translation(Vec3::new(
            rng.gen_range(-12.0..=12.0),
            rng.gen_range(-8.0..=8.0),
            rng.gen_range(-30.0..=-15.0),
        )),
    ));
}
