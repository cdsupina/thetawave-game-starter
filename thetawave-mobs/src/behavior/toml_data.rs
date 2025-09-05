use bevy::platform::collections::HashMap;
use serde::Deserialize;

use super::data::MobBehaviorType;

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

/// Simple deserializable version of behavior actions that converts to MobBehaviorType
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

impl BehaviorActionData {
    /// Convert BehaviorActionData to MobBehaviorType
    pub fn to_mob_behavior_type(&self) -> MobBehaviorType {
        use bevy::{math::Vec2, time::{Timer, TimerMode}};
        
        match self {
            // Simple movement actions
            BehaviorActionData::MoveDown => MobBehaviorType::MoveDown,
            BehaviorActionData::MoveUp => MobBehaviorType::MoveUp,
            BehaviorActionData::MoveLeft => MobBehaviorType::MoveLeft,
            BehaviorActionData::MoveRight => MobBehaviorType::MoveRight,
            BehaviorActionData::BrakeHorizontal => MobBehaviorType::BrakeHorizontal,
            BehaviorActionData::BrakeAngular => MobBehaviorType::BrakeAngular,
            
            // Position-based movement
            BehaviorActionData::MoveTo { x, y } => {
                MobBehaviorType::MoveTo(Vec2::new(*x, *y))
            }
            
            // Targeting actions
            BehaviorActionData::FindPlayerTarget => MobBehaviorType::FindPlayerTarget,
            BehaviorActionData::MoveToTarget => MobBehaviorType::MoveToTarget,
            BehaviorActionData::RotateToTarget => MobBehaviorType::RotateToTarget,
            BehaviorActionData::MoveForward => MobBehaviorType::MoveForward,
            BehaviorActionData::LoseTarget => MobBehaviorType::LoseTarget,
            
            // Spawning actions
            BehaviorActionData::SpawnMob { keys } => {
                MobBehaviorType::SpawnMob { keys: keys.clone() }
            }
            
            BehaviorActionData::SpawnProjectile { keys } => {
                MobBehaviorType::SpawnProjectile { keys: keys.clone() }
            }
            
            // Timing actions
            BehaviorActionData::DoForTime { seconds } => {
                MobBehaviorType::DoForTime(Timer::from_seconds(*seconds, TimerMode::Once))
            }
            
            // Communication actions
            BehaviorActionData::TransmitMobBehavior { mob_type, behaviors } => {
                let converted_behaviors: Vec<MobBehaviorType> = behaviors.iter()
                    .map(|b| b.to_mob_behavior_type())
                    .collect();
                
                // Convert to &'static str by leaking - safe for behavior trees loaded once at startup
                let static_mob_type: &'static str = Box::leak(mob_type.clone().into_boxed_str());
                MobBehaviorType::TransmitMobBehavior {
                    mob_type: static_mob_type,
                    behaviors: converted_behaviors,
                }
            }
            
            // Joint actions (for future use)
            BehaviorActionData::RotateJointsClockwise { keys } => {
                MobBehaviorType::RotateJointsClockwise { keys: keys.clone() }
            }
        }
    }
}


