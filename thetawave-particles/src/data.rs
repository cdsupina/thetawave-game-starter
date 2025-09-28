use bevy::{
    ecs::{component::Component, entity::Entity, event::Event},
    math::{Vec2, Vec3},
    time::{Timer, TimerMode},
    transform::components::Transform,
};
use rand::Rng;
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
    /// Override the particle direction vector (replaces the direction from the asset)
    pub direction: Option<Vec2>,
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

#[derive(Event)]
pub struct ToggleActiveParticleEvent {
    pub entity: Entity,
}

/// Component for managing particle spawner lifetime after parent despawn
/// Allows particles to finish their natural lifetime before despawning the spawner
#[derive(Component)]
pub struct ParticleLifeTimer {
    pub timer: Timer,
    pub parent_entity: Option<Entity>,
    pub offset: Vec3, // Offset from parent position
}

/// Component for blood effects that need random pulsing behavior
#[derive(Component, Debug)]
pub struct BloodEffectManager {
    pub min_interval: f32,
    pub max_interval: f32,
    pub timer: Timer,
    pub pulses_remaining: u8,
    pub decrease_factor: f32,
}

impl BloodEffectManager {
    /// Create a new BloodEffectManager with specified interval range
    pub fn new(min_interval: f32, max_interval: f32) -> Self {
        Self {
            timer: Self::reset_timer(min_interval, max_interval),
            min_interval,
            max_interval,
            pulses_remaining: 50,
            decrease_factor: 0.9,
        }
    }

    fn reset_timer(min_interval: f32, max_interval: f32) -> Timer {
        let random_duration = rand::rng().random_range(min_interval..=max_interval);
        Timer::from_seconds(random_duration, TimerMode::Once)
    }

    /// Reset the timer with a new random interval
    pub fn reset_timer_to_random(&mut self) {
        self.update_intervals();
        self.timer = Self::reset_timer(self.min_interval, self.max_interval);
    }

    fn update_intervals(&mut self) {
        self.min_interval *= self.decrease_factor;
        self.max_interval *= self.decrease_factor;
    }
}

impl ParticleLifeTimer {
    /// Create a new ParticleLifeTimer with the given lifetime in seconds
    /// Uses the maximum particle lifetime to ensure all particles can complete
    pub fn new(lifetime_seconds: f32, parent_entity: Option<Entity>) -> Self {
        Self {
            timer: Timer::from_seconds(lifetime_seconds, TimerMode::Once),
            parent_entity,
            offset: Vec3::ZERO,
        }
    }

    /// Create a new ParticleLifeTimer with an offset from the parent position
    pub fn new_with_offset(
        lifetime_seconds: f32,
        parent_entity: Option<Entity>,
        offset: Vec3,
    ) -> Self {
        Self {
            timer: Timer::from_seconds(lifetime_seconds, TimerMode::Once),
            parent_entity,
            offset,
        }
    }
}
