use bevy::{
    ecs::{component::Component, entity::Entity, event::Event},
    time::{Timer, TimerMode},
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
    pub key: Option<String>,
    /// Whether this particle effect should track its parent's position (for projectile trails)
    /// If false, maintains parent-child relationship (for spawner effects)
    pub needs_position_tracking: bool,
    /// If true, the spawner will emit once and despawn
    pub is_one_shot: bool,
    /// Scale multiplier for particle effect properties (emission_shape, speeds, etc.)
    pub scale: Option<f32>,
}

// Used for associating particle effects with spawners based on spawner keys
#[derive(Event)]
pub struct SpawnerParticleEffectSpawnedEvent {
    pub key: String,
    pub effect_entity: Entity,
    pub parent_entity: Entity,
}

/// Event for setting the active state of a particle entity
#[derive(Event)]
pub struct ActivateParticleEvent {
    pub entity: Entity,
    pub active: bool,
}

/// Component for managing particle spawner lifetime after parent despawn
/// Allows particles to finish their natural lifetime before despawning the spawner
#[derive(Component)]
pub struct ParticleLifeTimer {
    pub timer: Timer,
    pub parent_entity: Option<Entity>,
}

impl ParticleLifeTimer {
    /// Create a new ParticleLifeTimer with the given lifetime in seconds
    /// Uses the maximum particle lifetime to ensure all particles can complete
    pub fn new(lifetime_seconds: f32, parent_entity: Option<Entity>) -> Self {
        Self {
            timer: Timer::from_seconds(lifetime_seconds, TimerMode::Once),
            parent_entity,
        }
    }
}
