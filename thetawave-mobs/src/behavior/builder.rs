use bevy::{
    math::Vec2,
    time::{Timer, TimerMode},
};
use bevy_behave::{prelude::Tree, Behave, behave};

use super::{
    data::{MobBehaviorComponent, MobBehaviorType},
    toml_data::{BehaviorActionData, BehaviorNodeData},
};

/// Converts TOML behavior node data into bevy_behave Tree structures  
pub fn build_behavior_tree(node: &BehaviorNodeData) -> Tree<Behave> {
    match node {
        BehaviorNodeData::Forever { children } => {
            build_control_node("Forever", children)
        }
        
        BehaviorNodeData::Sequence { children } => {
            build_control_node("Sequence", children)
        }
        
        BehaviorNodeData::Fallback { children } => {
            build_control_node("Fallback", children)
        }
        
        BehaviorNodeData::While { child, condition: _condition } => {
            // For now, implement simple While without condition
            // TODO: Add condition support when needed
            build_control_node("While", &[child.as_ref().clone()])
        }
        
        BehaviorNodeData::IfThen { condition: _condition, then_child, else_child: _else_child } => {
            // For now, implement simple behavior - just use then_child
            // TODO: Add full IfThen support when needed
            build_behavior_tree(then_child)
        }
        
        BehaviorNodeData::Wait { seconds } => {
            behave! { Behave::Wait(*seconds) }
        }
        
        BehaviorNodeData::Action { name, behaviors } => {
            let mob_behaviors = convert_behavior_actions(behaviors);
            let component = MobBehaviorComponent { behaviors: mob_behaviors };
            // Convert name to &'static str by leaking it
            let static_name: &'static str = Box::leak(name.clone().into_boxed_str());
            behave! { Behave::spawn_named(static_name, component) }
        }
        
        BehaviorNodeData::Trigger { trigger_type: _trigger_type } => {
            // For future use - triggers are not currently implemented in our system
            // For now, create a no-op wait
            behave! { Behave::Wait(0.0) }
        }
    }
}

/// Helper function to build control flow nodes dynamically
fn build_control_node(node_type: &str, children: &[BehaviorNodeData]) -> Tree<Behave> {
    // Build the appropriate control node based on type
    match node_type {
        "Forever" => {
            if children.len() == 1 {
                let child_tree = build_behavior_tree(&children[0]);
                behave! { 
                    Behave::Forever => { 
                        @ child_tree 
                    } 
                }
            } else {
                // Multiple children - wrap in sequence first
                let child_trees = build_children_trees(children);
                behave! { 
                    Behave::Forever => {
                        Behave::Sequence => {
                            ... child_trees
                        }
                    }
                }
            }
        }
        "Sequence" => {
            let child_trees = build_children_trees(children);
            behave! { Behave::Sequence => { ... child_trees } }
        }
        "Fallback" => {
            let child_trees = build_children_trees(children);
            behave! { Behave::Fallback => { ... child_trees } }
        }
        "While" => {
            if !children.is_empty() {
                let child_tree = build_behavior_tree(&children[0]);
                behave! { Behave::While => { @ child_tree } }
            } else {
                behave! { Behave::Wait(0.0) }
            }
        }
        _ => {
            behave! { Behave::Wait(0.0) } // Fallback
        }
    }
}

/// Build a vector of child trees from node data
fn build_children_trees(children: &[BehaviorNodeData]) -> Vec<Tree<Behave>> {
    children.iter().map(build_behavior_tree).collect()
}

/// Convert BehaviorActionData to MobBehaviorType
fn convert_behavior_actions(actions: &[BehaviorActionData]) -> Vec<MobBehaviorType> {
    actions.iter().map(convert_single_action).collect()
}

/// Convert a single BehaviorActionData to MobBehaviorType
fn convert_single_action(action: &BehaviorActionData) -> MobBehaviorType {
    match action {
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
            MobBehaviorType::SpawnMob(keys.as_ref().map(|k| k.iter().map(|s| s.clone()).collect()))
        }
        
        BehaviorActionData::SpawnProjectile { keys } => {
            MobBehaviorType::SpawnProjectile(keys.as_ref().map(|k| k.iter().map(|s| s.clone()).collect()))
        }
        
        // Timing actions
        BehaviorActionData::DoForTime { seconds } => {
            MobBehaviorType::DoForTime(Timer::from_seconds(*seconds, TimerMode::Once))
        }
        
        // Communication actions
        BehaviorActionData::TransmitMobBehavior { mob_type, behaviors } => {
            let converted_behaviors = convert_behavior_actions(behaviors);
            // Need to leak the string to make it 'static since TransmitMobBehavior requires &'static str
            // This is safe because behavior trees are typically loaded once at startup
            let static_mob_type: &'static str = Box::leak(mob_type.clone().into_boxed_str());
            MobBehaviorType::TransmitMobBehavior {
                mob_type: static_mob_type,
                behaviors: converted_behaviors,
            }
        }
        
        // Joint actions (for future use)
        BehaviorActionData::RotateJointsClockwise { keys } => {
            MobBehaviorType::RotateJointsClockwise(keys.clone())
        }
    }
}

