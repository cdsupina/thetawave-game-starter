use std::path::PathBuf;

use bevy::prelude::*;

/// The type of file being edited
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FileType {
    #[default]
    Mob,
    MobPatch,
}

/// Stores the undo/redo history for the current editing session
#[derive(Default)]
pub struct UndoHistory {
    past: Vec<toml::Value>,
    future: Vec<toml::Value>,
}

impl UndoHistory {
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
    /// For .mobpatch files, this is the patch content only
    pub current_mob: Option<toml::Value>,

    /// Original mob value when loaded (for detecting actual modifications)
    pub original_mob: Option<toml::Value>,

    /// Merged mob data for preview (base + patch for .mobpatch files)
    /// For .mob files, this is None (use current_mob directly)
    pub merged_for_preview: Option<toml::Value>,

    /// Base mob data (for .mobpatch files only)
    /// Used to show inherited values in the properties panel
    pub base_mob: Option<toml::Value>,

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

    /// Selected jointed mob index for editing
    pub selected_jointed_mob: Option<usize>,

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
            merged_for_preview: None,
            base_mob: None,
            current_path: None,
            file_type: FileType::Mob,
            is_modified: false,
            history: UndoHistory::default(),
            selected_collider: None,
            selected_jointed_mob: None,
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

    /// Check if the current file is from the extended assets directory
    /// (i.e., in thetawave-test-game/assets/mobs/)
    pub fn is_extended_mob(&self) -> bool {
        self.current_path
            .as_ref()
            .map(|p| {
                let path_str = p.to_string_lossy();
                path_str.contains("thetawave-test-game")
            })
            .unwrap_or(false)
    }

    /// Check if extended sprites should be available for the current file
    /// Extended sprites are available for:
    /// - Extended mobs (in thetawave-test-game/assets/mobs/)
    /// - All mobpatches (they can override with extended sprites)
    pub fn can_use_extended_sprites(&self) -> bool {
        self.file_type == FileType::MobPatch || self.is_extended_mob()
    }

    /// Create a new empty mob with defaults
    pub fn new_mob(name: &str) -> toml::Value {
        let mut table = toml::value::Table::new();
        table.insert("name".to_string(), toml::Value::String(name.to_string()));
        // Default sprite path - user should update this
        table.insert(
            "sprite".to_string(),
            toml::Value::String(format!("media/aseprite/{}_mob.aseprite", name.to_lowercase().replace(' ', "_"))),
        );
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

    /// Get the mob data to use for preview rendering
    /// Returns merged data for .mobpatch files, or current_mob for .mob files
    pub fn mob_for_preview(&self) -> Option<&toml::Value> {
        self.merged_for_preview.as_ref().or(self.current_mob.as_ref())
    }
}
