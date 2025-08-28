use avian2d::prelude::{Collider, Rotation};
use bevy::{
    ecs::{component::Component, entity::Entity, event::Event, resource::Resource},
    math::Vec2,
    platform::collections::HashMap,
    reflect::Reflect,
    time::{Timer, TimerMode},
    transform::components::Transform,
};
use serde::Deserialize;
use thetawave_core::Faction;
use thetawave_physics::ThetawaveCollider;

#[derive(Component, Debug, Deserialize, Eq, PartialEq, Hash, Reflect, Clone)]
pub enum ProjectileType {
    Bullet,
    Blast,
}

/// Used for indicating which effect to spawn in the spawn effect system
pub(crate) enum ProjectileEffectType {
    Despawn,
    Hit,
}

/// Enforce a range the projectile based on the time existing
#[derive(Component)]
pub struct ProjectileRangeComponent {
    pub timer: Timer,
}

impl ProjectileRangeComponent {
    pub fn new(range_seconds: f32) -> Self {
        Self {
            timer: Timer::from_seconds(range_seconds, TimerMode::Once),
        }
    }
}

#[derive(Event)]
pub struct SpawnProjectileEvent {
    pub projectile_type: ProjectileType,
    pub faction: Faction,
    pub position: Vec2,
    pub rotation: f32,
    pub speed: f32,
    pub damage: u32,
    pub range_seconds: f32,
}

#[derive(Event)]
pub(crate) struct SpawnProjectileEffectEvent {
    pub projectile_type: ProjectileType,
    pub effect_type: ProjectileEffectType,
    pub faction: Faction,
    pub transform: Transform,
}

#[derive(Component)]
pub struct DespawnAfterAnimationComponent;

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
    pub damage_multiplier: f32,
    pub range_seconds_multiplier: f32,
    pub spawn_effect_entity: Option<Entity>,
    pub pre_spawn_animation_start_time: f32,
    pub pre_spawn_animation_end_time: f32,
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
            #[serde(default = "default_multiplier")]
            pub speed_multiplier: f32,
            #[serde(default = "default_multiplier")]
            pub damage_multiplier: f32,
            #[serde(default = "default_multiplier")]
            pub range_seconds_multiplier: f32,
            #[serde(default = "default_pre_spawn_animation_start_time")]
            pub pre_spawn_animation_start_time: f32,
            #[serde(default = "default_pre_spawn_animation_end_time")]
            pub pre_spawn_animation_end_time: f32,
        }

        fn default_multiplier() -> f32 {
            1.0
        }

        fn default_pre_spawn_animation_start_time() -> f32 {
            0.75
        }

        fn default_pre_spawn_animation_end_time() -> f32 {
            0.2
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
            damage_multiplier: helper.damage_multiplier,
            range_seconds_multiplier: helper.range_seconds_multiplier,
            pre_spawn_animation_start_time: helper.pre_spawn_animation_start_time,
            pre_spawn_animation_end_time: helper.pre_spawn_animation_end_time,
            spawn_effect_entity: None, // set to non because the entity cannot be known beforehand
        })
    }
}
