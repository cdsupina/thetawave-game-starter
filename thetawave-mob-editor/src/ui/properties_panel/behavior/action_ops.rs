//! Action behavior manipulation helpers.
//!
//! Handles CRUD operations for behaviors within Action nodes.

use thetawave_mobs::MobBehaviorVariant;

use crate::data::EditorSession;

use super::navigation::get_behavior_node_mut;

/// Add a new behavior to an Action node.
pub fn add_action_behavior(session: &mut EditorSession, path: &[usize]) {
    if let Some(node) = get_behavior_node_mut(session, path)
        && let Some(table) = node.as_table_mut()
    {
        let mut behavior = toml::value::Table::new();
        behavior.insert(
            "action".to_string(),
            toml::Value::String("MoveDown".to_string()),
        );

        if let Some(behaviors) = table.get_mut("behaviors").and_then(|v| v.as_array_mut()) {
            behaviors.push(toml::Value::Table(behavior));
        } else {
            table.insert(
                "behaviors".to_string(),
                toml::Value::Array(vec![toml::Value::Table(behavior)]),
            );
        }
    }
}

/// Delete a behavior from an Action node.
pub fn delete_action_behavior(session: &mut EditorSession, path: &[usize], index: usize) {
    if let Some(node) = get_behavior_node_mut(session, path)
        && let Some(table) = node.as_table_mut()
        && let Some(behaviors) = table.get_mut("behaviors").and_then(|v| v.as_array_mut())
        && index < behaviors.len()
    {
        behaviors.remove(index);
    }
}

/// Move a behavior up or down within an Action node.
pub fn move_action_behavior(
    session: &mut EditorSession,
    path: &[usize],
    index: usize,
    direction: i32,
) {
    if let Some(node) = get_behavior_node_mut(session, path)
        && let Some(table) = node.as_table_mut()
        && let Some(behaviors) = table.get_mut("behaviors").and_then(|v| v.as_array_mut())
    {
        let new_index = (index as i32 + direction) as usize;
        if new_index < behaviors.len() {
            behaviors.swap(index, new_index);
        }
    }
}

/// Change the type of a behavior in an Action node.
pub fn change_action_behavior_type(
    session: &mut EditorSession,
    path: &[usize],
    index: usize,
    new_action: MobBehaviorVariant,
) {
    if let Some(node) = get_behavior_node_mut(session, path)
        && let Some(table) = node.as_table_mut()
        && let Some(behaviors) = table.get_mut("behaviors").and_then(|v| v.as_array_mut())
        && let Some(behavior) = behaviors.get_mut(index).and_then(|v| v.as_table_mut())
    {
        // Clear old fields except action
        let keys_to_remove: Vec<_> = behavior
            .keys()
            .filter(|k| *k != "action")
            .cloned()
            .collect();
        for key in keys_to_remove {
            behavior.remove(&key);
        }

        // Set new action type
        behavior.insert(
            "action".to_string(),
            toml::Value::String(new_action.as_ref().to_string()),
        );

        // Add default parameters for actions that need them
        match new_action {
            MobBehaviorVariant::MoveTo => {
                behavior.insert("x".to_string(), toml::Value::Float(0.0));
                behavior.insert("y".to_string(), toml::Value::Float(0.0));
            }
            MobBehaviorVariant::DoForTime => {
                behavior.insert("seconds".to_string(), toml::Value::Float(1.0));
            }
            _ => {}
        }
    }
}

/// Set a parameter on a behavior in an Action node.
pub fn set_action_behavior_param(
    session: &mut EditorSession,
    path: &[usize],
    index: usize,
    param: &str,
    value: toml::Value,
) {
    if let Some(node) = get_behavior_node_mut(session, path)
        && let Some(table) = node.as_table_mut()
        && let Some(behaviors) = table.get_mut("behaviors").and_then(|v| v.as_array_mut())
        && let Some(behavior) = behaviors.get_mut(index).and_then(|v| v.as_table_mut())
    {
        behavior.insert(param.to_string(), value);
    }
}

/// Remove a parameter from a behavior in an Action node.
pub fn remove_action_behavior_param(
    session: &mut EditorSession,
    path: &[usize],
    index: usize,
    param: &str,
) {
    if let Some(node) = get_behavior_node_mut(session, path)
        && let Some(table) = node.as_table_mut()
        && let Some(behaviors) = table.get_mut("behaviors").and_then(|v| v.as_array_mut())
        && let Some(behavior) = behaviors.get_mut(index).and_then(|v| v.as_table_mut())
    {
        behavior.remove(param);
    }
}
