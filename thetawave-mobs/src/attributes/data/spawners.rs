use bevy::{
    ecs::component::Component,
    math::Vec2,
    platform::collections::HashMap,
    reflect::Reflect,
    time::{Timer, TimerMode},
};
use serde::Deserialize;
use thetawave_projectiles::ProjectileSpawner;

use crate::MobType;

/// Mob spawner component for use in spawned mobs
/// Maps String keys to MobSpawners
/// Intended to be used by behaviors
#[derive(Component, Deserialize, Debug, Clone, Reflect)]
pub(crate) struct MobSpawnerComponent {
    pub spawners: HashMap<String, MobSpawner>,
}

/// Projectile spawner component for use in spawned mobs
/// Maps String keys to ProjectileSpawners
/// Intended to be used by behaviors
#[derive(Component, Deserialize, Debug, Clone, Reflect)]
pub(crate) struct ProjectileSpawnerComponent {
    pub spawners: HashMap<String, ProjectileSpawner>,
}

/// Used for periodically spawning mobs with a MobSpawnerComponent
#[derive(Debug, Clone, Reflect)]
pub(crate) struct MobSpawner {
    pub timer: Timer,
    pub position: Vec2,
    pub rotation: f32,
    pub mob_type: MobType,
}

impl<'de> Deserialize<'de> for MobSpawner {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Define a "helper" struct that mirrors MobSpawner
        // but uses types that can be deserialized easily
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Helper {
            pub timer: f32,
            pub position: Vec2,
            pub rotation: f32,
            pub mob_type: MobType,
        }

        // Let serde deserialize into the Helper struct first
        let helper = Helper::deserialize(deserializer)?;

        // Construct our actual struct with the transformed data
        Ok(MobSpawner {
            timer: Timer::from_seconds(helper.timer, TimerMode::Repeating),
            position: helper.position,
            rotation: helper.rotation,
            mob_type: helper.mob_type,
        })
    }
}
