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

/// A validation error for a specific field
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field_path: String,
    pub message: String,
}

/// Result of validation
#[derive(Default)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
}

impl ValidationResult {
    pub fn add_error(&mut self, field_path: impl Into<String>, message: impl Into<String>) {
        self.errors.push(ValidationError {
            field_path: field_path.into(),
            message: message.into(),
        });
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

/// Validate a mob value before saving
pub fn validate_mob(mob: &toml::Value, is_patch: bool) -> ValidationResult {
    let mut result = ValidationResult::default();

    // Skip most validation for patches - they override only some fields
    if is_patch {
        return result;
    }

    let table = match mob.as_table() {
        Some(t) => t,
        None => {
            result.add_error("root", "Mob must be a TOML table");
            return result;
        }
    };

    // Check sprite path (required for non-patches)
    match table.get("sprite") {
        Some(toml::Value::String(s)) if s.is_empty() => {
            result.add_error("sprite", "Sprite path cannot be empty");
        }
        None => {
            result.add_error("sprite", "Sprite path is required");
        }
        _ => {}
    }

    // Check health (must be positive if spawnable)
    let is_spawnable = table
        .get("spawnable")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if is_spawnable {
        match table.get("health") {
            Some(toml::Value::Integer(h)) if *h <= 0 => {
                result.add_error("health", "Health must be positive for spawnable mobs");
            }
            None => {
                result.add_error("health", "Health is required for spawnable mobs");
            }
            _ => {}
        }
    }

    // Check colliders (dimensions must be positive)
    if let Some(colliders) = table.get("colliders").and_then(|v| v.as_array()) {
        for (i, collider) in colliders.iter().enumerate() {
            if let Some(table) = collider.as_table()
                && let Some(shape) = table.get("shape").and_then(|v| v.as_table())
            {
                // Check Rectangle dimensions
                if let Some(dims) = shape.get("Rectangle").and_then(|v| v.as_array()) {
                    for (j, dim) in dims.iter().enumerate() {
                        if let Some(v) = dim.as_float() {
                            if v <= 0.0 {
                                result.add_error(
                                    format!("colliders[{}].shape.Rectangle[{}]", i, j),
                                    "Collider dimension must be positive",
                                );
                            }
                        } else if let Some(v) = dim.as_integer()
                            && v <= 0
                        {
                            result.add_error(
                                format!("colliders[{}].shape.Rectangle[{}]", i, j),
                                "Collider dimension must be positive",
                            );
                        }
                    }
                }

                // Check Ball radius
                if let Some(radius) = shape.get("Ball") {
                    if let Some(r) = radius.as_float() {
                        if r <= 0.0 {
                            result.add_error(
                                format!("colliders[{}].shape.Ball", i),
                                "Ball radius must be positive",
                            );
                        }
                    } else if let Some(r) = radius.as_integer()
                        && r <= 0
                    {
                        result.add_error(
                            format!("colliders[{}].shape.Ball", i),
                            "Ball radius must be positive",
                        );
                    }
                }
            }
        }
    }

    result
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

    /// Selected collider index for editing
    pub selected_collider: Option<usize>,

    /// Selected jointed mob index for editing
    pub selected_jointed_mob: Option<usize>,

    /// Selected behavior node path for editing
    pub selected_behavior_node: Option<Vec<usize>>,

    /// Status log for messages
    pub log: StatusLog,
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
            selected_collider: None,
            selected_jointed_mob: None,
            selected_behavior_node: None,
            log: StatusLog::default(),
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
        // Default sprite path - user should update this
        table.insert(
            "sprite".to_string(),
            toml::Value::String(format!(
                "media/aseprite/{}_mob.aseprite",
                name.to_lowercase().replace(' ', "_")
            )),
        );
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

    #[test]
    fn validate_mob_valid() {
        let mob = valid_mob();
        let result = validate_mob(&mob, false);
        assert!(
            !result.has_errors(),
            "Valid mob should pass: {:?}",
            result.errors
        );
    }

    #[test]
    fn validate_mob_missing_sprite() {
        let mob = toml::from_str(
            r#"
            name = "No Sprite Mob"
            spawnable = false
            "#,
        )
        .unwrap();

        let result = validate_mob(&mob, false);
        assert!(result.has_errors());
        assert!(result.errors.iter().any(|e| e.field_path == "sprite"));
    }

    #[test]
    fn validate_mob_empty_sprite() {
        let mob = toml::from_str(
            r#"
            name = "Empty Sprite Mob"
            sprite = ""
            spawnable = false
            "#,
        )
        .unwrap();

        let result = validate_mob(&mob, false);
        assert!(result.has_errors());
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.field_path == "sprite" && e.message.contains("empty"))
        );
    }

