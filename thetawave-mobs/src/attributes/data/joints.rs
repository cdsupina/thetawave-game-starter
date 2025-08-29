use bevy::{
    ecs::{component::Component, entity::Entity},
    math::Vec2,
    platform::collections::HashMap,
    reflect::Reflect,
};
use serde::Deserialize;

use crate::MobType;

/// Describes an Avian2D angle limit for a joint
#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct JointAngleLimit {
    pub min: f32,
    pub max: f32,
    pub torque: f32,
}

/// Used for making mob chains of random length
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct RandomMobChain {
    pub min_length: u8,
    pub end_chance: f32,
}

/// Describes a chain of mobs that are spawned and jointed together
#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct MobChain {
    pub length: u8,
    pub pos_offset: Vec2,
    pub anchor_offset: Vec2,
    pub random_chain: Option<RandomMobChain>,
}

/// Mob that is also spawned and jointed to the original mob
#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct JointedMob {
    pub key: String,
    pub mob_type: MobType,
    #[serde(default)]
    pub offset_pos: Vec2,
    #[serde(default)]
    pub anchor_1_pos: Vec2,
    #[serde(default)]
    pub anchor_2_pos: Vec2,
    #[serde(default)]
    pub angle_limit_range: Option<JointAngleLimit>,
    #[serde(default)]
    pub compliance: f32,
    #[serde(default)]
    pub chain: Option<MobChain>,
}

/// Hashmap of joints connected to a mob
/// This is for "anchors" only
/// Used by behaviors for referencing joint entities
#[derive(Component, Reflect)]
pub(crate) struct JointsComponent {
    pub joints: HashMap<String, Entity>,
}
