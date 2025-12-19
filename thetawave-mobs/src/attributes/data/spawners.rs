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
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Helper {
            pub timer: f32,
            pub position: Vec2,
            pub rotation: f32,
            pub mob_ref: String,
        }

        let helper = Helper::deserialize(deserializer)?;

        Ok(MobSpawner {
            timer: Timer::from_seconds(helper.timer, TimerMode::Repeating),
            position: helper.position,
            rotation: helper.rotation,
            mob_ref: helper.mob_ref,
        })
    }
}
