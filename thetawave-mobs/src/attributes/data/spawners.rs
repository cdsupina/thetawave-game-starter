use bevy::{
    ecs::component::Component,
    math::Vec2,
    platform::collections::HashMap,
    reflect::Reflect,
    time::{Timer, TimerMode},
};
use serde::Deserialize;
use thetawave_projectiles::ProjectileSpawner;


/// Mob spawner component for use in spawned mobs
/// Maps String keys to MobSpawners
/// Intended to be used by behaviors
#[derive(Component, Deserialize, Debug, Clone, Reflect)]
#[serde(deny_unknown_fields)]
pub struct MobSpawnerComponent {
    pub spawners: HashMap<String, MobSpawner>,
}

/// Projectile spawner component for use in spawned mobs
/// Maps String keys to ProjectileSpawners
/// Intended to be used by behaviors
#[derive(Component, Deserialize, Debug, Clone, Reflect)]
#[serde(deny_unknown_fields)]
pub struct ProjectileSpawnerComponent {
    pub spawners: HashMap<String, ProjectileSpawner>,
}

/// Used for periodically spawning mobs with a MobSpawnerComponent
#[derive(Debug, Clone, Reflect)]
pub struct MobSpawner {
    pub timer: Timer,
    pub position: Vec2,
    pub rotation: f32,
    /// Path to the mob file to spawn, e.g., "mobs/xhitara/grunt.mob"
    pub mob_ref: String,
}

impl<'de> Deserialize<'de> for MobSpawner {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Define a "helper" struct that mirrors MobSpawner
        // but uses types that can be deserialized easily
        // Supports both "mob_ref" (new) and "mob_type" (legacy) field names
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Helper {
            pub timer: f32,
            pub position: Vec2,
            pub rotation: f32,
            /// New field name for mob path reference
            pub mob_ref: Option<String>,
            /// Legacy field name, kept for backward compatibility
            pub mob_type: Option<String>,
        }

        // Let serde deserialize into the Helper struct first
        let helper = Helper::deserialize(deserializer)?;

        // Get the mob reference from either new or legacy field
        let mob_ref = helper
            .mob_ref
            .or(helper.mob_type)
            .ok_or_else(|| serde::de::Error::missing_field("mob_ref"))?;

        // Construct our actual struct with the transformed data
        Ok(MobSpawner {
            timer: Timer::from_seconds(helper.timer, TimerMode::Repeating),
            position: helper.position,
            rotation: helper.rotation,
            mob_ref,
        })
    }
}
