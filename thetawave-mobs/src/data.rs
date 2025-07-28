use avian2d::prelude::{Collider, LockedAxes};
use bevy::{
    ecs::{event::Event, name::Name, resource::Resource},
    math::Vec2,
    platform::collections::HashMap,
};
use serde::Deserialize;

const DEFAULT_COLLIDER_DIMENSIONS: Vec2 = Vec2::new(10.0, 10.0);
const DEFAULT_Z_LEVEL: f32 = 0.0;
const DEFAULT_ROTATION_LOCKED: bool = true;

/// All types of spawnable mobs
#[derive(Deserialize, Debug, Eq, PartialEq, Hash)]
pub enum MobType {
    Grunt,
    Shooter,
}

/// Event for spawning mobs using a mob type and position
#[derive(Event, Debug)]
pub struct SpawnMobEvent {
    pub mob_type: MobType,
    pub position: Vec2,
}

// Contains all attributes for a mob
#[derive(Deserialize, Debug, Clone)]
pub struct MobAttributes {
    collider_dimensions: Option<Vec2>,
    name: String,
    z_level: Option<f32>,
    rotation_locked: Option<bool>,
}

impl MobAttributes {
    /// Get z level from attributes or default to DEFAULT_Z_LEVEL
    pub fn get_z_level(&self) -> f32 {
        self.z_level.unwrap_or(DEFAULT_Z_LEVEL)
    }
}

// Resource tracking all data for mobs
#[derive(Deserialize, Debug, Resource)]
pub struct MobResource {
    pub attributes: HashMap<MobType, MobAttributes>,
}

/// Create a collider component using mob attributes
impl From<&MobAttributes> for Collider {
    fn from(value: &MobAttributes) -> Self {
        let collider_dimensions = value
            .collider_dimensions
            .unwrap_or(DEFAULT_COLLIDER_DIMENSIONS);

        Collider::rectangle(collider_dimensions.x, collider_dimensions.y)
    }
}

impl From<&MobAttributes> for Name {
    fn from(value: &MobAttributes) -> Self {
        Name::new(value.name.clone())
    }
}

impl From<&MobAttributes> for LockedAxes {
    fn from(value: &MobAttributes) -> Self {
        let rotation_locked = value.rotation_locked.unwrap_or(DEFAULT_ROTATION_LOCKED);

        if rotation_locked {
            return LockedAxes::ROTATION_LOCKED;
        }

        // unlock rotation if rotation locked is not true
        LockedAxes::new()
    }
}
