//! Behavior tree structure operations.
//!
//! Handles adding, deleting, moving, and changing types of behavior nodes.

use thetawave_mobs::BehaviorNodeType;

use crate::data::EditorSession;

use super::navigation::get_behavior_node_mut;

/// Add a default behavior tree (Forever with one Action child)
pub fn add_default_behavior_tree(session: &mut EditorSession) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        let mut action = toml::value::Table::new();
        action.insert(
            "type".to_string(),
            toml::Value::String("Action".to_string()),
        );
        action.insert(
            "name".to_string(),
            toml::Value::String("Movement".to_string()),
        );
        action.insert(
            "behaviors".to_string(),
            toml::Value::Array(vec![{
                let mut behavior = toml::value::Table::new();
                behavior.insert(
                    "action".to_string(),
                    toml::Value::String("MoveDown".to_string()),
                );
                toml::Value::Table(behavior)
            }]),
        );

        let mut root = toml::value::Table::new();
        root.insert(
            "type".to_string(),
            toml::Value::String("Forever".to_string()),
        );
        root.insert(
            "children".to_string(),
            toml::Value::Array(vec![toml::Value::Table(action)]),
        );

        mob.insert("behavior".to_string(), toml::Value::Table(root));
    }
}

/// Add a child to a control node (Forever, Sequence, Fallback)
pub fn add_behavior_child(session: &mut EditorSession, path: &[usize]) {
    if let Some(node) = get_behavior_node_mut(session, path)
        && let Some(table) = node.as_table_mut()
    {
        // Create a default Action child
        let mut action = toml::value::Table::new();
        action.insert(
            "type".to_string(),
            toml::Value::String("Action".to_string()),
        );
        action.insert(
            "name".to_string(),
            toml::Value::String("New Action".to_string()),
        );
        action.insert("behaviors".to_string(), toml::Value::Array(vec![]));

        if let Some(children) = table.get_mut("children").and_then(|v| v.as_array_mut()) {
            children.push(toml::Value::Table(action));
        } else {
            table.insert(
                "children".to_string(),
                toml::Value::Array(vec![toml::Value::Table(action)]),
            );
        }
    }
}

/// Delete a behavior node at the given path
pub fn delete_behavior_node(session: &mut EditorSession, path: &[usize]) {
    if path.is_empty() {
        // Can't delete root via this function
        return;
    }

    let parent_path = &path[..path.len() - 1];
    let index = path[path.len() - 1];

    if let Some(parent) = get_behavior_node_mut(session, parent_path)
        && let Some(table) = parent.as_table_mut()
    {
        let node_type = table.get("type").and_then(|v| v.as_str()).unwrap_or("");

        match node_type {
            "Forever" | "Sequence" | "Fallback" => {
                if let Some(children) = table.get_mut("children").and_then(|v| v.as_array_mut())
                    && index < children.len()
                {
                    children.remove(index);
                }
            }
            "While" => {
                if index == 0 {
                    table.remove("condition");
                }
                // Don't allow deleting the child - While always needs one
            }
            "IfThen" => {
                if index == 2 {
                    table.remove("else_child");
                }
                // Don't allow deleting condition or then_child
            }
            _ => {}
        }
    }
}

/// Move a behavior node up or down within its parent
pub fn move_behavior_node(session: &mut EditorSession, path: &[usize], direction: i32) {
    if path.is_empty() {
        return;
    }

    let parent_path = &path[..path.len() - 1];
    let index = path[path.len() - 1];

    if let Some(parent) = get_behavior_node_mut(session, parent_path)
        && let Some(table) = parent.as_table_mut()
    {
        let node_type = table.get("type").and_then(|v| v.as_str()).unwrap_or("");

        if matches!(node_type, "Forever" | "Sequence" | "Fallback")
            && let Some(children) = table.get_mut("children").and_then(|v| v.as_array_mut())
        {
            let new_index = (index as i32 + direction) as usize;
            if new_index < children.len() {
                children.swap(index, new_index);
            }
        }
    }
}

/// Change the type of a behavior node
pub fn change_behavior_node_type(
    session: &mut EditorSession,
    path: &[usize],
    new_type: BehaviorNodeType,
) {
    if let Some(node) = get_behavior_node_mut(session, path)
        && let Some(table) = node.as_table_mut()
    {
        let old_type_str = table.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let old_type: Option<BehaviorNodeType> = old_type_str.parse().map_err(|e| {
            bevy::log::debug!("Unknown behavior node type '{}': {}", old_type_str, e);
        }).ok();

        // Only proceed if type is actually changing
        if old_type == Some(new_type) {
            return;
        }

        // Update type
        table.insert(
            "type".to_string(),
            toml::Value::String(new_type.as_ref().to_string()),
        );

        // Handle structure changes based on old/new type categories
        let old_is_control = old_type
            .map(|t: BehaviorNodeType| t.is_control_node())
            .unwrap_or(false);
        let new_is_control = new_type.is_control_node();

        if old_is_control && new_is_control {
            // Keep children array as-is
        } else if old_is_control && !new_is_control {
            // Switching from control to leaf - remove children
            table.remove("children");
            add_fields_for_node_type(table, new_type);
        } else if !old_is_control && new_is_control {
            // Switching from leaf to control - add empty children array
            remove_leaf_fields(table);
            table.insert("children".to_string(), toml::Value::Array(vec![]));
        } else {
            // Switching between different leaf/special types
            remove_leaf_fields(table);
            add_fields_for_node_type(table, new_type);
        }
    }
}

