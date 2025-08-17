use avian2d::prelude::{Collider, Rotation};
use bevy::{
    ecs::{event::Event, resource::Resource},
    math::Vec2,
    platform::collections::HashMap,
    reflect::Reflect,
    time::{Timer, TimerMode},
};
use serde::Deserialize;
use thetawave_core::Faction;
use thetawave_physics::ThetawaveCollider;

#[derive(Debug, Deserialize, Eq, PartialEq, Hash, Reflect, Clone)]
pub enum ProjectileType {
    Bullet,
    Blast,
}

#[derive(Event)]
pub struct SpawnProjectileEvent {
    pub projectile_type: ProjectileType,
    pub faction: Faction,
    pub position: Vec2,
    pub rotation: f32,
    pub speed: f32,
}

/// Contains all attributes for a mob
#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProjectileAttributes {
    colliders: Vec<ThetawaveCollider>,
    pub is_sensor: bool,
}

impl From<&ProjectileAttributes> for Collider {
    fn from(value: &ProjectileAttributes) -> Self {
        Collider::compound(
            value
                .colliders
                .iter()
                .map(|c| {
                    (
                        c.position,
                        Rotation::degrees(c.rotation),
                        Collider::from(&c.shape),
                    )
                })
                .collect(),
        )
    }
}

/// Resource for storing data for all mobs
/// Used mainly for spawning mobs with a given MobType
#[derive(Deserialize, Debug, Resource)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProjectileAttributesResource {
    pub attributes: HashMap<ProjectileType, ProjectileAttributes>,
}

#[derive(Debug, Clone, Reflect)]
pub struct ProjectileSpawner {
    pub timer: Timer,
    pub position: Vec2,
    pub rotation: f32,
    pub projectile_type: ProjectileType,
    pub faction: Faction,
    pub speed_multiplier: f32,
}

impl<'de> Deserialize<'de> for ProjectileSpawner {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Define a "helper" struct that mirrors ProjectileSpawner
        // but uses types that can be deserialized easily
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Helper {
            pub timer: f32,
            pub position: Vec2,
            pub rotation: f32,
            pub projectile_type: ProjectileType,
            pub faction: Faction,
            #[serde(default = "default_speed_multiplier")]
            pub speed_multiplier: f32,
        }

        fn default_speed_multiplier() -> f32 {
            1.0
        }

        // Let serde deserialize into the Helper struct first
        let helper = Helper::deserialize(deserializer)?;

        // Construct our actual struct with the transformed data
        Ok(ProjectileSpawner {
            timer: Timer::from_seconds(helper.timer, TimerMode::Repeating),
            position: helper.position,
            rotation: helper.rotation,
            projectile_type: helper.projectile_type,
            faction: helper.faction,
            speed_multiplier: helper.speed_multiplier,
        })
    }
}
