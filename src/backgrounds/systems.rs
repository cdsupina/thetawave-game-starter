use crate::assets::BackgroundAssets;
use bevy::{
    asset::Assets,
    color::{Alpha, Color},
    math::Vec3,
    pbr::{MeshMaterial3d, StandardMaterial},
    prelude::{AlphaMode, Commands, Mesh, Mesh3d, Rectangle, Res, ResMut, Transform},
    utils::default,
};

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
        Transform::default().with_scale(Vec3::splat(250.0).with_z(-10.0)),
    ));
}
