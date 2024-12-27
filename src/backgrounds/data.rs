use bevy::prelude::{Component, Resource};

// Resource to manage background regeneration state
#[derive(Resource)]
pub(super) struct BackgroundRes {
    // Flag to determine if background needs to be regenerated upon returning to the main manu
    pub regenerate_bg: bool,
}

// Implements default values for BackgroundRes
impl Default for BackgroundRes {
    fn default() -> Self {
        BackgroundRes {
            regenerate_bg: true,
        }
    }
}

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
