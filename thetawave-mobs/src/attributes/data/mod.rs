use bevy::{
    ecs::{component::Component, entity::Entity, message::Message},
    math::Vec2,
    reflect::Reflect,
};

mod joints;
mod spawners;

pub use joints::JointsComponent;
pub use spawners::{MobSpawnerComponent, ProjectileSpawnerComponent};

/// Marker component for identifying mob entities in queries
#[derive(Component, Reflect, Debug, Clone)]
pub struct MobMarker {
    mob_type: String,
}

impl MobMarker {
    pub fn new(mob_type: impl Into<String>) -> Self {
        Self {
            mob_type: mob_type.into(),
        }
    }

    pub fn mob_type(&self) -> &str {
        &self.mob_type
    }
}

/// Mob attributes used by behaviors for movement and combat
#[derive(Component, Reflect)]
pub struct MobAttributesComponent {
    pub linear_acceleration: Vec2,
    pub linear_deceleration: Vec2,
    pub max_linear_speed: Vec2,
    pub angular_acceleration: f32,
    pub angular_deceleration: f32,
    pub max_angular_speed: f32,
    pub targeting_range: Option<f32>,
    pub projectile_speed: f32,
    pub projectile_damage: u32,
    pub projectile_range_seconds: f32,
}

#[derive(Message)]
pub struct MobDeathEvent {
    pub mob_entity: Entity,
}
