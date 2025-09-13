use bevy::{
    ecs::{entity::Entity, event::Event},
    transform::components::Transform,
};
use thetawave_core::Faction;

#[derive(Event)]
pub struct SpawnParticleEffectEvent {
    /// If parent entity is some the particle effect should be spawned as a child entity of the parent
    /// The transform will be relative to the parent in this case
    pub parent_entity: Option<Entity>,
    pub effect_type: String,
    /// For particle effects Faction determines the color
    pub faction: Faction,
    pub transform: Transform,
    pub is_active: bool,
}

/// Event for setting the active state of a particle entity
#[derive(Event)]
pub struct ActivateParticleEvent {
    pub entity: Entity,
    pub active: bool,
}
