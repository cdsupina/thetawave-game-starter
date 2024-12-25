use bevy::prelude::Component;

#[derive(Component)]
pub(super) struct PlanetRotationComponent {
    pub rotation_speed: f32,
}
