use bevy::{
    ecs::{event::Event, resource::Resource},
    math::Vec2,
    platform::collections::HashMap,
};
use serde::Deserialize;

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
#[derive(Deserialize, Debug)]
pub struct MobAttributes {
    pub collider_dimensions: Vec2,
}

// Resource tracking all data for mobs
#[derive(Deserialize, Debug, Resource)]
pub struct MobResource {
    pub attributes: HashMap<MobType, MobAttributes>,
}
