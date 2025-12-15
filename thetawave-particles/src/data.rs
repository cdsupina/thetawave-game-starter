use bevy::{
    color::Color,
    ecs::{component::Component, entity::Entity, message::Message},
    math::{Vec2, Vec3},
    time::{Timer, TimerMode},
};
use rand::Rng;

// BloodEffectManager tuning constants
const BLOOD_MAX_ACTIVE_INTERVAL: f32 = 3.0;
const BLOOD_MIN_INACTIVE_INTERVAL: f32 = 0.1;
const BLOOD_RANDOM_RANGE: f32 = 0.5;
const BLOOD_MAX_PULSES: u8 = 100;
const BLOOD_MIN_AMOUNT: f32 = 0.1;
const BLOOD_MIN_PULSES: f32 = 5.0;
const BLOOD_BASE_DECREASE_FACTOR: f32 = 0.85;
const BLOOD_AMOUNT_DECREASE_FACTOR: f32 = 0.1;
const BLOOD_MIN_TIMER_INTERVAL: f32 = 0.1;
const BLOOD_INACTIVE_RANDOM_FACTOR: f32 = 0.3;
const BLOOD_MIN_INACTIVE_TIMER_INTERVAL: f32 = 0.05;

/// Event for spawning blood particle effects
#[derive(Message)]
pub struct SpawnBloodEffectEvent {
    // A value that determines the intensity, amount, and length of the effect
    pub amount: f32,
    // Color of the blood,
    pub color: Color,
    // Entity that is bleeding
    pub parent_entity: Entity,
    // Position of the bleeding relative to the parent
    pub position: Vec2,
    // Direction for the blood spray
    pub direction: Vec2,
}

/// Event for spawning projectile trail particle effects
#[derive(Message)]
pub struct SpawnProjectileTrailEffectEvent {
    // Color of the trail
    pub color: Color,
    // Entity that the trail follows (projectile)
    pub parent_entity: Entity,
    // Scale factor for the trail effect
    pub scale: f32,
}

/// Event for spawning explosion particle effects
#[derive(Message)]
pub struct SpawnExplosionEffectEvent {
    // Color of the explosion
    pub color: Color,
    // Position where the explosion occurs
    pub position: Vec2,
    // Scale multiplier for the explosion size
    pub scale: f32,
}

/// Event for spawning projectile despawn particle effects
#[derive(Message)]
pub struct SpawnProjectileDespawnEffectEvent {
    // Effect type (e.g., "despawn_bullet" or "despawn_blast")
    pub effect_type: String,
    // Color of the despawn effect
    pub color: Color,
    // Position where the despawn effect occurs
    pub position: Vec2,
    // Scale multiplier for the despawn effect size
    pub scale: f32,
}

/// Event for spawning projectile hit particle effects
#[derive(Message)]
pub struct SpawnProjectileHitEffectEvent {
    // Effect type (e.g., "hit_bullet" or "hit_blast")
    pub effect_type: String,
    // Color of the hit effect
    pub color: Color,
    // Position where the hit effect occurs
    pub position: Vec2,
    // Scale multiplier for the hit effect size
    pub scale: f32,
}

/// Event for spawning projectile spawner particle effects on mobs
#[derive(Message)]
pub struct SpawnSpawnerEffectEvent {
    // Parent entity (the mob that owns this spawner)
    pub parent_entity: Entity,
    // Effect type (e.g., "spawn_bullet" or "spawn_blast")
    pub effect_type: String,
    // Color of the spawner effect
    pub color: Color,
    // Position relative to the parent entity
    pub position: Vec2,
    // Key for associating this effect with a specific spawner
    pub key: String,
}

/// Used for associating particle effects with spawners based on spawner keys
#[derive(Message)]
pub struct SpawnerParticleEffectSpawnedEvent {
    pub key: String,
    pub effect_entity: Entity,
    pub parent_entity: Entity,
}

/// Event for setting the active state of a particle entity
#[derive(Message)]
pub struct ActivateParticleEvent {
    pub entity: Entity,
    pub active: bool,
}

#[derive(Message)]
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
    pub max_active_interval: f32,
    pub min_inactive_interval: f32,
    pub timer: Timer,
    pub pulses_remaining: u8,
    pub decrease_factor: f32,
    pub is_active: bool,
}

impl BloodEffectManager {
    /// Create a new BloodEffectManager with specified blood amount
    /// 0.0 is minimal blood, 1.0 is maximum blood intensity
    pub fn new(amount: f32) -> Self {
        let amount = amount.clamp(0.0, 1.0);

        // Scale active interval based on amount (more blood = longer spurts)
        let max_active_interval = BLOOD_MAX_ACTIVE_INTERVAL * amount.max(BLOOD_MIN_AMOUNT);

        // Scale pulses based on amount (more blood = more spurts)
        let pulses_remaining = (BLOOD_MAX_PULSES as f32 * amount).max(BLOOD_MIN_PULSES) as u8;

        // Faster decay for smaller amounts (small wounds heal faster)
        let decrease_factor = BLOOD_BASE_DECREASE_FACTOR + (amount * BLOOD_AMOUNT_DECREASE_FACTOR);

        Self {
            timer: Self::reset_timer(
                (max_active_interval - BLOOD_RANDOM_RANGE).max(BLOOD_MIN_TIMER_INTERVAL),
                max_active_interval + BLOOD_RANDOM_RANGE,
            ),
            max_active_interval,
            min_inactive_interval: BLOOD_MIN_INACTIVE_INTERVAL,
            pulses_remaining,
            decrease_factor,
            is_active: true,
        }
    }

    fn reset_timer(min_interval: f32, max_interval: f32) -> Timer {
        let random_duration = rand::rng().random_range(min_interval..=max_interval);
        Timer::from_seconds(random_duration, TimerMode::Once)
    }

    /// Reset the timer with a new random interval based on current state
    pub fn reset_timer_to_random(&mut self) {
        // Toggle to next state
        self.is_active = !self.is_active;

        if self.is_active {
            // Use active interval (blood spurting) with random variation
            self.timer = Self::reset_timer(
                (self.max_active_interval - BLOOD_RANDOM_RANGE).max(BLOOD_MIN_TIMER_INTERVAL),
                self.max_active_interval + BLOOD_RANDOM_RANGE,
            );
        } else {
            // Use inactive interval (pause between spurts) with random variation
            // Use a smaller random range for inactive intervals to allow growth
            let inactive_random_range =
                (self.min_inactive_interval * BLOOD_INACTIVE_RANDOM_FACTOR).min(BLOOD_RANDOM_RANGE);
            self.timer = Self::reset_timer(
                (self.min_inactive_interval - inactive_random_range)
                    .max(BLOOD_MIN_INACTIVE_TIMER_INTERVAL),
                self.min_inactive_interval + inactive_random_range,
            );
            // Apply decay only after inactive period (blood spurts get weaker over time)
            self.update_intervals();
        }
    }

    fn update_intervals(&mut self) {
        // Active intervals get shorter (blood spurts get weaker)
        self.max_active_interval *= self.decrease_factor;
        // Inactive intervals get longer (longer pauses between spurts)
        self.min_inactive_interval /= self.decrease_factor;
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
