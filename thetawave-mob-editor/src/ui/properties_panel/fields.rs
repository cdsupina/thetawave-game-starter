//! Field rendering helpers for the properties panel.
//!
//! This module contains reusable field rendering functions with patch-awareness,
//! including string, boolean, integer, float, and Vec2 fields.

use bevy_egui::egui;

/// Color for patched/overridden values in patch files.
pub const PATCHED_COLOR: egui::Color32 = egui::Color32::from_rgb(100, 200, 255);

/// Color for inherited values (from base mob) in patch files.
pub const INHERITED_COLOR: egui::Color32 = egui::Color32::from_rgb(140, 140, 140);

/// Result of editing a field - used to communicate changes back to the caller.
pub enum FieldResult<T> {
    /// No change was made to the field.
    NoChange,
    /// The field value was changed to the given value.
    Changed(T),
    /// The field should be removed from the patch (reset to inherited value).
    Reset,
}

/// Render a patch indicator (● for patched, ○ for inherited).
///
/// # Arguments
/// * `ui` - The egui UI context
/// * `is_patched` - Whether this field is overridden in the patch
/// * `show_indicator` - Whether to show the indicator (usually true for patch files)
///
/// # Returns
/// Always returns false (indicator doesn't have click behavior).
pub fn render_patch_indicator(ui: &mut egui::Ui, is_patched: bool, show_indicator: bool) -> bool {
    if show_indicator {
        if is_patched {
            ui.label(egui::RichText::new("●").color(PATCHED_COLOR));
        } else {
            ui.label(egui::RichText::new("○").color(INHERITED_COLOR));
        }
    }
    false
}

/// Render a reset button for patched fields.
///
/// # Arguments
/// * `ui` - The egui UI context
/// * `is_patched` - Whether this field is overridden in the patch
/// * `is_patch_file` - Whether we're editing a patch file
///
/// # Returns
/// True if the reset button was clicked.
pub fn render_reset_button(ui: &mut egui::Ui, is_patched: bool, is_patch_file: bool) -> bool {
    if is_patch_file && is_patched {
        let response = ui.add(
            egui::Button::new(egui::RichText::new("×").color(egui::Color32::WHITE))
                .fill(egui::Color32::from_rgb(120, 60, 60))
                .min_size(egui::vec2(16.0, 16.0)),
        );
        if response
            .on_hover_text("Remove from patch (use base value)")
            .clicked()
        {
            return true;
        }
    }
    false
}

/// Get label color based on patch state.
///
/// Returns dimmed color for inherited values in patch files.
pub fn label_color(ui: &egui::Ui, is_patch_file: bool, is_patched: bool) -> egui::Color32 {
    if is_patch_file && !is_patched {
        INHERITED_COLOR
    } else {
        ui.style().visuals.text_color()
    }
}

/// Render a string field with patch awareness.
///
/// # Arguments
/// * `ui` - The egui UI context
/// * `label` - The field label to display
/// * `current_value` - The current string value
/// * `is_patched` - Whether this field is overridden in the patch
/// * `is_patch_file` - Whether we're editing a patch file
///
/// # Returns
/// `FieldResult::Changed` with new value, `FieldResult::Reset` if reset clicked, or `FieldResult::NoChange`.
pub fn render_string_field(
    ui: &mut egui::Ui,
    label: &str,
    current_value: &str,
    is_patched: bool,
    is_patch_file: bool,
) -> FieldResult<String> {
    let mut result = FieldResult::NoChange;
    ui.horizontal(|ui| {
        render_patch_indicator(ui, is_patched, is_patch_file);
        ui.label(egui::RichText::new(label).color(label_color(ui, is_patch_file, is_patched)));

        let mut value = current_value.to_string();
        if ui.text_edit_singleline(&mut value).changed() {
            result = FieldResult::Changed(value);
        }

        if render_reset_button(ui, is_patched, is_patch_file) {
            result = FieldResult::Reset;
        }
    });
    result
}

/// Render a boolean field with patch awareness.
///
/// # Arguments
/// * `ui` - The egui UI context
/// * `label` - The field label to display
/// * `current_value` - The current boolean value
/// * `is_patched` - Whether this field is overridden in the patch
/// * `is_patch_file` - Whether we're editing a patch file
///
/// # Returns
/// `FieldResult::Changed` with new value, `FieldResult::Reset` if reset clicked, or `FieldResult::NoChange`.
pub fn render_bool_field(
    ui: &mut egui::Ui,
    label: &str,
    current_value: bool,
    is_patched: bool,
    is_patch_file: bool,
) -> FieldResult<bool> {
    let mut result = FieldResult::NoChange;
    ui.horizontal(|ui| {
        render_patch_indicator(ui, is_patched, is_patch_file);
        ui.label(egui::RichText::new(label).color(label_color(ui, is_patch_file, is_patched)));

        let mut value = current_value;
        if ui.checkbox(&mut value, "").changed() {
            result = FieldResult::Changed(value);
        }

        if render_reset_button(ui, is_patched, is_patch_file) {
            result = FieldResult::Reset;
        }
    });
    result
}