/// Remove leaf-specific fields from a node
fn remove_leaf_fields(table: &mut toml::value::Table) {
    table.remove("seconds");
    table.remove("name");
    table.remove("behaviors");
    table.remove("trigger_type");
    table.remove("child");
    table.remove("condition");
    table.remove("then_child");
    table.remove("else_child");
    table.remove("children");
}

/// Add required fields for a new node type
fn add_fields_for_node_type(table: &mut toml::value::Table, node_type: BehaviorNodeType) {
    match node_type {
        BehaviorNodeType::Wait => {
            table.insert("seconds".to_string(), toml::Value::Float(1.0));
        }
        BehaviorNodeType::Action => {
            table.insert(
                "name".to_string(),
                toml::Value::String("New Action".to_string()),
            );
            table.insert("behaviors".to_string(), toml::Value::Array(vec![]));
        }
        BehaviorNodeType::Trigger => {
            table.insert(
                "trigger_type".to_string(),
                toml::Value::String(String::new()),
            );
        }
        BehaviorNodeType::While => {
            let mut child = toml::value::Table::new();
            child.insert(
                "type".to_string(),
                toml::Value::String("Action".to_string()),
            );
            child.insert("name".to_string(), toml::Value::String("Child".to_string()));
            child.insert("behaviors".to_string(), toml::Value::Array(vec![]));
            table.insert("child".to_string(), toml::Value::Table(child));
        }
        BehaviorNodeType::IfThen => {
            let mut cond = toml::value::Table::new();
            cond.insert("type".to_string(), toml::Value::String("Wait".to_string()));
            cond.insert("seconds".to_string(), toml::Value::Float(1.0));
            table.insert("condition".to_string(), toml::Value::Table(cond));

            let mut then = toml::value::Table::new();
            then.insert(
                "type".to_string(),
                toml::Value::String("Action".to_string()),
            );
            then.insert("name".to_string(), toml::Value::String("Then".to_string()));
            then.insert("behaviors".to_string(), toml::Value::Array(vec![]));
            table.insert("then_child".to_string(), toml::Value::Table(then));
        }
        BehaviorNodeType::Forever | BehaviorNodeType::Sequence | BehaviorNodeType::Fallback => {
            table.insert("children".to_string(), toml::Value::Array(vec![]));
        }
    }
}

// While/IfThen specific helpers

pub fn add_while_condition(session: &mut EditorSession, path: &[usize]) {
    if let Some(node) = get_behavior_node_mut(session, path)
        && let Some(table) = node.as_table_mut()
    {
        let mut cond = toml::value::Table::new();
        cond.insert("type".to_string(), toml::Value::String("Wait".to_string()));
        cond.insert("seconds".to_string(), toml::Value::Float(1.0));
        table.insert("condition".to_string(), toml::Value::Table(cond));
    }
}

pub fn add_while_child(session: &mut EditorSession, path: &[usize]) {
    if let Some(node) = get_behavior_node_mut(session, path)
        && let Some(table) = node.as_table_mut()
    {
        let mut child = toml::value::Table::new();
        child.insert(
            "type".to_string(),
            toml::Value::String("Action".to_string()),
        );
        child.insert("name".to_string(), toml::Value::String("Child".to_string()));
        child.insert("behaviors".to_string(), toml::Value::Array(vec![]));
        table.insert("child".to_string(), toml::Value::Table(child));
    }
}

pub fn add_if_then_condition(session: &mut EditorSession, path: &[usize]) {
    if let Some(node) = get_behavior_node_mut(session, path)
        && let Some(table) = node.as_table_mut()
    {
        let mut cond = toml::value::Table::new();
        cond.insert("type".to_string(), toml::Value::String("Wait".to_string()));
        cond.insert("seconds".to_string(), toml::Value::Float(1.0));
        table.insert("condition".to_string(), toml::Value::Table(cond));
    }
}

pub fn add_if_then_child(session: &mut EditorSession, path: &[usize]) {
    if let Some(node) = get_behavior_node_mut(session, path)
        && let Some(table) = node.as_table_mut()
    {
        let mut child = toml::value::Table::new();
        child.insert(
            "type".to_string(),
            toml::Value::String("Action".to_string()),
        );
        child.insert("name".to_string(), toml::Value::String("Then".to_string()));
        child.insert("behaviors".to_string(), toml::Value::Array(vec![]));
        table.insert("then_child".to_string(), toml::Value::Table(child));
    }
}

pub fn add_if_else_child(session: &mut EditorSession, path: &[usize]) {
    if let Some(node) = get_behavior_node_mut(session, path)
        && let Some(table) = node.as_table_mut()
    {
        let mut child = toml::value::Table::new();
        child.insert(
            "type".to_string(),
            toml::Value::String("Action".to_string()),
        );
        child.insert("name".to_string(), toml::Value::String("Else".to_string()));
        child.insert("behaviors".to_string(), toml::Value::Array(vec![]));
        table.insert("else_child".to_string(), toml::Value::Table(child));
    }
}

pub fn remove_if_else_child(session: &mut EditorSession, path: &[usize]) {
    if let Some(node) = get_behavior_node_mut(session, path)
        && let Some(table) = node.as_table_mut()
    {
        table.remove("else_child");
    }
}
