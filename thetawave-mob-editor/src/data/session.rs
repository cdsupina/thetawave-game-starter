//! Editor session state management.
//!
//! Contains [`EditorSession`] which tracks the current editing session,
//! including the loaded mob data, modification state, and status log.

use std::collections::VecDeque;
use std::path::PathBuf;

use bevy::prelude::{Resource, Time};

use crate::plugin::EditorConfig;

// =============================================================================
// Constants
// =============================================================================

/// Maximum number of log entries to keep in the status log
const STATUS_LOG_MAX_ENTRIES: usize = 50;

/// The type of file being edited
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FileType {
    #[default]
    Mob,
    MobPatch,
}

/// Status message severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusLevel {
    Success,
    Warning,
    Error,
}

impl StatusLevel {
    pub fn color(&self) -> bevy_egui::egui::Color32 {
        match self {
            StatusLevel::Success => bevy_egui::egui::Color32::from_rgb(100, 200, 100),
            StatusLevel::Warning => bevy_egui::egui::Color32::YELLOW,
            StatusLevel::Error => bevy_egui::egui::Color32::from_rgb(255, 100, 100),
        }
    }
}

/// A single log entry
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub text: String,
    pub level: StatusLevel,
    pub timestamp: f64,
}

/// Scrolling log of status messages
///
/// Uses VecDeque for O(1) removal from the front when at capacity
pub struct StatusLog {
    entries: VecDeque<LogEntry>,
    max_entries: usize,
    /// Whether the log panel is expanded
    pub expanded: bool,
}

impl Default for StatusLog {
    fn default() -> Self {
        Self {
            entries: VecDeque::new(),
            max_entries: STATUS_LOG_MAX_ENTRIES,
            expanded: false,
        }
    }
}

impl StatusLog {
    /// Add a new entry to the log.
    ///
    /// If at capacity, the oldest entry is removed (O(1) with VecDeque).
    pub fn push(&mut self, text: impl Into<String>, level: StatusLevel, timestamp: f64) {
        self.entries.push_back(LogEntry {
            text: text.into(),
            level,
            timestamp,
        });
        // Remove oldest entries if over capacity (O(1) with VecDeque)
        while self.entries.len() > self.max_entries {
            self.entries.pop_front();
        }
    }

    /// Iterate over all log entries in order.
    pub fn iter(&self) -> impl Iterator<Item = &LogEntry> {
        self.entries.iter()
    }

    /// Get the most recent entry.
    pub fn last(&self) -> Option<&LogEntry> {
        self.entries.back()
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Check if log is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
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

    /// Expected base mob relative path (for .mobpatch files)
    /// e.g., "xhitara/spitter.mob" - set even if base mob not found
    pub expected_base_path: Option<String>,

    /// Path to the current file being edited
    pub current_path: Option<PathBuf>,

    /// Whether the file is .mob or .mobpatch
    pub file_type: FileType,

    /// Has unsaved modifications
    pub is_modified: bool,

    /// Selected collider index for editing
    pub selected_collider: Option<usize>,

    /// Selected jointed mob index for editing
    pub selected_jointed_mob: Option<usize>,

    /// Selected behavior node path for editing
    pub selected_behavior_node: Option<Vec<usize>>,

    /// Status log for messages
    pub log: StatusLog,

    /// Flag to signal that the preview needs to be rebuilt
    /// Set by properties panel when preview-affecting fields change
    pub preview_needs_rebuild: bool,

    /// Flag to signal that the app should exit after the current save completes
    /// Set by the unsaved changes dialog when "Save & Exit" is clicked
    pub pending_exit_after_save: bool,
}

impl Default for EditorSession {
    fn default() -> Self {
        Self {
            current_mob: None,
            original_mob: None,
            merged_for_preview: None,
            base_mob: None,
            expected_base_path: None,
            current_path: None,
            file_type: FileType::Mob,
            is_modified: false,
            selected_collider: None,
            selected_jointed_mob: None,
            selected_behavior_node: None,
            log: StatusLog::default(),
            preview_needs_rebuild: false,
            pending_exit_after_save: false,
        }
    }
}

impl EditorSession {
    /// Check if current_mob differs from original_mob and update is_modified
    ///
    /// Uses type-aware comparison to handle Integer/Float type differences
    pub fn check_modified(&mut self) {
        self.is_modified = match (&self.current_mob, &self.original_mob) {
            (Some(current), Some(original)) => !Self::values_equal(current, original),
            (None, None) => false,
            _ => true,
        };
    }