/// Render an integer field with patch awareness.
///
/// # Arguments
/// * `ui` - The egui UI context
/// * `label` - The field label to display
/// * `current_value` - The current integer value
/// * `range` - The valid range for the value
/// * `is_patched` - Whether this field is overridden in the patch
/// * `is_patch_file` - Whether we're editing a patch file
///
/// # Returns
/// `FieldResult::Changed` with new value, `FieldResult::Reset` if reset clicked, or `FieldResult::NoChange`.
pub fn render_int_field(
    ui: &mut egui::Ui,
    label: &str,
    current_value: i32,
    range: std::ops::RangeInclusive<i32>,
    is_patched: bool,
    is_patch_file: bool,
) -> FieldResult<i32> {
    let mut result = FieldResult::NoChange;
    ui.horizontal(|ui| {
        render_patch_indicator(ui, is_patched, is_patch_file);
        ui.label(egui::RichText::new(label).color(label_color(ui, is_patch_file, is_patched)));

        let mut value = current_value;
        if ui
            .add(egui::DragValue::new(&mut value).range(range))
            .changed()
        {
            result = FieldResult::Changed(value);
        }

        if render_reset_button(ui, is_patched, is_patch_file) {
            result = FieldResult::Reset;
        }
    });
    result
}

/// Render a float field with patch awareness.
///
/// # Arguments
/// * `ui` - The egui UI context
/// * `label` - The field label to display
/// * `current_value` - The current float value
/// * `range` - The valid range for the value
/// * `speed` - Optional drag speed (default if None)
/// * `is_patched` - Whether this field is overridden in the patch
/// * `is_patch_file` - Whether we're editing a patch file
///
/// # Returns
/// `FieldResult::Changed` with new value, `FieldResult::Reset` if reset clicked, or `FieldResult::NoChange`.
pub fn render_float_field(
    ui: &mut egui::Ui,
    label: &str,
    current_value: f32,
    range: std::ops::RangeInclusive<f32>,
    speed: Option<f64>,
    is_patched: bool,
    is_patch_file: bool,
) -> FieldResult<f32> {
    let mut result = FieldResult::NoChange;
    ui.horizontal(|ui| {
        render_patch_indicator(ui, is_patched, is_patch_file);
        ui.label(egui::RichText::new(label).color(label_color(ui, is_patch_file, is_patched)));

        let mut value = current_value;
        let mut drag = egui::DragValue::new(&mut value).range(range);
        if let Some(s) = speed {
            drag = drag.speed(s);
        }
        if ui.add(drag).changed() {
            result = FieldResult::Changed(value);
        }

        if render_reset_button(ui, is_patched, is_patch_file) {
            result = FieldResult::Reset;
        }
    });
    result
}

/// Render a Vec2 field with patch awareness.
///
/// # Arguments
/// * `ui` - The egui UI context
/// * `label` - The field label to display
/// * `x` - The current X value
/// * `y` - The current Y value
/// * `range` - The valid range for both values
/// * `speed` - Optional drag speed (default if None)
/// * `is_patched` - Whether this field is overridden in the patch
/// * `is_patch_file` - Whether we're editing a patch file
///
/// # Returns
/// `FieldResult::Changed` with new (x, y) tuple, `FieldResult::Reset` if reset clicked, or `FieldResult::NoChange`.
pub fn render_vec2_field(
    ui: &mut egui::Ui,
    label: &str,
    x: f32,
    y: f32,
    range: std::ops::RangeInclusive<f32>,
    speed: Option<f64>,
    is_patched: bool,
    is_patch_file: bool,
) -> FieldResult<(f32, f32)> {
    let mut result = FieldResult::NoChange;

    ui.horizontal(|ui| {
        render_patch_indicator(ui, is_patched, is_patch_file);
        ui.label(egui::RichText::new(label).color(label_color(ui, is_patch_file, is_patched)));

        if render_reset_button(ui, is_patched, is_patch_file) {
            result = FieldResult::Reset;
        }
    });

    if !matches!(result, FieldResult::Reset) {
        ui.horizontal(|ui| {
            ui.add_space(16.0);
            ui.label("X:");
            let mut new_x = x;
            let mut drag_x = egui::DragValue::new(&mut new_x).range(range.clone());
            if let Some(s) = speed {
                drag_x = drag_x.speed(s);
            }
            let x_changed = ui.add(drag_x).changed();

            ui.label("Y:");
            let mut new_y = y;
            let mut drag_y = egui::DragValue::new(&mut new_y).range(range);
            if let Some(s) = speed {
                drag_y = drag_y.speed(s);
            }
            let y_changed = ui.add(drag_y).changed();

            if x_changed || y_changed {
                result = FieldResult::Changed((new_x, new_y));
            }
        });
    }

    result
}

/// Helper to get a Vec2 value from a TOML table.
///
/// # Arguments
/// * `table` - The TOML table to read from
/// * `key` - The key to look up
/// * `default_x` - Default X value if not found
/// * `default_y` - Default Y value if not found
///
/// # Returns
/// A tuple of (x, y) values.
pub fn get_vec2_value(
    table: &toml::value::Table,
    key: &str,
    default_x: f32,
    default_y: f32,
) -> (f32, f32) {
    if let Some(arr) = table.get(key).and_then(|v| v.as_array()) {
        let x = arr
            .first()
            .and_then(|v| v.as_float())
            .unwrap_or(default_x as f64) as f32;
        let y = arr
            .get(1)
            .and_then(|v| v.as_float())
            .unwrap_or(default_y as f64) as f32;
        (x, y)
    } else {
        (default_x, default_y)
    }
}

/// Helper to set a Vec2 value in a TOML table.
///
/// # Arguments
/// * `table` - The TOML table to modify
/// * `key` - The key to set
/// * `x` - The X value
/// * `y` - The Y value
pub fn set_vec2_value(table: &mut toml::value::Table, key: &str, x: f32, y: f32) {
    table.insert(
        key.to_string(),
        toml::Value::Array(vec![
            toml::Value::Float(x as f64),
            toml::Value::Float(y as f64),
        ]),
    );
}
