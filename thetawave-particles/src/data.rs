use bevy::{
    ecs::{entity::Entity, event::Event},
    reflect::Reflect,
    transform::components::Transform,
};

#[derive(Reflect)]
pub enum ParticleEffectType {
    SpawnBlast,
}

#[derive(Event)]
pub struct SpawnParticleEffectEvent {
    /// If parent entity is some the particle effect should be spawned as a child entity of the parent
    /// The transform will be relative to the parent in this case
    pub parent_entity: Option<Entity>,
    pub effect_type: ParticleEffectType,
    pub transform: Transform,
}

/// Event for settig the active state of a particle entity
#[derive(Event)]
pub struct ActivateParticleEvent {
    pub entity: Entity,
    pub active: bool,
}
