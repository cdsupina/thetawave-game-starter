use bevy::{ecs::resource::Resource, platform::collections::HashMap, prelude::Component};
use serde::Deserialize;

use crate::MobType;

const DEFAULT_DURATION: f32 = 1.0;
const DEFAULT_WEIGHT: f32 = 1.0;

/// Simple behaviors for mobs
#[derive(Deserialize, Debug, Clone)]
enum MobBehavior {
    MoveDown,
    BrakeHorizontal,
}

/// A collection of behaviors to execute together
#[derive(Deserialize, Debug, Clone)]
struct MobBehaviorBlock {
    behaviors: Vec<MobBehavior>,
    #[serde(default = "default_duration")]
    duration: f32,
    /// How likely the behavior block is to be chosen using
    /// MobBehaviorSquenceMethod::Random
    #[serde(default = "default_weight")]
    weight: f32,
}

fn default_weight() -> f32 {
    DEFAULT_WEIGHT
}

fn default_duration() -> f32 {
    DEFAULT_DURATION
}

/// A collection of behavior blocks that execute in order
#[derive(Component, Deserialize, Debug, Clone)]
pub(crate) struct MobBehaviorSequence {
    blocks: Vec<MobBehaviorBlock>,
    execution_order: ExecutionOrder,
}

/// How behavior blocks are executed
#[derive(Deserialize, Debug, Clone)]
enum ExecutionOrder {
    Sequential,
    Random,
}

/// Resource containing behavior sequences for mobs
#[derive(Deserialize, Debug, Resource)]
pub(crate) struct MobBehaviorsResource {
    pub behaviors: HashMap<MobType, MobBehaviorSequence>,
}
