use bevy::{
    ecs::{component::Component, entity::Entity, event::Event},
    math::Vec2,
};
use thetawave_projectiles::ProjectileSpread;

use crate::CharacterAttributes;

/// Component for storing values used in systems for player entities
#[derive(Component)]
pub struct PlayerStats {
    pub acceleration: f32,
    pub deceleration_factor: f32,
    pub projectile_damage: u32,
    pub projectile_speed: f32,
    pub projectile_range_seconds: f32,
    pub inherited_velocity_multiplier: f32,
    pub projectile_spawner_position: Vec2,
    pub projectile_count: u8,
    pub projectile_spread: ProjectileSpread,
}

impl From<&CharacterAttributes> for PlayerStats {
    fn from(value: &CharacterAttributes) -> Self {
        PlayerStats {
            acceleration: value.acceleration,
            deceleration_factor: value.deceleration_factor,
            projectile_damage: value.projectile_damage,
            projectile_speed: value.projectile_speed,
            projectile_range_seconds: value.projectile_range_seconds,
            inherited_velocity_multiplier: value.inherited_velocity_multiplier,
            projectile_spawner_position: value.projectile_spawner_position,
            projectile_count: value.projectile_count,
            projectile_spread: value.projectile_spread.clone(),
        }
    }
}

#[derive(Event)]
pub struct PlayerDeathEvent {
    pub player_entity: Entity,
}
