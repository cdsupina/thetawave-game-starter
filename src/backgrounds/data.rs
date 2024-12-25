use bevy::prelude::Component;

#[derive(Component)]
pub(super) struct PlanetRotationComponent {
    pub rotation_speed: f32,
}

impl PlanetRotationComponent {
    pub(super) fn new(speed: f32) -> Self {
        Self {
            rotation_speed: speed,
        }
    }
}
