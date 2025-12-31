use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, EnumIter, EnumString};

use super::data::MobBehaviorType;

/// Simple enum representing behavior node types for UI purposes.
/// This mirrors the variants of `BehaviorNodeData` without the associated data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, AsRefStr, EnumString)]
pub enum BehaviorNodeType {
    Forever,
    Sequence,
    Fallback,
    While,
    IfThen,
    Wait,
    Action,
    Trigger,
}

impl BehaviorNodeType {
    /// Check if this is a control node (has children array).
    pub fn is_control_node(&self) -> bool {
        matches!(self, Self::Forever | Self::Sequence | Self::Fallback)
    }

    /// Check if this is a leaf node (no children).
    pub fn is_leaf_node(&self) -> bool {
        matches!(self, Self::Wait | Self::Action | Self::Trigger)
    }
}

/// Represents a node in the behavior tree that can be deserialized from TOML
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum BehaviorNodeData {
    /// Control Flow Nodes

    /// Loops child forever
    Forever { children: Vec<BehaviorNodeData> },

    /// Runs children in sequence, fails if any fail
    Sequence { children: Vec<BehaviorNodeData> },

    /// Runs children until one succeeds (selector pattern)
    Fallback { children: Vec<BehaviorNodeData> },

    /// Repeats child until it fails, optionally with condition
    While {
        condition: Option<Box<BehaviorNodeData>>,
        child: Box<BehaviorNodeData>,
    },

    /// Conditional execution
    IfThen {
        condition: Box<BehaviorNodeData>,
        then_child: Box<BehaviorNodeData>,
        else_child: Option<Box<BehaviorNodeData>>,
    },

    /// Action Nodes

    /// Waits for specified seconds
    Wait { seconds: f32 },

    /// Spawns a named action with behaviors (maps to spawn_named)
    Action {
        name: String,
        behaviors: Vec<MobBehaviorType>,
    },

    /// Executes a trigger (for future use)
    Trigger { trigger_type: String },
}
