//! TransmitMobBehavior nested behaviors helpers.
//!
//! Handles operations on the nested behaviors array within TransmitMobBehavior actions.

use thetawave_mobs::MobBehaviorVariant;

use crate::data::EditorSession;

use super::navigation::get_behavior_node_mut;

/// Add a nested behavior to a TransmitMobBehavior action.
pub fn add_transmit_nested_behavior(
    session: &mut EditorSession,
    path: &[usize],
    behavior_index: usize,
) {
    if let Some(node) = get_behavior_node_mut(session, path)
        && let Some(table) = node.as_table_mut()
        && let Some(behaviors) = table.get_mut("behaviors").and_then(|v| v.as_array_mut())
        && let Some(behavior) = behaviors
            .get_mut(behavior_index)
            .and_then(|v| v.as_table_mut())
    {
        let mut new_nested = toml::value::Table::new();
        new_nested.insert(
            "action".to_string(),
            toml::Value::String("MoveDown".to_string()),
        );

        if let Some(nested_arr) = behavior.get_mut("behaviors").and_then(|v| v.as_array_mut()) {
            nested_arr.push(toml::Value::Table(new_nested));
        } else {
            behavior.insert(
                "behaviors".to_string(),
                toml::Value::Array(vec![toml::Value::Table(new_nested)]),
            );
        }
    }
}

/// Delete a nested behavior from a TransmitMobBehavior action.
pub fn delete_transmit_nested_behavior(
    session: &mut EditorSession,
    path: &[usize],
    behavior_index: usize,
    nested_index: usize,
) {
    if let Some(node) = get_behavior_node_mut(session, path)
        && let Some(table) = node.as_table_mut()
        && let Some(behaviors) = table.get_mut("behaviors").and_then(|v| v.as_array_mut())
        && let Some(behavior) = behaviors
            .get_mut(behavior_index)
            .and_then(|v| v.as_table_mut())
        && let Some(nested_arr) = behavior.get_mut("behaviors").and_then(|v| v.as_array_mut())
        && nested_index < nested_arr.len()
    {
        nested_arr.remove(nested_index);
    }
}

/// Move a nested behavior up or down within a TransmitMobBehavior action.
pub fn move_transmit_nested_behavior(
    session: &mut EditorSession,
    path: &[usize],
    behavior_index: usize,
    nested_index: usize,
    direction: i32,
) {
    if let Some(node) = get_behavior_node_mut(session, path)
        && let Some(table) = node.as_table_mut()
        && let Some(behaviors) = table.get_mut("behaviors").and_then(|v| v.as_array_mut())
        && let Some(behavior) = behaviors
            .get_mut(behavior_index)
            .and_then(|v| v.as_table_mut())
        && let Some(nested_arr) = behavior.get_mut("behaviors").and_then(|v| v.as_array_mut())
    {
        let new_index = (nested_index as i32 + direction) as usize;
        if new_index < nested_arr.len() {
            nested_arr.swap(nested_index, new_index);
        }
    }
}

/// Change the type of a nested behavior in a TransmitMobBehavior action.
pub fn change_transmit_nested_behavior_type(
    session: &mut EditorSession,
    path: &[usize],
    behavior_index: usize,
    nested_index: usize,
    new_action: MobBehaviorVariant,
) {
    if let Some(node) = get_behavior_node_mut(session, path)
        && let Some(table) = node.as_table_mut()
        && let Some(behaviors) = table.get_mut("behaviors").and_then(|v| v.as_array_mut())
        && let Some(behavior) = behaviors
            .get_mut(behavior_index)
            .and_then(|v| v.as_table_mut())
        && let Some(nested_arr) = behavior.get_mut("behaviors").and_then(|v| v.as_array_mut())
        && let Some(nested) = nested_arr.get_mut(nested_index).and_then(|v| v.as_table_mut())
    {
        // Clear old fields except action
        let keys_to_remove: Vec<_> = nested
            .keys()
            .filter(|k| *k != "action")
            .cloned()
            .collect();
        for key in keys_to_remove {
            nested.remove(&key);
        }

        // Set new action type
        nested.insert(
            "action".to_string(),
            toml::Value::String(new_action.as_ref().to_string()),
        );

        // Add default parameters
        match new_action {
            MobBehaviorVariant::MoveTo => {
                nested.insert("x".to_string(), toml::Value::Float(0.0));
                nested.insert("y".to_string(), toml::Value::Float(0.0));
            }
            MobBehaviorVariant::DoForTime => {
                nested.insert("seconds".to_string(), toml::Value::Float(1.0));
            }
            _ => {}
        }
    }
}

/// Set a parameter on a nested behavior in a TransmitMobBehavior action.
pub fn set_transmit_nested_param(
    session: &mut EditorSession,
    path: &[usize],
    behavior_index: usize,
    nested_index: usize,
    param: &str,
    value: toml::Value,
) {
    if let Some(node) = get_behavior_node_mut(session, path)
        && let Some(table) = node.as_table_mut()
        && let Some(behaviors) = table.get_mut("behaviors").and_then(|v| v.as_array_mut())
        && let Some(behavior) = behaviors
            .get_mut(behavior_index)
            .and_then(|v| v.as_table_mut())
        && let Some(nested_arr) = behavior.get_mut("behaviors").and_then(|v| v.as_array_mut())
        && let Some(nested) = nested_arr.get_mut(nested_index).and_then(|v| v.as_table_mut())
    {
        nested.insert(param.to_string(), value);
    }
}
