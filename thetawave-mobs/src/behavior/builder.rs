use bevy_behave::{prelude::Tree, Behave, behave};

use super::{
    data::MobBehaviorComponent,
    toml_data::BehaviorNodeData,
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
            let component = MobBehaviorComponent { behaviors: behaviors.clone() };
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

