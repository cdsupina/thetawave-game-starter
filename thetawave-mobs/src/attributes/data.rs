use avian2d::prelude::{Collider, LockedAxes, Restitution};
use bevy::{
    ecs::{component::Component, event::Event, name::Name, resource::Resource},
    math::Vec2,
    platform::collections::HashMap,
};
use serde::Deserialize;

const DEFAULT_COLLIDER_DIMENSIONS: Vec2 = Vec2::new(10.0, 10.0);
const DEFAULT_Z_LEVEL: f32 = 0.0;
const DEFAULT_ROTATION_LOCKED: bool = true;
const DEFAULT_MAX_LINEAR_SPEED: Vec2 = Vec2::new(20.0, 20.0);
const DEFAULT_LINEAR_ACCELERATION: Vec2 = Vec2::new(0.1, 0.1);
const DEFAULT_LINEAR_DECELERATION: Vec2 = Vec2::new(0.3, 0.3);
const DEFAULT_RESTITUTION: f32 = 0.3;

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

/// Component to hold mob attributes that are not used in cases outside of creating components
/// Such as in mob behaviors
#[derive(Component)]
pub(crate) struct MobAttributesComponent {
    pub linear_acceleration: Vec2,
    pub linear_deceleration: Vec2,
    pub max_linear_speed: Vec2,
}

// Contains all attributes for a mob
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct MobAttributes {
    #[serde(default = "default_collider_dimensions")]
    collider_dimensions: Vec2,
    name: String,
    #[serde(default = "default_z_level")]
    pub z_level: f32,
    #[serde(default = "default_rotation_locked")]
    rotation_locked: bool,
    #[serde(default = "default_max_linear_speed")]
    max_linear_speed: Vec2,
    #[serde(default = "default_linear_acceleration")]
    pub linear_acceleration: Vec2,
    #[serde(default = "default_linear_deceleration")]
    pub linear_deceleration: Vec2,
    #[serde(default = "default_restitution")]
    pub restitution: f32,
}

fn default_collider_dimensions() -> Vec2 {
    DEFAULT_COLLIDER_DIMENSIONS
}

fn default_z_level() -> f32 {
    DEFAULT_Z_LEVEL
}

fn default_rotation_locked() -> bool {
    DEFAULT_ROTATION_LOCKED
}

fn default_max_linear_speed() -> Vec2 {
    DEFAULT_MAX_LINEAR_SPEED
}

fn default_linear_acceleration() -> Vec2 {
    DEFAULT_LINEAR_ACCELERATION
}

fn default_linear_deceleration() -> Vec2 {
    DEFAULT_LINEAR_DECELERATION
}

fn default_restitution() -> f32 {
    DEFAULT_RESTITUTION
}

// Resource tracking all data for mobs
#[derive(Deserialize, Debug, Resource)]
pub(crate) struct MobAttributesResource {
    pub attributes: HashMap<MobType, MobAttributes>,
}

impl From<&MobAttributes> for Restitution {
    fn from(value: &MobAttributes) -> Self {
        Restitution::new(value.restitution)
    }
}

/// Create a collider component using mob attributes
impl From<&MobAttributes> for Collider {
    fn from(value: &MobAttributes) -> Self {
        let collider_dimensions = value.collider_dimensions;

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
        let rotation_locked = value.rotation_locked;

        if rotation_locked {
            return LockedAxes::ROTATION_LOCKED;
        }

        // unlock rotation if rotation locked is not true
        LockedAxes::new()
    }
}

impl From<&MobAttributes> for MobAttributesComponent {
    fn from(value: &MobAttributes) -> Self {
        MobAttributesComponent {
            linear_acceleration: value.linear_acceleration,
            linear_deceleration: value.linear_deceleration,
            max_linear_speed: value.max_linear_speed,
        }
    }
}
