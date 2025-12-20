//! Validation framework for mob files

use std::fmt;
use toml::Value;

/// Represents a validation error with location and severity
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
    pub severity: Severity,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let severity = match self.severity {
            Severity::Error => "ERROR",
            Severity::Warning => "WARN",
        };
        if self.path.is_empty() {
            write!(f, "[{}] {}", severity, self.message)
        } else {
            write!(f, "[{}] {}: {}", severity, self.path, self.message)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

/// Result of validation
#[derive(Debug, Clone, Default)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_error(&mut self, path: impl Into<String>, message: impl Into<String>) {
        self.errors.push(ValidationError {
            path: path.into(),
            message: message.into(),
            severity: Severity::Error,
        });
    }

    pub fn add_warning(&mut self, path: impl Into<String>, message: impl Into<String>) {
        self.errors.push(ValidationError {
            path: path.into(),
            message: message.into(),
            severity: Severity::Warning,
        });
    }

    pub fn has_errors(&self) -> bool {
        self.errors.iter().any(|e| e.severity == Severity::Error)
    }

    pub fn has_warnings(&self) -> bool {
        self.errors.iter().any(|e| e.severity == Severity::Warning)
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn merge(&mut self, other: ValidationResult) {
        self.errors.extend(other.errors);
    }

    /// Get all error messages as a formatted string
    pub fn format_errors(&self) -> String {
        self.errors
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Get only errors (not warnings) as strings
    pub fn error_messages(&self) -> Vec<String> {
        self.errors
            .iter()
            .filter(|e| e.severity == Severity::Error)
            .map(|e| e.to_string())
            .collect()
    }
}

/// Validates a mob TOML value
pub fn validate_mob(value: &Value, is_patch: bool) -> ValidationResult {
    let mut result = ValidationResult::new();

    let Some(table) = value.as_table() else {
        result.add_error("", "Root must be a TOML table");
        return result;
    };

    // Validate name field
    if !is_patch {
        validate_required_string(&mut result, table, "name", "");
    } else if let Some(name) = table.get("name") {
        validate_string_value(&mut result, name, "name");
    }

    // Validate numeric fields with ranges
    if let Some(health) = table.get("health") {
        validate_positive_integer(&mut result, health, "health");
    }

    // Movement parameters
    validate_optional_vec2(&mut result, table, "max_linear_speed");
    validate_optional_vec2(&mut result, table, "linear_acceleration");
    validate_optional_vec2(&mut result, table, "linear_deceleration");
    validate_optional_positive_f32(&mut result, table, "max_angular_speed");
    validate_optional_positive_f32(&mut result, table, "angular_acceleration");
    validate_optional_positive_f32(&mut result, table, "angular_deceleration");

    // Physics parameters
    validate_optional_f32_range(&mut result, table, "restitution", 0.0, 1.0);
    validate_optional_f32_range(&mut result, table, "friction", 0.0, 10.0);
    validate_optional_positive_f32(&mut result, table, "collider_density");

    // Combat parameters
    validate_optional_positive_f32(&mut result, table, "projectile_speed");
    validate_optional_positive_integer(&mut result, table, "projectile_damage");
    validate_optional_positive_f32(&mut result, table, "range_seconds");

    // Colliders
    if let Some(colliders) = table.get("colliders") {
        validate_colliders(&mut result, colliders);
    }

    // Spawners
    if let Some(spawners) = table.get("projectile_spawners") {
        validate_projectile_spawners(&mut result, spawners);
    }

    if let Some(spawners) = table.get("mob_spawners") {
        validate_mob_spawners(&mut result, spawners);
    }

    // Behavior
    if let Some(behavior) = table.get("behavior") {
        validate_behavior(&mut result, behavior, "behavior");
    }

    // Jointed mobs
    if let Some(jointed) = table.get("jointed_mobs") {
        validate_jointed_mobs(&mut result, jointed);
    }

    // Decorations
    if let Some(decorations) = table.get("decorations") {
        validate_decorations(&mut result, decorations);
    }

    result
}

fn validate_required_string(
    result: &mut ValidationResult,
    table: &toml::map::Map<String, Value>,
    field: &str,
    path_prefix: &str,
) {
    let path = if path_prefix.is_empty() {
        field.to_string()
    } else {
        format!("{}.{}", path_prefix, field)
    };

    match table.get(field) {
        None => result.add_error(&path, "Required field is missing"),
        Some(value) => {
            if let Some(s) = value.as_str() {
                if s.is_empty() {
                    result.add_error(&path, "Cannot be empty");
                }
            } else {
                result.add_error(&path, "Must be a string");
            }
        }
    }
}

fn validate_string_value(result: &mut ValidationResult, value: &Value, path: &str) {
    if let Some(s) = value.as_str() {
        if s.is_empty() {
            result.add_error(path, "Cannot be empty");
        }
    } else {
        result.add_error(path, "Must be a string");
    }
}

fn validate_positive_integer(result: &mut ValidationResult, value: &Value, path: &str) {
    if let Some(i) = value.as_integer() {
        if i <= 0 {
            result.add_error(path, "Must be a positive integer");
        }
    } else {
        result.add_error(path, "Must be an integer");
    }
}

fn validate_optional_positive_integer(
    result: &mut ValidationResult,
    table: &toml::map::Map<String, Value>,
    field: &str,
) {
    if let Some(value) = table.get(field) {
        validate_positive_integer(result, value, field);
    }
}

fn validate_optional_positive_f32(
    result: &mut ValidationResult,
    table: &toml::map::Map<String, Value>,
    field: &str,
) {
    if let Some(value) = table.get(field) {
        if let Some(f) = value.as_float() {
            if f <= 0.0 {
                result.add_error(field, "Must be positive");
            }
        } else if let Some(i) = value.as_integer() {
            if i <= 0 {
                result.add_error(field, "Must be positive");
            }
        } else {
            result.add_error(field, "Must be a number");
        }
    }
}

fn validate_optional_f32_range(
    result: &mut ValidationResult,
    table: &toml::map::Map<String, Value>,
    field: &str,
    min: f32,
    max: f32,
) {
    if let Some(value) = table.get(field) {
        let f = if let Some(f) = value.as_float() {
            f as f32
        } else if let Some(i) = value.as_integer() {
            i as f32
        } else {
            result.add_error(field, "Must be a number");
            return;
        };

        if f < min || f > max {
            result.add_error(field, format!("Must be between {} and {}", min, max));
        }
    }
}

fn validate_optional_vec2(
    result: &mut ValidationResult,
    table: &toml::map::Map<String, Value>,
    field: &str,
) {
    if let Some(value) = table.get(field) {
        if let Some(arr) = value.as_array() {
            if arr.len() != 2 {
                result.add_error(field, "Must be an array of 2 numbers [x, y]");
                return;
            }
            for (i, v) in arr.iter().enumerate() {
                if v.as_float().is_none() && v.as_integer().is_none() {
                    result.add_error(field, format!("Element {} must be a number", i));
                }
            }
        } else {
            result.add_error(field, "Must be an array [x, y]");
        }
    }
}

fn validate_colliders(result: &mut ValidationResult, value: &Value) {
    let Some(arr) = value.as_array() else {
        result.add_error("colliders", "Must be an array");
        return;
    };

    for (i, collider) in arr.iter().enumerate() {
        let path = format!("colliders[{}]", i);
        let Some(table) = collider.as_table() else {
            result.add_error(&path, "Must be a table");
            continue;
        };

        // Shape is required
        match table.get("shape") {
            None => result.add_error(&path, "Missing required field 'shape'"),
            Some(shape) => validate_collider_shape(result, shape, &format!("{}.shape", path)),
        }

        // Position is optional but must be valid if present
        if let Some(pos) = table.get("position") {
            if let Some(arr) = pos.as_array() {
                if arr.len() != 2 {
                    result.add_error(format!("{}.position", path), "Must be [x, y]");
                }
            } else {
                result.add_error(format!("{}.position", path), "Must be an array [x, y]");
            }
        }

        // Rotation is optional but must be a number
        if let Some(rot) = table.get("rotation") {
            if rot.as_float().is_none() && rot.as_integer().is_none() {
                result.add_error(format!("{}.rotation", path), "Must be a number");
            }
        }
    }
}

fn validate_collider_shape(result: &mut ValidationResult, value: &Value, path: &str) {
    let Some(table) = value.as_table() else {
        result.add_error(path, "Shape must be a table");
        return;
    };

    // Should have exactly one key (the shape type)
    if table.len() != 1 {
        result.add_error(path, "Shape must have exactly one type");
        return;
    }

    let (shape_type, dimensions) = table.iter().next().unwrap();

    match shape_type.as_str() {
        "Rectangle" => {
            if let Some(arr) = dimensions.as_array() {
                if arr.len() != 2 {
                    result.add_error(path, "Rectangle requires [width, height]");
                } else {
                    for (i, dim) in arr.iter().enumerate() {
                        if let Some(v) = dim.as_float().or_else(|| dim.as_integer().map(|i| i as f64)) {
                            if v <= 0.0 {
                                result.add_error(
                                    path,
                                    format!("Rectangle dimension {} must be positive", i),
                                );
                            }
                        } else {
                            result.add_error(path, format!("Rectangle dimension {} must be a number", i));
                        }
                    }
                }
            } else {
                result.add_error(path, "Rectangle requires [width, height]");
            }
        }
        "Circle" => {
            if let Some(r) = dimensions.as_float().or_else(|| dimensions.as_integer().map(|i| i as f64)) {
                if r <= 0.0 {
                    result.add_error(path, "Circle radius must be positive");
                }
            } else {
                result.add_error(path, "Circle requires a radius number");
            }
        }
        "Capsule" => {
            if let Some(arr) = dimensions.as_array() {
                if arr.len() != 2 {
                    result.add_error(path, "Capsule requires [radius, half_length]");
                }
            } else {
                result.add_error(path, "Capsule requires [radius, half_length]");
            }
        }
        _ => {
            result.add_warning(path, format!("Unknown shape type '{}'", shape_type));
        }
    }
}

fn validate_projectile_spawners(result: &mut ValidationResult, value: &Value) {
    let Some(table) = value.as_table() else {
        result.add_error("projectile_spawners", "Must be a table");
        return;
    };

    if let Some(spawners) = table.get("spawners") {
        let Some(spawners_table) = spawners.as_table() else {
            result.add_error("projectile_spawners.spawners", "Must be a table");
            return;
        };

        for (key, spawner) in spawners_table {
            let path = format!("projectile_spawners.spawners.{}", key);
            validate_spawner(result, spawner, &path);
        }
    }
}

fn validate_mob_spawners(result: &mut ValidationResult, value: &Value) {
    let Some(table) = value.as_table() else {
        result.add_error("mob_spawners", "Must be a table");
        return;
    };

    if let Some(spawners) = table.get("spawners") {
        let Some(spawners_table) = spawners.as_table() else {
            result.add_error("mob_spawners.spawners", "Must be a table");
            return;
        };

        for (key, spawner) in spawners_table {
            let path = format!("mob_spawners.spawners.{}", key);

            let Some(spawner_table) = spawner.as_table() else {
                result.add_error(&path, "Must be a table");
                continue;
            };

            // Timer is required
            if let Some(timer) = spawner_table.get("timer") {
                if let Some(t) = timer.as_float().or_else(|| timer.as_integer().map(|i| i as f64)) {
                    if t <= 0.0 {
                        result.add_error(format!("{}.timer", path), "Must be positive");
                    }
                } else {
                    result.add_error(format!("{}.timer", path), "Must be a number");
                }
            } else {
                result.add_error(&path, "Missing required field 'timer'");
            }

            // mob_ref is required
            if !spawner_table.contains_key("mob_ref") {
                result.add_error(&path, "Missing required field 'mob_ref'");
            }
        }
    }
}

fn validate_spawner(result: &mut ValidationResult, value: &Value, path: &str) {
    let Some(table) = value.as_table() else {
        result.add_error(path, "Must be a table");
        return;
    };

    // Timer is required
    if let Some(timer) = table.get("timer") {
        if let Some(t) = timer.as_float().or_else(|| timer.as_integer().map(|i| i as f64)) {
            if t <= 0.0 {
                result.add_error(format!("{}.timer", path), "Must be positive");
            }
        } else {
            result.add_error(format!("{}.timer", path), "Must be a number");
        }
    } else {
        result.add_error(path, "Missing required field 'timer'");
    }

    // projectile_type is required
    if !table.contains_key("projectile_type") {
        result.add_error(path, "Missing required field 'projectile_type'");
    }

    // faction is required
    if !table.contains_key("faction") {
        result.add_error(path, "Missing required field 'faction'");
    }
}

fn validate_behavior(result: &mut ValidationResult, value: &Value, path: &str) {
    let Some(table) = value.as_table() else {
        result.add_error(path, "Must be a table");
        return;
    };

    // type is required
    match table.get("type") {
        None => result.add_error(path, "Missing required field 'type'"),
        Some(t) => {
            if let Some(type_str) = t.as_str() {
                let valid_types = [
                    "Forever", "Sequence", "Select", "Invert", "Succeed", "Fail",
                    "Condition", "Action", "Wait", "WaitUntil", "Repeat",
                ];
                if !valid_types.contains(&type_str) {
                    result.add_warning(format!("{}.type", path), format!("Unknown behavior type '{}'", type_str));
                }
            } else {
                result.add_error(format!("{}.type", path), "Must be a string");
            }
        }
    }

    // Validate children if present
    if let Some(children) = table.get("children") {
        if let Some(arr) = children.as_array() {
            for (i, child) in arr.iter().enumerate() {
                validate_behavior(result, child, &format!("{}.children[{}]", path, i));
            }
        } else {
            result.add_error(format!("{}.children", path), "Must be an array");
        }
    }
}

fn validate_jointed_mobs(result: &mut ValidationResult, value: &Value) {
    let Some(arr) = value.as_array() else {
        result.add_error("jointed_mobs", "Must be an array");
        return;
    };

    for (i, joint) in arr.iter().enumerate() {
        let path = format!("jointed_mobs[{}]", i);
        let Some(table) = joint.as_table() else {
            result.add_error(&path, "Must be a table");
            continue;
        };

        // key is required
        if !table.contains_key("key") {
            result.add_error(&path, "Missing required field 'key'");
        }

        // mob_ref is required
        if !table.contains_key("mob_ref") {
            result.add_error(&path, "Missing required field 'mob_ref'");
        }
    }
}

fn validate_decorations(result: &mut ValidationResult, value: &Value) {
    let Some(arr) = value.as_array() else {
        result.add_error("decorations", "Must be an array");
        return;
    };

    for (i, decoration) in arr.iter().enumerate() {
        let path = format!("decorations[{}]", i);

        // Each decoration should be [sprite_key, [x, y]]
        let Some(dec_arr) = decoration.as_array() else {
            result.add_error(&path, "Must be an array [sprite_key, [x, y]]");
            continue;
        };

        if dec_arr.len() != 2 {
            result.add_error(&path, "Must be [sprite_key, [x, y]]");
            continue;
        }

        // First element is sprite key
        if dec_arr[0].as_str().is_none() {
            result.add_error(&path, "First element must be a string (sprite key)");
        }

        // Second element is position
        if let Some(pos) = dec_arr[1].as_array() {
            if pos.len() != 2 {
                result.add_error(&path, "Position must be [x, y]");
            }
        } else {
            result.add_error(&path, "Second element must be position [x, y]");
        }
    }
}
