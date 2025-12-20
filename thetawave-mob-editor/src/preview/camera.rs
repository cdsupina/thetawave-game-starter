use bevy::prelude::*;

/// Marker component for the preview camera
#[derive(Component)]
pub struct PreviewCamera;

/// Resource for preview camera settings
#[derive(Resource)]
pub struct PreviewSettings {
    pub zoom: f32,
    pub pan_offset: Vec2,
    pub show_grid: bool,
    pub show_colliders: bool,
}

impl Default for PreviewSettings {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan_offset: Vec2::ZERO,
            show_grid: true,
            show_colliders: true,
        }
    }
}

/// Set up the preview camera
pub fn setup_preview_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        PreviewCamera,
        Transform::default(),
    ));
}
