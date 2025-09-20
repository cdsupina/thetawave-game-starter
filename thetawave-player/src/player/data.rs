use bevy::ecs::{component::Component, entity::Entity, event::Event};

use crate::CharacterAttributes;

/// Component for storing values used in systems for player entities
#[derive(Component)]
pub struct PlayerStats {
    pub acceleration: f32,
    pub deceleration_factor: f32,
    pub projectile_damage: u32,
    pub projectile_speed: f32,
    pub projectile_range_seconds: f32,
}

impl From<&CharacterAttributes> for PlayerStats {
    fn from(value: &CharacterAttributes) -> Self {
        PlayerStats {
            acceleration: value.acceleration,
            deceleration_factor: value.deceleration_factor,
            projectile_damage: value.projectile_damage,
            projectile_speed: value.projectile_speed,
            projectile_range_seconds: value.projectile_range_seconds,
        }
    }
}

#[derive(Event)]
pub struct PlayerDeathEvent {
    pub player_entity: Entity,
}
