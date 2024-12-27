use bevy::prelude::Component;

// Component to handle planet rotation behavior
#[derive(Component)]
pub(super) struct PlanetRotationComponent {
    // Speed at which the planet rotates
    pub rotation_speed: f32,
}

// Implementation of PlanetRotationComponent methods
impl PlanetRotationComponent {
    // Creates a new PlanetRotationComponent with specified speed
    pub(super) fn new(speed: f32) -> Self {
        Self {
            rotation_speed: speed,
        }
    }
}
