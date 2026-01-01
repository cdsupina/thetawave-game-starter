use bevy::{
    camera::ScalingMode,
    ecs::message::MessageReader,
    input::mouse::{MouseMotion, MouseWheel},
    prelude::{
        ButtonInput, Camera2d, Commands, Component, KeyCode, MouseButton, OrthographicProjection,
        Projection, Query, Res, ResMut, Resource, Time, Transform, Vec2, Vec3, With,
    },
};
use bevy_egui::EguiContexts;

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
    pub show_jointed_mobs: bool,
    pub show_joint_gizmos: bool,
    /// Target zoom (for smooth zooming)
    target_zoom: f32,
    /// Target pan offset (for smooth panning)
    target_pan: Vec2,
}

impl Default for PreviewSettings {
    fn default() -> Self {
        Self {
            zoom: 3.0, // Start zoomed in a bit for pixel art
            pan_offset: Vec2::ZERO,
            show_grid: true,
            show_colliders: true,
            show_jointed_mobs: false, // Hidden by default
            show_joint_gizmos: true,  // Show anchor points when enabled
            target_zoom: 3.0,
            target_pan: Vec2::ZERO,
        }
    }
}

impl PreviewSettings {
    /// Set target zoom level (will animate to this)
    pub fn set_zoom(&mut self, zoom: f32) {
        self.target_zoom = zoom.clamp(0.5, 20.0);
    }

    /// Adjust zoom by a delta factor
    pub fn adjust_zoom(&mut self, delta: f32) {
        self.set_zoom(self.target_zoom * (1.0 + delta));
    }

    /// Adjust pan by a delta
    pub fn adjust_pan(&mut self, delta: Vec2) {
        self.target_pan += delta / self.zoom;
    }

    /// Reset view to default
    pub fn reset_view(&mut self) {
        self.target_zoom = 3.0;
        self.target_pan = Vec2::ZERO;
    }

    /// Update smooth transitions
    pub fn update(&mut self, dt: f32) {
        let smoothing = 10.0 * dt;
        self.zoom = self.zoom + (self.target_zoom - self.zoom) * smoothing.min(1.0);
        self.pan_offset =
            self.pan_offset + (self.target_pan - self.pan_offset) * smoothing.min(1.0);
    }
}

/// Set up the preview camera
pub fn setup_preview_camera(mut commands: Commands) {
    // Use a fixed viewport height for consistent scaling
    const VIEWPORT_HEIGHT: f32 = 200.0;

    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: VIEWPORT_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        }),
        Transform::from_scale(Vec3::splat(1.0 / 3.0)), // Start zoomed in
        PreviewCamera,
    ));
}

/// Update camera based on preview settings
pub fn update_preview_camera(
    settings: Res<PreviewSettings>,
    mut camera_query: Query<&mut Transform, With<PreviewCamera>>,
) {
    for mut transform in &mut camera_query {
        // Apply zoom via scale (smaller scale = more zoomed in)
        let scale = 1.0 / settings.zoom;
        transform.scale = Vec3::splat(scale);

        // Apply pan offset
        transform.translation.x = settings.pan_offset.x;
        transform.translation.y = settings.pan_offset.y;
    }
}

/// System to update preview settings (called before camera update)
pub fn update_preview_settings(mut settings: ResMut<PreviewSettings>, time: Res<Time>) {
    settings.update(time.delta_secs());
}

/// Handle mouse input for camera control
pub fn handle_camera_input(
    mut settings: ResMut<PreviewSettings>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut scroll_events: MessageReader<MouseWheel>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut contexts: EguiContexts,
) {
    // Check if egui has any popup open (like dropdown menus)
    let egui_wants_scroll = contexts
        .ctx_mut()
        .map(|ctx| {
            // Check if any popup is open (dropdowns, menus, etc.)
            bevy_egui::egui::Popup::is_any_open(ctx)
        })
        .unwrap_or(false);

    if !egui_wants_scroll {
        // Zoom with scroll wheel
        for event in scroll_events.read() {
            let zoom_delta = event.y * 0.1;
            settings.adjust_zoom(zoom_delta);
        }

        // Pan with middle mouse button or right mouse button
        if mouse_button.pressed(MouseButton::Middle) || mouse_button.pressed(MouseButton::Right) {
            for event in mouse_motion.read() {
                settings.adjust_pan(Vec2::new(-event.delta.x, event.delta.y));
            }
        }
    }

    // Reset view with Home key (always allow this)
    if keyboard.just_pressed(KeyCode::Home) {
        settings.reset_view();
    }
}