    /// Check if a specific field has been modified since last save
    ///
    /// Compares the field value in current_mob vs original_mob
    /// Uses type-aware comparison to handle Integer/Float type differences
    pub fn is_field_modified(&self, field_name: &str) -> bool {
        match (&self.current_mob, &self.original_mob) {
            (Some(current), Some(original)) => {
                let current_val = current.get(field_name);
                let original_val = original.get(field_name);
                match (current_val, original_val) {
                    (Some(a), Some(b)) => !Self::values_equal(a, b),
                    (None, None) => false,
                    _ => true, // One exists, other doesn't
                }
            }
            _ => false,
        }
    }

    /// Compare two TOML values with type coercion for numbers
    ///
    /// This handles the case where a value might be stored as Integer in the
    /// original file but set as Float by the editor (or vice versa).
    pub fn values_equal(a: &toml::Value, b: &toml::Value) -> bool {
        use toml::Value;
        match (a, b) {
            // Same types - compare directly
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Datetime(a), Value::Datetime(b)) => a == b,

            // Integer/Float coercion - compare as f64
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
            (Value::Integer(a), Value::Float(b)) | (Value::Float(b), Value::Integer(a)) => {
                (*a as f64 - b).abs() < f64::EPSILON
            }

            // Arrays - compare element by element
            (Value::Array(a), Value::Array(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| Self::values_equal(x, y))
            }

            // Tables - compare key by key
            (Value::Table(a), Value::Table(b)) => {
                a.len() == b.len()
                    && a.keys().all(|k| {
                        b.get(k)
                            .map(|bv| Self::values_equal(a.get(k).unwrap(), bv))
                            .unwrap_or(false)
                    })
            }

            // Different types (and not Integer/Float pair) - not equal
            _ => false,
        }
    }

