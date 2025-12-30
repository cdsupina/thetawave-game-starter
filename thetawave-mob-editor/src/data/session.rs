use std::collections::VecDeque;
use std::path::PathBuf;

use bevy::prelude::*;

use crate::plugin::EditorConfig;

// =============================================================================
// Constants
// =============================================================================

/// Maximum number of log entries to keep in the status log.
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

/// Scrolling log of status messages.
///
/// Uses VecDeque for O(1) removal from the front when at capacity.
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

    /// Get all log entries as a slice.
    ///
    /// Note: This returns a tuple of two slices since VecDeque may not be contiguous.
    /// For most UI purposes, use `iter()` instead.
    pub fn entries(&self) -> (&[LogEntry], &[LogEntry]) {
        self.entries.as_slices()
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

    /// Get the number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
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
            if let Some(table) = collider.as_table() {
                if let Some(shape) = table.get("shape").and_then(|v| v.as_table()) {
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
                            } else if let Some(v) = dim.as_integer() {
                                if v <= 0 {
                                    result.add_error(
                                        format!("colliders[{}].shape.Rectangle[{}]", i, j),
                                        "Collider dimension must be positive",
                                    );
                                }
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
                        } else if let Some(r) = radius.as_integer() {
                            if r <= 0 {
                                result.add_error(
                                    format!("colliders[{}].shape.Ball", i),
                                    "Ball radius must be positive",
                                );
                            }
                        }
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
        self.log.push(message, StatusLevel::Success, time.elapsed_secs_f64());
    }

    /// Log a warning message
    pub fn log_warning(&mut self, message: impl Into<String>, time: &Time) {
        self.log.push(message, StatusLevel::Warning, time.elapsed_secs_f64());
    }

    /// Log an error message
    pub fn log_error(&mut self, message: impl Into<String>, time: &Time) {
        self.log.push(message, StatusLevel::Error, time.elapsed_secs_f64());
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
