use avian2d::prelude::{Collider, Rotation};
use bevy::{
    ecs::{event::Event, resource::Resource},
    math::Vec2,
    platform::collections::HashMap,
};
use serde::Deserialize;
use thetawave_core::Faction;
use thetawave_physics::ThetawaveCollider;

#[derive(Debug, Deserialize, Eq, PartialEq, Hash)]
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
}

/// Contains all attributes for a mob
#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProjectileAttributes {
    colliders: Vec<ThetawaveCollider>,
    is_sensor: bool,
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
