use bevy::{
    ecs::component::Component,
    math::Vec2,
    platform::collections::HashMap,
    reflect::Reflect,
    time::{Timer, TimerMode},
};
use serde::{Deserialize, Serialize};
use thetawave_projectiles::ProjectileSpawner;

use crate::asset::MobRef;

/// Mob spawner component for use in spawned mobs
/// Maps String keys to MobSpawners
/// Intended to be used by behaviors
#[derive(Component, Serialize, Deserialize, Debug, Clone, Reflect)]
#[serde(deny_unknown_fields)]
pub struct MobSpawnerComponent {
    pub spawners: HashMap<String, MobSpawner>,
}

/// Projectile spawner component for use in spawned mobs
/// Maps String keys to ProjectileSpawners
/// Intended to be used by behaviors
#[derive(Component, Serialize, Deserialize, Debug, Clone, Reflect)]
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
    /// Automatically normalized to "xhitara/grunt" format.
    pub mob_ref: MobRef,
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
            pub mob_ref: MobRef,
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

impl Serialize for MobSpawner {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("MobSpawner", 4)?;
        state.serialize_field("timer", &self.timer.duration().as_secs_f32())?;
        state.serialize_field("position", &self.position)?;
        state.serialize_field("rotation", &self.rotation)?;
        state.serialize_field("mob_ref", &self.mob_ref)?;
        state.end()
    }
}
