use bevy_egui::egui;

use crate::data::EditorSession;

/// Render the properties editing panel
pub fn properties_panel_ui(ui: &mut egui::Ui, session: &mut EditorSession) {
    ui.heading("Properties");
    ui.separator();

    let Some(mob) = &mut session.current_mob else {
        ui.label("No mob loaded");
        return;
    };

    let Some(table) = mob.as_table_mut() else {
        ui.colored_label(egui::Color32::RED, "Invalid mob data");
        return;
    };

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // General section
            egui::CollapsingHeader::new("General")
                .default_open(true)
                .show(ui, |ui| {
                    // Name
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        let name = table
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let mut name_edit = name.clone();
                        if ui.text_edit_singleline(&mut name_edit).changed() {
                            table.insert(
                                "name".to_string(),
                                toml::Value::String(name_edit),
                            );
                            session.is_modified = true;
                        }
                    });

                    // Spawnable
                    ui.horizontal(|ui| {
                        ui.label("Spawnable:");
                        let spawnable = table
                            .get("spawnable")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(true);
                        let mut spawnable_edit = spawnable;
                        if ui.checkbox(&mut spawnable_edit, "").changed() {
                            table.insert(
                                "spawnable".to_string(),
                                toml::Value::Boolean(spawnable_edit),
                            );
                            session.is_modified = true;
                        }
                    });

                    // Sprite Key
                    ui.horizontal(|ui| {
                        ui.label("Sprite Key:");
                        let sprite_key = table
                            .get("sprite_key")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let mut key_edit = sprite_key.clone();
                        if ui.text_edit_singleline(&mut key_edit).changed() {
                            if key_edit.is_empty() {
                                table.remove("sprite_key");
                            } else {
                                table.insert(
                                    "sprite_key".to_string(),
                                    toml::Value::String(key_edit),
                                );
                            }
                            session.is_modified = true;
                        }
                    });
                });

            // Combat section
            egui::CollapsingHeader::new("Combat")
                .default_open(true)
                .show(ui, |ui| {
                    // Health
                    ui.horizontal(|ui| {
                        ui.label("Health:");
                        let health = table
                            .get("health")
                            .and_then(|v| v.as_integer())
                            .unwrap_or(50) as i32;
                        let mut health_edit = health;
                        if ui
                            .add(egui::DragValue::new(&mut health_edit).range(1..=10000))
                            .changed()
                        {
                            table.insert(
                                "health".to_string(),
                                toml::Value::Integer(health_edit as i64),
                            );
                            session.is_modified = true;
                        }
                    });

                    // Projectile Speed
                    ui.horizontal(|ui| {
                        ui.label("Proj Speed:");
                        let speed = table
                            .get("projectile_speed")
                            .and_then(|v| v.as_float())
                            .unwrap_or(100.0) as f32;
                        let mut speed_edit = speed;
                        if ui
                            .add(egui::DragValue::new(&mut speed_edit).range(0.0..=1000.0))
                            .changed()
                        {
                            table.insert(
                                "projectile_speed".to_string(),
                                toml::Value::Float(speed_edit as f64),
                            );
                            session.is_modified = true;
                        }
                    });

                    // Projectile Damage
                    ui.horizontal(|ui| {
                        ui.label("Proj Damage:");
                        let damage = table
                            .get("projectile_damage")
                            .and_then(|v| v.as_integer())
                            .unwrap_or(5) as i32;
                        let mut damage_edit = damage;
                        if ui
                            .add(egui::DragValue::new(&mut damage_edit).range(0..=1000))
                            .changed()
                        {
                            table.insert(
                                "projectile_damage".to_string(),
                                toml::Value::Integer(damage_edit as i64),
                            );
                            session.is_modified = true;
                        }
                    });

                    // Targeting Range
                    ui.horizontal(|ui| {
                        ui.label("Target Range:");
                        let range = table
                            .get("targeting_range")
                            .and_then(|v| v.as_float())
                            .map(|f| f as f32);
                        let mut has_range = range.is_some();
                        let mut range_edit = range.unwrap_or(100.0);

                        if ui.checkbox(&mut has_range, "").changed() {
                            if has_range {
                                table.insert(
                                    "targeting_range".to_string(),
                                    toml::Value::Float(range_edit as f64),
                                );
                            } else {
                                table.remove("targeting_range");
                            }
                            session.is_modified = true;
                        }

                        if has_range {
                            if ui
                                .add(egui::DragValue::new(&mut range_edit).range(0.0..=1000.0))
                                .changed()
                            {
                                table.insert(
                                    "targeting_range".to_string(),
                                    toml::Value::Float(range_edit as f64),
                                );
                                session.is_modified = true;
                            }
                        }
                    });
                });

            // Movement section
            egui::CollapsingHeader::new("Movement")
                .default_open(false)
                .show(ui, |ui| {
                    // Max Linear Speed
                    ui.label("Max Linear Speed:");
                    let max_speed = get_vec2_value(table, "max_linear_speed", 20.0, 20.0);
                    let mut speed_x = max_speed.0;
                    let mut speed_y = max_speed.1;
                    ui.horizontal(|ui| {
                        ui.label("X:");
                        let x_changed = ui
                            .add(egui::DragValue::new(&mut speed_x).range(0.0..=500.0))
                            .changed();
                        ui.label("Y:");
                        let y_changed = ui
                            .add(egui::DragValue::new(&mut speed_y).range(0.0..=500.0))
                            .changed();
                        if x_changed || y_changed {
                            set_vec2_value(table, "max_linear_speed", speed_x, speed_y);
                            session.is_modified = true;
                        }
                    });

                    // Linear Acceleration
                    ui.label("Linear Acceleration:");
                    let accel = get_vec2_value(table, "linear_acceleration", 0.1, 0.1);
                    let mut accel_x = accel.0;
                    let mut accel_y = accel.1;
                    ui.horizontal(|ui| {
                        ui.label("X:");
                        let x_changed = ui
                            .add(egui::DragValue::new(&mut accel_x).range(0.0..=10.0).speed(0.01))
                            .changed();
                        ui.label("Y:");
                        let y_changed = ui
                            .add(egui::DragValue::new(&mut accel_y).range(0.0..=10.0).speed(0.01))
                            .changed();
                        if x_changed || y_changed {
                            set_vec2_value(table, "linear_acceleration", accel_x, accel_y);
                            session.is_modified = true;
                        }
                    });

                    // Linear Deceleration
                    ui.label("Linear Deceleration:");
                    let decel = get_vec2_value(table, "linear_deceleration", 0.3, 0.3);
                    let mut decel_x = decel.0;
                    let mut decel_y = decel.1;
                    ui.horizontal(|ui| {
                        ui.label("X:");
                        let x_changed = ui
                            .add(egui::DragValue::new(&mut decel_x).range(0.0..=10.0).speed(0.01))
                            .changed();
                        ui.label("Y:");
                        let y_changed = ui
                            .add(egui::DragValue::new(&mut decel_y).range(0.0..=10.0).speed(0.01))
                            .changed();
                        if x_changed || y_changed {
                            set_vec2_value(table, "linear_deceleration", decel_x, decel_y);
                            session.is_modified = true;
                        }
                    });

                    // Max Angular Speed
                    ui.horizontal(|ui| {
                        ui.label("Max Angular Speed:");
                        let angular = table
                            .get("max_angular_speed")
                            .and_then(|v| v.as_float())
                            .unwrap_or(1.0) as f32;
                        let mut angular_edit = angular;
                        if ui
                            .add(egui::DragValue::new(&mut angular_edit).range(0.0..=20.0).speed(0.1))
                            .changed()
                        {
                            table.insert(
                                "max_angular_speed".to_string(),
                                toml::Value::Float(angular_edit as f64),
                            );
                            session.is_modified = true;
                        }
                    });
                });

            // Physics section
            egui::CollapsingHeader::new("Physics")
                .default_open(false)
                .show(ui, |ui| {
                    // Z Level
                    ui.horizontal(|ui| {
                        ui.label("Z Level:");
                        let z = table
                            .get("z_level")
                            .and_then(|v| v.as_float())
                            .unwrap_or(0.0) as f32;
                        let mut z_edit = z;
                        if ui
                            .add(egui::DragValue::new(&mut z_edit).range(-10.0..=10.0).speed(0.1))
                            .changed()
                        {
                            table.insert(
                                "z_level".to_string(),
                                toml::Value::Float(z_edit as f64),
                            );
                            session.is_modified = true;
                        }
                    });

                    // Rotation Locked
                    ui.horizontal(|ui| {
                        ui.label("Rotation Locked:");
                        let locked = table
                            .get("rotation_locked")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(true);
                        let mut locked_edit = locked;
                        if ui.checkbox(&mut locked_edit, "").changed() {
                            table.insert(
                                "rotation_locked".to_string(),
                                toml::Value::Boolean(locked_edit),
                            );
                            session.is_modified = true;
                        }
                    });

                    // Restitution
                    ui.horizontal(|ui| {
                        ui.label("Restitution:");
                        let restitution = table
                            .get("restitution")
                            .and_then(|v| v.as_float())
                            .unwrap_or(0.5) as f32;
                        let mut r_edit = restitution;
                        if ui
                            .add(egui::DragValue::new(&mut r_edit).range(0.0..=1.0).speed(0.01))
                            .changed()
                        {
                            table.insert(
                                "restitution".to_string(),
                                toml::Value::Float(r_edit as f64),
                            );
                            session.is_modified = true;
                        }
                    });

                    // Friction
                    ui.horizontal(|ui| {
                        ui.label("Friction:");
                        let friction = table
                            .get("friction")
                            .and_then(|v| v.as_float())
                            .unwrap_or(0.5) as f32;
                        let mut f_edit = friction;
                        if ui
                            .add(egui::DragValue::new(&mut f_edit).range(0.0..=2.0).speed(0.01))
                            .changed()
                        {
                            table.insert(
                                "friction".to_string(),
                                toml::Value::Float(f_edit as f64),
                            );
                            session.is_modified = true;
                        }
                    });

                    // Density
                    ui.horizontal(|ui| {
                        ui.label("Density:");
                        let density = table
                            .get("collider_density")
                            .and_then(|v| v.as_float())
                            .unwrap_or(1.0) as f32;
                        let mut d_edit = density;
                        if ui
                            .add(egui::DragValue::new(&mut d_edit).range(0.1..=10.0).speed(0.1))
                            .changed()
                        {
                            table.insert(
                                "collider_density".to_string(),
                                toml::Value::Float(d_edit as f64),
                            );
                            session.is_modified = true;
                        }
                    });
                });

            // Colliders section (simplified)
            egui::CollapsingHeader::new("Colliders")
                .default_open(false)
                .show(ui, |ui| {
                    if let Some(colliders) = table.get("colliders").and_then(|v| v.as_array()) {
                        ui.label(format!("{} collider(s)", colliders.len()));
                        for (i, _collider) in colliders.iter().enumerate() {
                            ui.label(format!("  Collider {}", i + 1));
                        }
                    } else {
                        ui.label("No colliders defined");
                    }
                    ui.colored_label(
                        egui::Color32::GRAY,
                        "Detailed collider editing coming soon",
                    );
                });

            // Behavior section (simplified)
            egui::CollapsingHeader::new("Behavior")
                .default_open(false)
                .show(ui, |ui| {
                    if table.contains_key("behavior") {
                        ui.label("Behavior tree defined");
                    } else {
                        ui.label("No behavior tree");
                    }
                    ui.colored_label(
                        egui::Color32::GRAY,
                        "Behavior tree editing coming soon",
                    );
                });
        });
}

/// Helper to get a Vec2 value from TOML
fn get_vec2_value(
    table: &toml::value::Table,
    key: &str,
    default_x: f32,
    default_y: f32,
) -> (f32, f32) {
    if let Some(arr) = table.get(key).and_then(|v| v.as_array()) {
        let x = arr.first().and_then(|v| v.as_float()).unwrap_or(default_x as f64) as f32;
        let y = arr.get(1).and_then(|v| v.as_float()).unwrap_or(default_y as f64) as f32;
        (x, y)
    } else {
        (default_x, default_y)
    }
}

/// Helper to set a Vec2 value in TOML
fn set_vec2_value(table: &mut toml::value::Table, key: &str, x: f32, y: f32) {
    table.insert(
        key.to_string(),
        toml::Value::Array(vec![
            toml::Value::Float(x as f64),
            toml::Value::Float(y as f64),
        ]),
    );
}
