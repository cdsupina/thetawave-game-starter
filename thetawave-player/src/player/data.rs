use bevy::ecs::{component::Component, entity::Entity, event::Event};

/// Component for storing values used in systems for player entities
#[derive(Component)]
pub struct PlayerStats {
    pub acceleration: f32,
    pub deceleration_factor: f32,
}

#[derive(Event)]
pub struct PlayerDeathEvent {
    pub player_entity: Entity,
}