    /// Check if an array item is new or modified (for decorations, colliders, etc.)
    ///
    /// Returns true if:
    /// - The item is new (index didn't exist in original)
    /// - The item was modified (value differs from original)
    pub fn is_array_item_modified(&self, field_name: &str, index: usize) -> bool {
        match (&self.current_mob, &self.original_mob) {
            (Some(current), Some(original)) => {
                let current_arr = current.get(field_name).and_then(|v| v.as_array());
                let original_arr = original.get(field_name).and_then(|v| v.as_array());

                match (current_arr, original_arr) {
                    (Some(curr), Some(orig)) => match (curr.get(index), orig.get(index)) {
                        (Some(a), Some(b)) => !Self::values_equal(a, b),
                        (None, None) => false,
                        _ => true,
                    },
                    (Some(_), None) => true, // Entire array is new
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// Check if a table entry is new or modified (for spawners with named keys)
    ///
    /// Returns true if:
    /// - The key is new (didn't exist in original)
    /// - The value at that key was modified
    #[allow(dead_code)] // May be useful for future simple table comparisons
    pub fn is_table_key_modified(&self, field_name: &str, key: &str) -> bool {
        match (&self.current_mob, &self.original_mob) {
            (Some(current), Some(original)) => {
                let current_table = current.get(field_name).and_then(|v| v.as_table());
                let original_table = original.get(field_name).and_then(|v| v.as_table());

                match (current_table, original_table) {
                    (Some(curr), Some(orig)) => match (curr.get(key), orig.get(key)) {
                        (Some(a), Some(b)) => !Self::values_equal(a, b),
                        (None, None) => false,
                        _ => true,
                    },
                    (Some(_), None) => true, // Entire table is new
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// Check if a nested table entry is new or modified
    /// (for spawners.spawners.north, etc.)
    ///
    /// path example: ["projectile_spawners", "spawners", "north"]
    pub fn is_nested_key_modified(&self, path: &[&str]) -> bool {
        match (&self.current_mob, &self.original_mob) {
            (Some(current), Some(original)) => {
                let current_val = Self::get_nested_value(current, path);
                let original_val = Self::get_nested_value(original, path);
                match (current_val, original_val) {
                    (Some(a), Some(b)) => !Self::values_equal(a, b),
                    (None, None) => false,
                    _ => true,
                }
            }
            _ => false,
        }
    }

    /// Helper to get a nested value by path
    fn get_nested_value<'a>(value: &'a toml::Value, path: &[&str]) -> Option<&'a toml::Value> {
        let mut current = value;
        for key in path {
            current = current.get(*key)?;
        }
        Some(current)
    }

    /// Update merged_for_preview after changes to current_mob (for patches)
    ///
    /// For .mobpatch files, this re-merges base_mob + current_mob so that
    /// the preview reflects the latest changes. Should be called whenever
    /// current_mob is modified for patch files.
    ///
    /// Also sets `preview_needs_rebuild` to signal the preview system.
    pub fn update_merged_for_preview(&mut self) {
        if self.file_type == FileType::MobPatch {
            if let (Some(base), Some(patch)) = (&self.base_mob, &self.current_mob) {
                let mut merged = base.clone();
                crate::file::merge_toml_values(&mut merged, patch.clone());
                self.merged_for_preview = Some(merged);
            }
        }
        // Signal preview system to rebuild
        self.preview_needs_rebuild = true;
    }

    /// Log a success message
    pub fn log_success(&mut self, message: impl Into<String>, time: &Time) {
        self.log
            .push(message, StatusLevel::Success, time.elapsed_secs_f64());
    }

    /// Log a warning message
    pub fn log_warning(&mut self, message: impl Into<String>, time: &Time) {
        self.log
            .push(message, StatusLevel::Warning, time.elapsed_secs_f64());
    }

    /// Log an error message
    pub fn log_error(&mut self, message: impl Into<String>, time: &Time) {
        self.log
            .push(message, StatusLevel::Error, time.elapsed_secs_f64());
    }

    /// Check if the current file is from the extended assets directory
    pub fn is_extended_mob(&self, config: &EditorConfig) -> bool {
        self.current_path
            .as_ref()
            .map(|p| config.is_extended_path(p))
            .unwrap_or(false)
    }

    /// Check if extended sprites should be available for the current file
    /// Extended sprites are available for:
    /// - Extended mobs (in the extended assets directory)
    /// - All mobpatches (they can override with extended sprites)
    pub fn can_use_extended_sprites(&self, config: &EditorConfig) -> bool {
        self.file_type == FileType::MobPatch || self.is_extended_mob(config)
    }

    /// Create a new empty mob with defaults
    pub fn new_mob(name: &str) -> toml::Value {
        let mut table = toml::value::Table::new();
        table.insert("name".to_string(), toml::Value::String(name.to_string()));
        // No default sprite - user must register one explicitly
        // This avoids creating references to non-existent files
        table.insert("spawnable".to_string(), toml::Value::Boolean(true));
        table.insert("health".to_string(), toml::Value::Integer(50));

        // Default collider
        let mut collider = toml::value::Table::new();
        let mut shape = toml::value::Table::new();
        shape.insert(
            "Rectangle".to_string(),
            toml::Value::Array(vec![toml::Value::Float(10.0), toml::Value::Float(10.0)]),
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
        self.merged_for_preview
            .as_ref()
            .or(self.current_mob.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a valid mob TOML value
    fn valid_mob() -> toml::Value {
        toml::from_str(
            r#"
            name = "Test Mob"
            sprite = "media/aseprite/test.aseprite"
            spawnable = true
            health = 50

            [[colliders]]
            shape = { Rectangle = [10.0, 10.0] }
            position = [0.0, 0.0]
            rotation = 0.0
            "#,
        )
        .unwrap()
    }

    // StatusLog tests
    #[test]
    fn status_log_push_and_iter() {
        let mut log = StatusLog::default();
        log.push("Test message", StatusLevel::Success, 1.0);
        log.push("Warning", StatusLevel::Warning, 2.0);

        assert_eq!(log.entries.len(), 2);
        assert!(!log.is_empty());

        let entries: Vec<_> = log.iter().collect();
        assert_eq!(entries[0].text, "Test message");
        assert_eq!(entries[1].text, "Warning");
    }

    #[test]
    fn status_log_max_entries() {
        let mut log = StatusLog {
            entries: VecDeque::new(),
            max_entries: 3,
            expanded: false,
        };

        log.push("One", StatusLevel::Success, 1.0);
        log.push("Two", StatusLevel::Success, 2.0);
        log.push("Three", StatusLevel::Success, 3.0);
        log.push("Four", StatusLevel::Success, 4.0);

        assert_eq!(log.entries.len(), 3);
        assert_eq!(log.iter().next().unwrap().text, "Two"); // "One" was removed
    }

    #[test]
    fn status_log_last() {
        let mut log = StatusLog::default();
        assert!(log.last().is_none());

        log.push("First", StatusLevel::Success, 1.0);
        log.push("Last", StatusLevel::Error, 2.0);

        assert_eq!(log.last().unwrap().text, "Last");
    }

    // EditorSession tests
    #[test]
    fn editor_session_check_modified() {
        let mut session = EditorSession::default();

        // No mobs loaded - not modified
        session.check_modified();
        assert!(!session.is_modified);

        // Load a mob
        session.current_mob = Some(valid_mob());
        session.original_mob = Some(valid_mob());
        session.check_modified();
        assert!(!session.is_modified);

        // Modify the mob
        if let Some(mob) = &mut session.current_mob {
            mob.as_table_mut()
                .unwrap()
                .insert("health".to_string(), toml::Value::Integer(999));
        }
        session.check_modified();
        assert!(session.is_modified);
    }

    #[test]
    fn editor_session_new_mob() {
        let mob = EditorSession::new_mob("Test Enemy");

        assert_eq!(mob.get("name").and_then(|v| v.as_str()), Some("Test Enemy"));
        // Sprite is optional - new mobs don't have one by default
        assert!(mob.get("sprite").is_none());
        assert!(mob.get("health").is_some());
        assert!(mob.get("colliders").is_some());
        assert_eq!(mob.get("spawnable").and_then(|v| v.as_bool()), Some(true));
    }

    // values_equal tests for type coercion
    #[test]
    fn values_equal_integer_vs_float() {
        // Integer(50) should equal Float(50.0)
        let int_val = toml::Value::Integer(50);
        let float_val = toml::Value::Float(50.0);
        assert!(EditorSession::values_equal(&int_val, &float_val));
        assert!(EditorSession::values_equal(&float_val, &int_val));

        // Different values should not be equal
        let int_other = toml::Value::Integer(51);
        assert!(!EditorSession::values_equal(&int_other, &float_val));
    }

    #[test]
    fn values_equal_arrays_with_type_coercion() {
        // [50, 50] should equal [50.0, 50.0]
        let int_arr = toml::Value::Array(vec![
            toml::Value::Integer(50),
            toml::Value::Integer(50),
        ]);
        let float_arr = toml::Value::Array(vec![
            toml::Value::Float(50.0),
            toml::Value::Float(50.0),
        ]);
        assert!(EditorSession::values_equal(&int_arr, &float_arr));
    }

    #[test]
    fn check_modified_with_type_coercion() {
        let mut session = EditorSession::default();

        // Create original with Integer value
        let original: toml::Value = toml::from_str(
            r#"
            name = "Test"
            z_level = 0
            "#,
        )
        .unwrap();

        // Create current with Float value (same numeric value)
        let current: toml::Value = toml::from_str(
            r#"
            name = "Test"
            z_level = 0.0
            "#,
        )
        .unwrap();

        session.original_mob = Some(original);
        session.current_mob = Some(current);
        session.check_modified();

        // Should NOT be modified because 0 == 0.0
        assert!(!session.is_modified);
        assert!(!session.is_field_modified("z_level"));
    }
}
