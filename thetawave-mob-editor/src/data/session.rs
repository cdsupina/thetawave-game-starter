use std::path::PathBuf;

use bevy::prelude::*;

/// The type of file being edited
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FileType {
    #[default]
    Mob,
    MobPatch,
}

impl FileType {
    pub fn extension(&self) -> &'static str {
        match self {
            FileType::Mob => "mob",
            FileType::MobPatch => "mobpatch",
        }
    }
}

/// Stores the undo/redo history for the current editing session
#[derive(Default)]
pub struct UndoHistory {
    past: Vec<toml::Value>,
    future: Vec<toml::Value>,
    max_history: usize,
}

impl UndoHistory {
    pub fn new(max_history: usize) -> Self {
        Self {
            past: Vec::new(),
            future: Vec::new(),
            max_history,
        }
    }

    pub fn push(&mut self, state: toml::Value) {
        self.past.push(state);
        self.future.clear();
        if self.past.len() > self.max_history {
            self.past.remove(0);
        }
    }

    pub fn undo(&mut self, current: &toml::Value) -> Option<toml::Value> {
        if let Some(prev) = self.past.pop() {
            self.future.push(current.clone());
            Some(prev)
        } else {
            None
        }
    }

    pub fn redo(&mut self, current: &toml::Value) -> Option<toml::Value> {
        if let Some(next) = self.future.pop() {
            self.past.push(current.clone());
            Some(next)
        } else {
            None
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.past.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }

    pub fn clear(&mut self) {
        self.past.clear();
        self.future.clear();
    }
}

/// Resource tracking the current editor session state
#[derive(Resource)]
pub struct EditorSession {
    /// Currently loaded mob as raw TOML value (allows flexible editing)
    pub current_mob: Option<toml::Value>,

    /// Original mob value when loaded (for detecting actual modifications)
    pub original_mob: Option<toml::Value>,

    /// Path to the current file being edited
    pub current_path: Option<PathBuf>,

    /// Whether the file is .mob or .mobpatch
    pub file_type: FileType,

    /// Has unsaved modifications
    pub is_modified: bool,

    /// Undo/redo history
    pub history: UndoHistory,

    /// Selected collider index for editing
    pub selected_collider: Option<usize>,

    /// Selected behavior node path for editing
    pub selected_behavior_node: Option<Vec<usize>>,

    /// Status message to display
    pub status_message: Option<(String, f64)>,
}

impl Default for EditorSession {
    fn default() -> Self {
        Self {
            current_mob: None,
            original_mob: None,
            current_path: None,
            file_type: FileType::Mob,
            is_modified: false,
            history: UndoHistory::new(50),
            selected_collider: None,
            selected_behavior_node: None,
            status_message: None,
        }
    }
}

impl EditorSession {
    /// Check if current_mob differs from original_mob and update is_modified
    pub fn check_modified(&mut self) {
        self.is_modified = match (&self.current_mob, &self.original_mob) {
            (Some(current), Some(original)) => current != original,
            (None, None) => false,
            _ => true,
        };
    }

    /// Mark the current state as modified (call before making changes)
    pub fn mark_modified(&mut self) {
        self.is_modified = true;
    }

    /// Set a status message that will be displayed temporarily
    pub fn set_status(&mut self, message: impl Into<String>, time: &Time) {
        self.status_message = Some((message.into(), time.elapsed_secs_f64() + 3.0));
    }

    /// Clear expired status messages
    pub fn update_status(&mut self, time: &Time) {
        if let Some((_, expiry)) = &self.status_message {
            if time.elapsed_secs_f64() > *expiry {
                self.status_message = None;
            }
        }
    }

    /// Get the mob name from the current TOML data
    pub fn get_mob_name(&self) -> Option<String> {
        self.current_mob.as_ref().and_then(|v| {
            v.get("name")
                .and_then(|n| n.as_str())
                .map(|s| s.to_string())
        })
    }

    /// Create a new empty mob with defaults
    pub fn new_mob(name: &str) -> toml::Value {
        let mut table = toml::value::Table::new();
        table.insert("name".to_string(), toml::Value::String(name.to_string()));
        table.insert("spawnable".to_string(), toml::Value::Boolean(true));
        table.insert("health".to_string(), toml::Value::Integer(50));

        // Default collider
        let mut collider = toml::value::Table::new();
        let mut shape = toml::value::Table::new();
        shape.insert(
            "Rectangle".to_string(),
            toml::Value::Array(vec![
                toml::Value::Float(10.0),
                toml::Value::Float(10.0),
            ]),
        );
        collider.insert("shape".to_string(), toml::Value::Table(shape));
        collider.insert(
            "position".to_string(),
            toml::Value::Array(vec![toml::Value::Float(0.0), toml::Value::Float(0.0)]),
        );
        collider.insert("rotation".to_string(), toml::Value::Float(0.0));

        table.insert(
            "colliders".to_string(),
            toml::Value::Array(vec![toml::Value::Table(collider)]),
        );

        toml::Value::Table(table)
    }
}
