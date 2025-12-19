use serde::Deserialize;

use super::data::MobBehaviorType;

/// Represents a node in the behavior tree that can be deserialized from TOML
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum BehaviorNodeData {
    /// Control Flow Nodes
    
    /// Loops child forever
    Forever { 
        children: Vec<BehaviorNodeData> 
    },
    
    /// Runs children in sequence, fails if any fail
    Sequence { 
        children: Vec<BehaviorNodeData> 
    },
    
    /// Runs children until one succeeds (selector pattern)
    Fallback { 
        children: Vec<BehaviorNodeData> 
    },
    
    /// Repeats child until it fails, optionally with condition
    While { 
        condition: Option<Box<BehaviorNodeData>>,
        child: Box<BehaviorNodeData> 
    },
    
    /// Conditional execution
    IfThen {
        condition: Box<BehaviorNodeData>,
        then_child: Box<BehaviorNodeData>,
        else_child: Option<Box<BehaviorNodeData>>
    },
    
    /// Action Nodes
    
    /// Waits for specified seconds
    Wait { 
        seconds: f32 
    },
    
    /// Spawns a named action with behaviors (maps to spawn_named)
    Action { 
        name: String,
        behaviors: Vec<MobBehaviorType> 
    },
    
    /// Executes a trigger (for future use)
    Trigger {
        trigger_type: String
    },
}





