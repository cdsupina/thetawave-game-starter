use bevy::platform::collections::HashMap;
use serde::Deserialize;

/// Root structure for deserializing mob_behaviors.toml
#[derive(Deserialize, Debug)]
pub struct MobBehaviorsTomlData {
    pub behaviors: HashMap<String, BehaviorNodeData>,
}

/// Represents a node in the behavior tree that can be deserialized from TOML
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
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
        behaviors: Vec<BehaviorActionData> 
    },
    
    /// Executes a trigger (for future use)
    Trigger {
        trigger_type: String
    },
}

/// Individual behavior actions that can be performed
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "action")]
pub enum BehaviorActionData {
    // Movement behaviors
    MoveDown,
    MoveUp,
    MoveLeft,
    MoveRight,
    BrakeHorizontal,
    BrakeAngular,
    
    /// Move to specific position
    MoveTo { 
        x: f32, 
        y: f32 
    },
    
    // Targeting behaviors
    FindPlayerTarget,
    MoveToTarget,
    RotateToTarget,
    MoveForward,
    LoseTarget,
    
    // Spawning behaviors
    SpawnMob { 
        keys: Option<Vec<String>> 
    },
    SpawnProjectile { 
        keys: Option<Vec<String>> 
    },
    
    // Timing behaviors
    DoForTime { 
        seconds: f32 
    },
    
    // Communication behaviors
    TransmitMobBehavior { 
        mob_type: String,
        behaviors: Vec<BehaviorActionData> 
    },
    
    // Joint behaviors (for future use)
    RotateJointsClockwise { 
        keys: Vec<String> 
    },
}

