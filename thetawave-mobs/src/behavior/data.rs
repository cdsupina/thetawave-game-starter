use std::time::Duration;

use bevy::{
    ecs::{entity::Entity, event::Event, resource::Resource},
    platform::collections::HashMap,
    prelude::Component,
    time::{Timer, TimerMode},
};
use serde::Deserialize;

use crate::MobType;

const DEFAULT_DURATION: f32 = 1.0;
const DEFAULT_WEIGHT: f32 = 1.0;

/// Simple behaviors for mobs
#[derive(Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) enum MobBehavior {
    MoveDown,
    MoveLeft,
    MoveRight,
    BrakeHorizontal,
    BrakeVertical,
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
    #[serde(default)]
    timer: Timer,
}

impl MobBehaviorSequence {
    /// Gets the active block in the blocks vec using current_idx
    pub(super) fn get_active_block(&self) -> Option<&MobBehaviorBlock> {
        self.blocks.get(self.current_idx)
    }

    /// Initializes the timer based on the active block's duration
    pub(crate) fn init_timer(&self) -> Self {
        Self {
            blocks: self.blocks.clone(),
            execution_order: self.execution_order.clone(),
            current_idx: self.current_idx,
            timer: if let Some(active_block) = self.get_active_block() {
                Timer::new(
                    Duration::from_secs_f32(active_block.duration),
                    TimerMode::Once,
                )
            } else {
                Timer::default()
            },
        }
    }

    /// Updates the timer based on the delta time
    /// Updates current_idx to next block if timer is finished
    /// Sets the timer duration to the next block's duration if available
    pub(super) fn update_timer(&mut self, delta_time: f32) {
        self.timer.tick(Duration::from_secs_f32(delta_time));
        if self.timer.just_finished() {
            // Get the current idx of the next block using the execution order method
            self.current_idx = match self.execution_order {
                ExecutionOrder::Sequential => (self.current_idx + 1) % self.blocks.len(),
                ExecutionOrder::Random => {
                    let total_weight: f32 = self.blocks.iter().map(|block| block.weight).sum();
                    let mut random_value = rand::random::<f32>() * total_weight;
                    let mut current_idx = 0;

                    for (idx, block) in self.blocks.iter().enumerate() {
                        random_value -= block.weight;
                        if random_value <= 0.0 {
                            current_idx = idx;
                            break;
                        }
                    }
                    current_idx
                }
            };

            // Set the timer to the duration of the new active block
            if let Some(active_block) = self.get_active_block() {
                self.timer
                    .set_duration(Duration::from_secs_f32(active_block.duration));
                self.timer.reset();
            }
        }
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