    #[test]
    fn validate_mob_missing_health_spawnable() {
        let mob = toml::from_str(
            r#"
            name = "No Health Mob"
            sprite = "media/aseprite/test.aseprite"
            spawnable = true
            "#,
        )
        .unwrap();

        let result = validate_mob(&mob, false);
        assert!(result.has_errors());
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.field_path == "health" && e.message.contains("required"))
        );
    }

    #[test]
    fn validate_mob_negative_health() {
        let mob = toml::from_str(
            r#"
            name = "Negative Health Mob"
            sprite = "media/aseprite/test.aseprite"
            spawnable = true
            health = -10
            "#,
        )
        .unwrap();

        let result = validate_mob(&mob, false);
        assert!(result.has_errors());
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.field_path == "health" && e.message.contains("positive"))
        );
    }

    #[test]
    fn validate_mob_zero_health() {
        let mob = toml::from_str(
            r#"
            name = "Zero Health Mob"
            sprite = "media/aseprite/test.aseprite"
            spawnable = true
            health = 0
            "#,
        )
        .unwrap();

        let result = validate_mob(&mob, false);
        assert!(result.has_errors());
        assert!(result.errors.iter().any(|e| e.field_path == "health"));
    }

    #[test]
    fn validate_mob_invalid_rectangle_negative() {
        let mob = toml::from_str(
            r#"
            name = "Bad Collider Mob"
            sprite = "media/aseprite/test.aseprite"
            spawnable = false

            [[colliders]]
            shape = { Rectangle = [-5.0, 10.0] }
            position = [0.0, 0.0]
            rotation = 0.0
            "#,
        )
        .unwrap();

        let result = validate_mob(&mob, false);
        assert!(result.has_errors());
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.field_path.contains("Rectangle") && e.message.contains("positive"))
        );
    }

    #[test]
    fn validate_mob_invalid_ball_negative() {
        let mob = toml::from_str(
            r#"
            name = "Bad Ball Mob"
            sprite = "media/aseprite/test.aseprite"
            spawnable = false

            [[colliders]]
            shape = { Ball = -5.0 }
            position = [0.0, 0.0]
            rotation = 0.0
            "#,
        )
        .unwrap();

        let result = validate_mob(&mob, false);
        assert!(result.has_errors());
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.field_path.contains("Ball") && e.message.contains("positive"))
        );
    }

    #[test]
    fn validate_mob_patch_skips_validation() {
        // Patches with missing required fields should still pass
        let patch = toml::from_str(
            r#"
            name = "Patch Only"
            health = 200
            "#,
        )
        .unwrap();

        let result = validate_mob(&patch, true);
        assert!(!result.has_errors(), "Patches should skip validation");
    }

    #[test]
    fn validate_mob_multiple_errors() {
        let mob = toml::from_str(
            r#"
            name = "Multiple Errors"
            spawnable = true

            [[colliders]]
            shape = { Rectangle = [-5.0, -10.0] }
            position = [0.0, 0.0]
            rotation = 0.0
            "#,
        )
        .unwrap();

        let result = validate_mob(&mob, false);
        assert!(result.has_errors());
        // Should have errors for: missing sprite, missing health, two negative dimensions
        assert!(
            result.errors.len() >= 3,
            "Expected at least 3 errors, got {}",
            result.errors.len()
        );
    }

    #[test]
    fn validate_mob_non_spawnable_no_health_ok() {
        // Non-spawnable mobs don't require health
        let mob = toml::from_str(
            r#"
            name = "Non-spawnable Mob"
            sprite = "media/aseprite/test.aseprite"
            spawnable = false

            [[colliders]]
            shape = { Rectangle = [10.0, 10.0] }
            position = [0.0, 0.0]
            rotation = 0.0
            "#,
        )
        .unwrap();

        let result = validate_mob(&mob, false);
        assert!(
            !result.has_errors(),
            "Non-spawnable mob without health should be valid"
        );
    }

    #[test]
    fn validate_mob_mixed_colliders() {
        let mob = toml::from_str(
            r#"
            name = "Mixed Colliders"
            sprite = "media/aseprite/test.aseprite"
            spawnable = false

            [[colliders]]
            shape = { Rectangle = [10.0, 10.0] }
            position = [0.0, 0.0]
            rotation = 0.0

            [[colliders]]
            shape = { Ball = 5.0 }
            position = [5.0, 0.0]
            rotation = 0.0
            "#,
        )
        .unwrap();

        let result = validate_mob(&mob, false);
        assert!(!result.has_errors(), "Mixed valid colliders should pass");
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
        assert!(mob.get("sprite").is_some());
        assert!(mob.get("health").is_some());
        assert!(mob.get("colliders").is_some());
        assert_eq!(mob.get("spawnable").and_then(|v| v.as_bool()), Some(true));
    }

    // Additional edge case tests

    #[test]
    fn validate_mob_zero_dimensions() {
        let mob = toml::from_str(
            r#"
            name = "Zero Dimensions"
            sprite = "media/aseprite/test.aseprite"
            spawnable = false

            [[colliders]]
            shape = { Rectangle = [0.0, 10.0] }
            position = [0.0, 0.0]
            rotation = 0.0
            "#,
        )
        .unwrap();

        let result = validate_mob(&mob, false);
        assert!(result.has_errors());
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.field_path.contains("Rectangle"))
        );
    }

    #[test]
    fn validate_mob_integer_collider_dimensions() {
        // Test that integer dimensions are handled correctly
        let mob = toml::from_str(
            r#"
            name = "Integer Dimensions"
            sprite = "media/aseprite/test.aseprite"
            spawnable = false

            [[colliders]]
            shape = { Rectangle = [10, 15] }
            position = [0.0, 0.0]
            rotation = 0.0
            "#,
        )
        .unwrap();

        let result = validate_mob(&mob, false);
        assert!(
            !result.has_errors(),
            "Positive integer dimensions should be valid"
        );
    }

    #[test]
    fn validate_mob_negative_integer_collider() {
        let mob = toml::from_str(
            r#"
            name = "Negative Integer"
            sprite = "media/aseprite/test.aseprite"
            spawnable = false

            [[colliders]]
            shape = { Rectangle = [-5, 10] }
            position = [0.0, 0.0]
            rotation = 0.0
            "#,
        )
        .unwrap();

        let result = validate_mob(&mob, false);
        assert!(result.has_errors());
    }

    #[test]
    fn validate_mob_ball_zero_radius() {
        let mob = toml::from_str(
            r#"
            name = "Zero Ball"
            sprite = "media/aseprite/test.aseprite"
            spawnable = false

            [[colliders]]
            shape = { Ball = 0.0 }
            position = [0.0, 0.0]
            rotation = 0.0
            "#,
        )
        .unwrap();

        let result = validate_mob(&mob, false);
        assert!(result.has_errors());
        assert!(result.errors.iter().any(|e| e.field_path.contains("Ball")));
    }

    #[test]
    fn validate_mob_integer_ball_radius() {
        let mob = toml::from_str(
            r#"
            name = "Integer Ball"
            sprite = "media/aseprite/test.aseprite"
            spawnable = false

            [[colliders]]
            shape = { Ball = 5 }
            position = [0.0, 0.0]
            rotation = 0.0
            "#,
        )
        .unwrap();

        let result = validate_mob(&mob, false);
        assert!(
            !result.has_errors(),
            "Positive integer ball radius should be valid"
        );
    }

    #[test]
    fn validate_mob_empty_colliders_array() {
        let mob = toml::from_str(
            r#"
            name = "Empty Colliders"
            sprite = "media/aseprite/test.aseprite"
            spawnable = false
            colliders = []
            "#,
        )
        .unwrap();

        let result = validate_mob(&mob, false);
        // Empty colliders array is allowed - validation doesn't require colliders
        assert!(!result.has_errors());
    }

    #[test]
    fn validate_mob_no_colliders() {
        let mob = toml::from_str(
            r#"
            name = "No Colliders"
            sprite = "media/aseprite/test.aseprite"
            spawnable = false
            "#,
        )
        .unwrap();

        let result = validate_mob(&mob, false);
        assert!(
            !result.has_errors(),
            "Mobs without colliders should be valid"
        );
    }

    #[test]
    fn validation_result_add_multiple_errors() {
        let mut result = ValidationResult::default();
        assert!(!result.has_errors());

        result.add_error("field1", "Error 1");
        result.add_error("field2", "Error 2");
        result.add_error("field3", "Error 3");

        assert!(result.has_errors());
        assert_eq!(result.errors.len(), 3);
    }
}
