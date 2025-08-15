use bevy::ecs::component::Component;

/// Component for storing values used in systems for player entities
#[derive(Component)]
pub struct PlayerStats {
    pub acceleration: f32,
    pub deceleration_factor: f32,
}
