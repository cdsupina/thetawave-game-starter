//! Path traversal and navigation helpers for behavior tree nodes.

use crate::data::EditorSession;

/// Get a mutable reference to a behavior node at the given path
pub fn get_behavior_node_mut<'a>(
    session: &'a mut EditorSession,
    path: &[usize],
) -> Option<&'a mut toml::Value> {
    let mob = session.current_mob.as_mut()?.as_table_mut()?;
    let mut current = mob.get_mut("behavior")?;

    for &index in path {
        let table = current.as_table_mut()?;
        let node_type = table.get("type").and_then(|v| v.as_str())?;

        current = match node_type {
            "Forever" | "Sequence" | "Fallback" => table
                .get_mut("children")?
                .as_array_mut()?
                .get_mut(index)?,
            "While" => {
                if index == 0 {
                    table.get_mut("condition")?
                } else {
                    table.get_mut("child")?
                }
            }
            "IfThen" => match index {
                0 => table.get_mut("condition")?,
                1 => table.get_mut("then_child")?,
                2 => table.get_mut("else_child")?,
                _ => return None,
            },
            _ => return None, // Leaf nodes have no children
        };
    }

    Some(current)
}

/// Set a field on a behavior node at the given path
pub fn set_behavior_node_field(
    session: &mut EditorSession,
    path: &[usize],
    field: &str,
    value: toml::Value,
) {
    if let Some(node) = get_behavior_node_mut(session, path)
        && let Some(table) = node.as_table_mut()
    {
        table.insert(field.to_string(), value);
    }
}

/// Get the number of children for a control node at the given path
pub fn get_children_count(session: &EditorSession, path: &[usize]) -> usize {
    let Some(mob) = session.current_mob.as_ref().and_then(|v| v.as_table()) else {
        return 0;
    };

    let Some(behavior) = mob.get("behavior") else {
        return 0;
    };

    let mut current = behavior;

    for &index in path {
        let Some(table) = current.as_table() else {
            return 0;
        };
        let node_type = table.get("type").and_then(|v| v.as_str()).unwrap_or("");

        current = match node_type {
            "Forever" | "Sequence" | "Fallback" => {
                match table
                    .get("children")
                    .and_then(|v| v.as_array())
                    .and_then(|arr| arr.get(index))
                {
                    Some(c) => c,
                    None => return 0,
                }
            }
            "While" => {
                if index == 0 {
                    match table.get("condition") {
                        Some(c) => c,
                        None => return 0,
                    }
                } else {
                    match table.get("child") {
                        Some(c) => c,
                        None => return 0,
                    }
                }
            }
            "IfThen" => match index {
                0 => table.get("condition"),
                1 => table.get("then_child"),
                2 => table.get("else_child"),
                _ => None,
            }
            .unwrap_or(&toml::Value::Boolean(false)), // placeholder
            _ => return 0,
        };
    }

    // Now count children of the node at path
    let Some(table) = current.as_table() else {
        return 0;
    };
    let node_type = table.get("type").and_then(|v| v.as_str()).unwrap_or("");

    match node_type {
        "Forever" | "Sequence" | "Fallback" => table
            .get("children")
            .and_then(|v| v.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0),
        _ => 0,
    }
}
