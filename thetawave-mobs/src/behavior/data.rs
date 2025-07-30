use bevy::{
    ecs::{entity::Entity, event::Event, resource::Resource},
    platform::collections::HashMap,
    prelude::Component,
};
use serde::Deserialize;

use crate::MobType;

const DEFAULT_DURATION: f32 = 1.0;
const DEFAULT_WEIGHT: f32 = 1.0;

/// Simple behaviors for mobs
#[derive(Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) enum MobBehavior {
    MoveDown,
    BrakeHorizontal,
}

/// Event storing a behavior and Vec of entities to run on behavior on
#[derive(Event)]
pub(super) struct MobBehaviorEvent {
    pub behavior: MobBehavior,
    pub entities: Vec<Entity>,
}

/// A collection of behaviors to execute together
#[derive(Deserialize, Debug, Clone)]
pub(crate) struct MobBehaviorBlock {
    pub behaviors: Vec<MobBehavior>,
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
    pub blocks: Vec<MobBehaviorBlock>,
    execution_order: ExecutionOrder,
    #[serde(default)]
    current_idx: usize,
}

impl MobBehaviorSequence {
    pub(super) fn get_active_block(&self) -> Option<&MobBehaviorBlock> {
        self.blocks.get(self.current_idx)
    }
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
