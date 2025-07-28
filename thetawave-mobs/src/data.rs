use avian2d::prelude::Collider;
use bevy::{
    ecs::{event::Event, resource::Resource},
    math::Vec2,
    platform::collections::HashMap,
};
use serde::Deserialize;

const DEFAULT_COLLIDER_DIMENSIONS: Vec2 = Vec2::new(10.0, 10.0);

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
    pub collider_dimensions: Option<Vec2>,
    pub name: String,
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
