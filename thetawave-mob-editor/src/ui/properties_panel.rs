use bevy_egui::egui;

use crate::data::EditorSession;

/// Render the properties editing panel
pub fn properties_panel_ui(ui: &mut egui::Ui, session: &mut EditorSession) {
    ui.heading("Properties");
    ui.separator();

    // Track if any value was modified during this frame
    let mut modified = false;

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
                            modified = true;
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
                            modified = true;
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
                            modified = true;
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
                            modified = true;
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
                            modified = true;
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
                            modified = true;
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
                            modified = true;
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
                                modified = true;
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
                            modified = true;
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
                            modified = true;
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
                            modified = true;
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
                            modified = true;
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
                            modified = true;
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
                            modified = true;
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
                            modified = true;
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
                            modified = true;
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
                            modified = true;
                        }
                    });
                });

            // Colliders section
            egui::CollapsingHeader::new("Colliders")
                .default_open(false)
                .show(ui, |ui| {
                    let colliders = table
                        .get("colliders")
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default();

                    let mut colliders_modified = false;
                    let mut new_colliders = colliders.clone();
                    let mut to_remove: Option<usize> = None;

                    for (i, collider) in colliders.iter().enumerate() {
                        let id = ui.make_persistent_id(format!("collider_{}", i));
                        egui::CollapsingHeader::new(format!("Collider {}", i + 1))
                            .id_salt(id)
                            .default_open(false)
                            .show(ui, |ui| {
                                if let Some(collider_table) = collider.as_table() {
                                    let mut c = collider_table.clone();

                                    // Shape type selection
                                    let current_shape = c.get("shape").and_then(|s| s.as_table());
                                    let shape_type = if current_shape
                                        .map(|s| s.contains_key("Circle"))
                                        .unwrap_or(false)
                                    {
                                        0 // Circle
                                    } else {
                                        1 // Rectangle
                                    };

                                    ui.horizontal(|ui| {
                                        ui.label("Shape:");
                                        let mut new_shape_type = shape_type;
                                        egui::ComboBox::from_id_salt(format!("shape_type_{}", i))
                                            .selected_text(if shape_type == 0 {
                                                "Circle"
                                            } else {
                                                "Rectangle"
                                            })
                                            .show_ui(ui, |ui| {
                                                ui.selectable_value(
                                                    &mut new_shape_type,
                                                    0,
                                                    "Circle",
                                                );
                                                ui.selectable_value(
                                                    &mut new_shape_type,
                                                    1,
                                                    "Rectangle",
                                                );
                                            });

                                        if new_shape_type != shape_type {
                                            let new_shape = if new_shape_type == 0 {
                                                let mut shape_table = toml::value::Table::new();
                                                shape_table.insert(
                                                    "Circle".to_string(),
                                                    toml::Value::Float(10.0),
                                                );
                                                toml::Value::Table(shape_table)
                                            } else {
                                                let mut shape_table = toml::value::Table::new();
                                                shape_table.insert(
                                                    "Rectangle".to_string(),
                                                    toml::Value::Array(vec![
                                                        toml::Value::Float(10.0),
                                                        toml::Value::Float(10.0),
                                                    ]),
                                                );
                                                toml::Value::Table(shape_table)
                                            };
                                            c.insert("shape".to_string(), new_shape);
                                            new_colliders[i] = toml::Value::Table(c.clone());
                                            colliders_modified = true;
                                        }
                                    });

                                    // Shape parameters
                                    if let Some(shape) = c.get("shape").and_then(|s| s.as_table()) {
                                        if let Some(radius) = shape.get("Circle") {
                                            let mut r = radius.as_float().unwrap_or(10.0) as f32;
                                            ui.horizontal(|ui| {
                                                ui.label("Radius:");
                                                if ui
                                                    .add(
                                                        egui::DragValue::new(&mut r)
                                                            .range(0.1..=100.0)
                                                            .speed(0.5),
                                                    )
                                                    .changed()
                                                {
                                                    let mut shape_table =
                                                        toml::value::Table::new();
                                                    shape_table.insert(
                                                        "Circle".to_string(),
                                                        toml::Value::Float(r as f64),
                                                    );
                                                    c.insert(
                                                        "shape".to_string(),
                                                        toml::Value::Table(shape_table),
                                                    );
                                                    new_colliders[i] = toml::Value::Table(c.clone());
                                                    colliders_modified = true;
                                                }
                                            });
                                        } else if let Some(dims) = shape.get("Rectangle") {
                                            if let Some(arr) = dims.as_array() {
                                                let mut w = arr
                                                    .first()
                                                    .and_then(|v| v.as_float())
                                                    .unwrap_or(10.0)
                                                    as f32;
                                                let mut h = arr
                                                    .get(1)
                                                    .and_then(|v| v.as_float())
                                                    .unwrap_or(10.0)
                                                    as f32;

                                                ui.horizontal(|ui| {
                                                    ui.label("Width:");
                                                    let w_changed = ui
                                                        .add(
                                                            egui::DragValue::new(&mut w)
                                                                .range(0.1..=200.0)
                                                                .speed(0.5),
                                                        )
                                                        .changed();
                                                    ui.label("Height:");
                                                    let h_changed = ui
                                                        .add(
                                                            egui::DragValue::new(&mut h)
                                                                .range(0.1..=200.0)
                                                                .speed(0.5),
                                                        )
                                                        .changed();

                                                    if w_changed || h_changed {
                                                        let mut shape_table =
                                                            toml::value::Table::new();
                                                        shape_table.insert(
                                                            "Rectangle".to_string(),
                                                            toml::Value::Array(vec![
                                                                toml::Value::Float(w as f64),
                                                                toml::Value::Float(h as f64),
                                                            ]),
                                                        );
                                                        c.insert(
                                                            "shape".to_string(),
                                                            toml::Value::Table(shape_table),
                                                        );
                                                        new_colliders[i] =
                                                            toml::Value::Table(c.clone());
                                                        colliders_modified = true;
                                                    }
                                                });
                                            }
                                        }
                                    }

                                    // Position
                                    let pos = c
                                        .get("position")
                                        .and_then(|v| v.as_array())
                                        .map(|arr| {
                                            (
                                                arr.first()
                                                    .and_then(|v| v.as_float())
                                                    .unwrap_or(0.0)
                                                    as f32,
                                                arr.get(1)
                                                    .and_then(|v| v.as_float())
                                                    .unwrap_or(0.0)
                                                    as f32,
                                            )
                                        })
                                        .unwrap_or((0.0, 0.0));
                                    let mut px = pos.0;
                                    let mut py = pos.1;
                                    ui.horizontal(|ui| {
                                        ui.label("Pos X:");
                                        let x_changed = ui
                                            .add(
                                                egui::DragValue::new(&mut px)
                                                    .range(-100.0..=100.0)
                                                    .speed(0.5),
                                            )
                                            .changed();
                                        ui.label("Y:");
                                        let y_changed = ui
                                            .add(
                                                egui::DragValue::new(&mut py)
                                                    .range(-100.0..=100.0)
                                                    .speed(0.5),
                                            )
                                            .changed();

                                        if x_changed || y_changed {
                                            c.insert(
                                                "position".to_string(),
                                                toml::Value::Array(vec![
                                                    toml::Value::Float(px as f64),
                                                    toml::Value::Float(py as f64),
                                                ]),
                                            );
                                            new_colliders[i] = toml::Value::Table(c.clone());
                                            colliders_modified = true;
                                        }
                                    });

                                    // Rotation
                                    let mut rot = c
                                        .get("rotation")
                                        .and_then(|v| v.as_float())
                                        .unwrap_or(0.0) as f32;
                                    ui.horizontal(|ui| {
                                        ui.label("Rotation:");
                                        if ui
                                            .add(
                                                egui::DragValue::new(&mut rot)
                                                    .range(-180.0..=180.0)
                                                    .speed(1.0)
                                                    .suffix("°"),
                                            )
                                            .changed()
                                        {
                                            c.insert(
                                                "rotation".to_string(),
                                                toml::Value::Float(rot as f64),
                                            );
                                            new_colliders[i] = toml::Value::Table(c.clone());
                                            colliders_modified = true;
                                        }
                                    });

                                    // Remove button
                                    if ui
                                        .add(
                                            egui::Button::new("Remove")
                                                .fill(egui::Color32::from_rgb(80, 30, 30)),
                                        )
                                        .clicked()
                                    {
                                        to_remove = Some(i);
                                    }
                                }
                            });
                    }

                    // Handle removal
                    if let Some(idx) = to_remove {
                        new_colliders.remove(idx);
                        colliders_modified = true;
                    }

                    // Add collider button
                    if ui.button("+ Add Collider").clicked() {
                        let mut new_collider = toml::value::Table::new();
                        let mut shape_table = toml::value::Table::new();
                        shape_table.insert(
                            "Rectangle".to_string(),
                            toml::Value::Array(vec![
                                toml::Value::Float(10.0),
                                toml::Value::Float(10.0),
                            ]),
                        );
                        new_collider.insert("shape".to_string(), toml::Value::Table(shape_table));
                        new_collider.insert(
                            "position".to_string(),
                            toml::Value::Array(vec![
                                toml::Value::Float(0.0),
                                toml::Value::Float(0.0),
                            ]),
                        );
                        new_collider.insert("rotation".to_string(), toml::Value::Float(0.0));
                        new_colliders.push(toml::Value::Table(new_collider));
                        colliders_modified = true;
                    }

                    if colliders_modified {
                        table.insert("colliders".to_string(), toml::Value::Array(new_colliders));
                        modified = true;
                    }
                });

            // Projectile Spawners section
            egui::CollapsingHeader::new("Projectile Spawners")
                .default_open(false)
                .show(ui, |ui| {
                    let has_spawners = table.contains_key("projectile_spawners");
                    let mut spawners_modified = false;

                    if has_spawners {
                        // Get or create the projectile_spawners table
                        let spawners_table = table
                            .get("projectile_spawners")
                            .and_then(|v| v.as_table())
                            .cloned()
                            .unwrap_or_default();

                        let inner_spawners = spawners_table
                            .get("spawners")
                            .and_then(|v| v.as_table())
                            .cloned()
                            .unwrap_or_default();

                        let mut new_spawners = inner_spawners.clone();
                        let mut to_remove: Option<String> = None;

                        // Sort keys for consistent display
                        let mut keys: Vec<_> = inner_spawners.keys().cloned().collect();
                        keys.sort();

                        for key in keys {
                            if let Some(spawner) = inner_spawners.get(&key) {
                                let id = ui.make_persistent_id(format!("proj_spawner_{}", key));
                                egui::CollapsingHeader::new(format!("Spawner: {}", key))
                                    .id_salt(id)
                                    .default_open(false)
                                    .show(ui, |ui| {
                                        if let Some(spawner_table) = spawner.as_table() {
                                            let mut s = spawner_table.clone();

                                            // Timer
                                            let mut timer = s
                                                .get("timer")
                                                .and_then(|v| v.as_float())
                                                .unwrap_or(1.0)
                                                as f32;
                                            ui.horizontal(|ui| {
                                                ui.label("Timer (s):");
                                                if ui
                                                    .add(
                                                        egui::DragValue::new(&mut timer)
                                                            .range(0.1..=10.0)
                                                            .speed(0.05),
                                                    )
                                                    .changed()
                                                {
                                                    s.insert(
                                                        "timer".to_string(),
                                                        toml::Value::Float(timer as f64),
                                                    );
                                                    new_spawners.insert(
                                                        key.clone(),
                                                        toml::Value::Table(s.clone()),
                                                    );
                                                    spawners_modified = true;
                                                }
                                            });

                                            // Position
                                            let pos = s
                                                .get("position")
                                                .and_then(|v| v.as_array())
                                                .map(|arr| {
                                                    (
                                                        arr.first()
                                                            .and_then(|v| v.as_float())
                                                            .unwrap_or(0.0)
                                                            as f32,
                                                        arr.get(1)
                                                            .and_then(|v| v.as_float())
                                                            .unwrap_or(0.0)
                                                            as f32,
                                                    )
                                                })
                                                .unwrap_or((0.0, 0.0));
                                            let mut px = pos.0;
                                            let mut py = pos.1;
                                            ui.horizontal(|ui| {
                                                ui.label("Pos X:");
                                                let x_changed = ui
                                                    .add(
                                                        egui::DragValue::new(&mut px)
                                                            .range(-100.0..=100.0)
                                                            .speed(0.5),
                                                    )
                                                    .changed();
                                                ui.label("Y:");
                                                let y_changed = ui
                                                    .add(
                                                        egui::DragValue::new(&mut py)
                                                            .range(-100.0..=100.0)
                                                            .speed(0.5),
                                                    )
                                                    .changed();

                                                if x_changed || y_changed {
                                                    s.insert(
                                                        "position".to_string(),
                                                        toml::Value::Array(vec![
                                                            toml::Value::Float(px as f64),
                                                            toml::Value::Float(py as f64),
                                                        ]),
                                                    );
                                                    new_spawners.insert(
                                                        key.clone(),
                                                        toml::Value::Table(s.clone()),
                                                    );
                                                    spawners_modified = true;
                                                }
                                            });

                                            // Rotation
                                            let mut rot = s
                                                .get("rotation")
                                                .and_then(|v| v.as_float())
                                                .unwrap_or(0.0)
                                                as f32;
                                            ui.horizontal(|ui| {
                                                ui.label("Rotation:");
                                                if ui
                                                    .add(
                                                        egui::DragValue::new(&mut rot)
                                                            .range(-180.0..=180.0)
                                                            .speed(1.0)
                                                            .suffix("°"),
                                                    )
                                                    .changed()
                                                {
                                                    s.insert(
                                                        "rotation".to_string(),
                                                        toml::Value::Float(rot as f64),
                                                    );
                                                    new_spawners.insert(
                                                        key.clone(),
                                                        toml::Value::Table(s.clone()),
                                                    );
                                                    spawners_modified = true;
                                                }
                                            });

                                            // Projectile Type
                                            let proj_type = s
                                                .get("projectile_type")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("Bullet")
                                                .to_string();
                                            let type_idx = if proj_type == "Blast" { 1 } else { 0 };
                                            ui.horizontal(|ui| {
                                                ui.label("Type:");
                                                let mut new_type_idx = type_idx;
                                                egui::ComboBox::from_id_salt(format!(
                                                    "proj_type_{}",
                                                    key
                                                ))
                                                .selected_text(&proj_type)
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(
                                                        &mut new_type_idx,
                                                        0,
                                                        "Bullet",
                                                    );
                                                    ui.selectable_value(
                                                        &mut new_type_idx,
                                                        1,
                                                        "Blast",
                                                    );
                                                });
                                                if new_type_idx != type_idx {
                                                    s.insert(
                                                        "projectile_type".to_string(),
                                                        toml::Value::String(
                                                            if new_type_idx == 1 {
                                                                "Blast"
                                                            } else {
                                                                "Bullet"
                                                            }
                                                            .to_string(),
                                                        ),
                                                    );
                                                    new_spawners.insert(
                                                        key.clone(),
                                                        toml::Value::Table(s.clone()),
                                                    );
                                                    spawners_modified = true;
                                                }
                                            });

                                            // Faction
                                            let faction = s
                                                .get("faction")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("Enemy")
                                                .to_string();
                                            let faction_idx = if faction == "Ally" { 0 } else { 1 };
                                            ui.horizontal(|ui| {
                                                ui.label("Faction:");
                                                let mut new_faction_idx = faction_idx;
                                                egui::ComboBox::from_id_salt(format!(
                                                    "faction_{}",
                                                    key
                                                ))
                                                .selected_text(&faction)
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(
                                                        &mut new_faction_idx,
                                                        0,
                                                        "Ally",
                                                    );
                                                    ui.selectable_value(
                                                        &mut new_faction_idx,
                                                        1,
                                                        "Enemy",
                                                    );
                                                });
                                                if new_faction_idx != faction_idx {
                                                    s.insert(
                                                        "faction".to_string(),
                                                        toml::Value::String(
                                                            if new_faction_idx == 0 {
                                                                "Ally"
                                                            } else {
                                                                "Enemy"
                                                            }
                                                            .to_string(),
                                                        ),
                                                    );
                                                    new_spawners.insert(
                                                        key.clone(),
                                                        toml::Value::Table(s.clone()),
                                                    );
                                                    spawners_modified = true;
                                                }
                                            });

                                            // Count
                                            let mut count = s
                                                .get("count")
                                                .and_then(|v| v.as_integer())
                                                .unwrap_or(1)
                                                as i32;
                                            ui.horizontal(|ui| {
                                                ui.label("Count:");
                                                if ui
                                                    .add(
                                                        egui::DragValue::new(&mut count)
                                                            .range(1..=20),
                                                    )
                                                    .changed()
                                                {
                                                    s.insert(
                                                        "count".to_string(),
                                                        toml::Value::Integer(count as i64),
                                                    );
                                                    new_spawners.insert(
                                                        key.clone(),
                                                        toml::Value::Table(s.clone()),
                                                    );
                                                    spawners_modified = true;
                                                }
                                            });

                                            // Multipliers (collapsed)
                                            egui::CollapsingHeader::new("Multipliers")
                                                .id_salt(format!("mult_{}", key))
                                                .default_open(false)
                                                .show(ui, |ui| {
                                                    let mut speed_mult = s
                                                        .get("speed_multiplier")
                                                        .and_then(|v| v.as_float())
                                                        .unwrap_or(1.0)
                                                        as f32;
                                                    ui.horizontal(|ui| {
                                                        ui.label("Speed:");
                                                        if ui
                                                            .add(
                                                                egui::DragValue::new(&mut speed_mult)
                                                                    .range(0.1..=5.0)
                                                                    .speed(0.05),
                                                            )
                                                            .changed()
                                                        {
                                                            s.insert(
                                                                "speed_multiplier".to_string(),
                                                                toml::Value::Float(
                                                                    speed_mult as f64,
                                                                ),
                                                            );
                                                            new_spawners.insert(
                                                                key.clone(),
                                                                toml::Value::Table(s.clone()),
                                                            );
                                                            spawners_modified = true;
                                                        }
                                                    });

                                                    let mut damage_mult = s
                                                        .get("damage_multiplier")
                                                        .and_then(|v| v.as_float())
                                                        .unwrap_or(1.0)
                                                        as f32;
                                                    ui.horizontal(|ui| {
                                                        ui.label("Damage:");
                                                        if ui
                                                            .add(
                                                                egui::DragValue::new(
                                                                    &mut damage_mult,
                                                                )
                                                                .range(0.1..=5.0)
                                                                .speed(0.05),
                                                            )
                                                            .changed()
                                                        {
                                                            s.insert(
                                                                "damage_multiplier".to_string(),
                                                                toml::Value::Float(
                                                                    damage_mult as f64,
                                                                ),
                                                            );
                                                            new_spawners.insert(
                                                                key.clone(),
                                                                toml::Value::Table(s.clone()),
                                                            );
                                                            spawners_modified = true;
                                                        }
                                                    });

                                                    let mut range_mult = s
                                                        .get("range_seconds_multiplier")
                                                        .and_then(|v| v.as_float())
                                                        .unwrap_or(1.0)
                                                        as f32;
                                                    ui.horizontal(|ui| {
                                                        ui.label("Range:");
                                                        if ui
                                                            .add(
                                                                egui::DragValue::new(&mut range_mult)
                                                                    .range(0.1..=5.0)
                                                                    .speed(0.05),
                                                            )
                                                            .changed()
                                                        {
                                                            s.insert(
                                                                "range_seconds_multiplier"
                                                                    .to_string(),
                                                                toml::Value::Float(
                                                                    range_mult as f64,
                                                                ),
                                                            );
                                                            new_spawners.insert(
                                                                key.clone(),
                                                                toml::Value::Table(s.clone()),
                                                            );
                                                            spawners_modified = true;
                                                        }
                                                    });
                                                });

                                            // Remove button
                                            if ui
                                                .add(
                                                    egui::Button::new("Remove")
                                                        .fill(egui::Color32::from_rgb(80, 30, 30)),
                                                )
                                                .clicked()
                                            {
                                                to_remove = Some(key.clone());
                                            }
                                        }
                                    });
                            }
                        }

                        // Handle removal
                        if let Some(key) = to_remove {
                            new_spawners.remove(&key);
                            spawners_modified = true;
                        }

                        // Add spawner button with key input
                        ui.horizontal(|ui| {
                            if ui.button("+ Add Spawner").clicked() {
                                // Find a unique key
                                let mut new_key = "south".to_string();
                                let mut counter = 1;
                                while new_spawners.contains_key(&new_key) {
                                    new_key = format!("spawner_{}", counter);
                                    counter += 1;
                                }

                                let mut new_spawner = toml::value::Table::new();
                                new_spawner.insert(
                                    "timer".to_string(),
                                    toml::Value::Float(1.0),
                                );
                                new_spawner.insert(
                                    "position".to_string(),
                                    toml::Value::Array(vec![
                                        toml::Value::Float(0.0),
                                        toml::Value::Float(-10.0),
                                    ]),
                                );
                                new_spawner.insert(
                                    "rotation".to_string(),
                                    toml::Value::Float(0.0),
                                );
                                new_spawner.insert(
                                    "projectile_type".to_string(),
                                    toml::Value::String("Bullet".to_string()),
                                );
                                new_spawner.insert(
                                    "faction".to_string(),
                                    toml::Value::String("Enemy".to_string()),
                                );
                                new_spawners.insert(new_key, toml::Value::Table(new_spawner));
                                spawners_modified = true;
                            }
                        });

                        if spawners_modified {
                            let mut new_proj_spawners = toml::value::Table::new();
                            new_proj_spawners
                                .insert("spawners".to_string(), toml::Value::Table(new_spawners));
                            table.insert(
                                "projectile_spawners".to_string(),
                                toml::Value::Table(new_proj_spawners),
                            );
                            modified = true;
                        }
                    } else {
                        ui.label("No projectile spawners");
                        if ui.button("+ Add Projectile Spawners").clicked() {
                            let mut spawners = toml::value::Table::new();
                            let mut inner = toml::value::Table::new();

                            let mut new_spawner = toml::value::Table::new();
                            new_spawner.insert("timer".to_string(), toml::Value::Float(1.0));
                            new_spawner.insert(
                                "position".to_string(),
                                toml::Value::Array(vec![
                                    toml::Value::Float(0.0),
                                    toml::Value::Float(-10.0),
                                ]),
                            );
                            new_spawner.insert("rotation".to_string(), toml::Value::Float(0.0));
                            new_spawner.insert(
                                "projectile_type".to_string(),
                                toml::Value::String("Bullet".to_string()),
                            );
                            new_spawner.insert(
                                "faction".to_string(),
                                toml::Value::String("Enemy".to_string()),
                            );

                            inner.insert("south".to_string(), toml::Value::Table(new_spawner));
                            spawners.insert("spawners".to_string(), toml::Value::Table(inner));
                            table.insert(
                                "projectile_spawners".to_string(),
                                toml::Value::Table(spawners),
                            );
                            modified = true;
                        }
                    }
                });

            // Mob Spawners section
            egui::CollapsingHeader::new("Mob Spawners")
                .default_open(false)
                .show(ui, |ui| {
                    let has_spawners = table.contains_key("mob_spawners");
                    let mut spawners_modified = false;

                    if has_spawners {
                        let spawners_table = table
                            .get("mob_spawners")
                            .and_then(|v| v.as_table())
                            .cloned()
                            .unwrap_or_default();

                        let inner_spawners = spawners_table
                            .get("spawners")
                            .and_then(|v| v.as_table())
                            .cloned()
                            .unwrap_or_default();

                        let mut new_spawners = inner_spawners.clone();
                        let mut to_remove: Option<String> = None;

                        let mut keys: Vec<_> = inner_spawners.keys().cloned().collect();
                        keys.sort();

                        for key in keys {
                            if let Some(spawner) = inner_spawners.get(&key) {
                                let id = ui.make_persistent_id(format!("mob_spawner_{}", key));
                                egui::CollapsingHeader::new(format!("Spawner: {}", key))
                                    .id_salt(id)
                                    .default_open(false)
                                    .show(ui, |ui| {
                                        if let Some(spawner_table) = spawner.as_table() {
                                            let mut s = spawner_table.clone();

                                            // Timer
                                            let mut timer = s
                                                .get("timer")
                                                .and_then(|v| v.as_float())
                                                .unwrap_or(1.0)
                                                as f32;
                                            ui.horizontal(|ui| {
                                                ui.label("Timer (s):");
                                                if ui
                                                    .add(
                                                        egui::DragValue::new(&mut timer)
                                                            .range(0.1..=30.0)
                                                            .speed(0.1),
                                                    )
                                                    .changed()
                                                {
                                                    s.insert(
                                                        "timer".to_string(),
                                                        toml::Value::Float(timer as f64),
                                                    );
                                                    new_spawners.insert(
                                                        key.clone(),
                                                        toml::Value::Table(s.clone()),
                                                    );
                                                    spawners_modified = true;
                                                }
                                            });

                                            // Position
                                            let pos = s
                                                .get("position")
                                                .and_then(|v| v.as_array())
                                                .map(|arr| {
                                                    (
                                                        arr.first()
                                                            .and_then(|v| v.as_float())
                                                            .unwrap_or(0.0)
                                                            as f32,
                                                        arr.get(1)
                                                            .and_then(|v| v.as_float())
                                                            .unwrap_or(0.0)
                                                            as f32,
                                                    )
                                                })
                                                .unwrap_or((0.0, 0.0));
                                            let mut px = pos.0;
                                            let mut py = pos.1;
                                            ui.horizontal(|ui| {
                                                ui.label("Pos X:");
                                                let x_changed = ui
                                                    .add(
                                                        egui::DragValue::new(&mut px)
                                                            .range(-200.0..=200.0)
                                                            .speed(1.0),
                                                    )
                                                    .changed();
                                                ui.label("Y:");
                                                let y_changed = ui
                                                    .add(
                                                        egui::DragValue::new(&mut py)
                                                            .range(-200.0..=200.0)
                                                            .speed(1.0),
                                                    )
                                                    .changed();

                                                if x_changed || y_changed {
                                                    s.insert(
                                                        "position".to_string(),
                                                        toml::Value::Array(vec![
                                                            toml::Value::Float(px as f64),
                                                            toml::Value::Float(py as f64),
                                                        ]),
                                                    );
                                                    new_spawners.insert(
                                                        key.clone(),
                                                        toml::Value::Table(s.clone()),
                                                    );
                                                    spawners_modified = true;
                                                }
                                            });

                                            // Rotation
                                            let mut rot = s
                                                .get("rotation")
                                                .and_then(|v| v.as_float())
                                                .unwrap_or(0.0)
                                                as f32;
                                            ui.horizontal(|ui| {
                                                ui.label("Rotation:");
                                                if ui
                                                    .add(
                                                        egui::DragValue::new(&mut rot)
                                                            .range(-180.0..=180.0)
                                                            .speed(1.0)
                                                            .suffix("°"),
                                                    )
                                                    .changed()
                                                {
                                                    s.insert(
                                                        "rotation".to_string(),
                                                        toml::Value::Float(rot as f64),
                                                    );
                                                    new_spawners.insert(
                                                        key.clone(),
                                                        toml::Value::Table(s.clone()),
                                                    );
                                                    spawners_modified = true;
                                                }
                                            });

                                            // Mob Ref
                                            let mob_ref = s
                                                .get("mob_ref")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("")
                                                .to_string();
                                            let mut mob_ref_edit = mob_ref.clone();
                                            ui.horizontal(|ui| {
                                                ui.label("Mob Ref:");
                                                if ui
                                                    .text_edit_singleline(&mut mob_ref_edit)
                                                    .changed()
                                                {
                                                    s.insert(
                                                        "mob_ref".to_string(),
                                                        toml::Value::String(mob_ref_edit),
                                                    );
                                                    new_spawners.insert(
                                                        key.clone(),
                                                        toml::Value::Table(s.clone()),
                                                    );
                                                    spawners_modified = true;
                                                }
                                            });
                                            ui.label(
                                                egui::RichText::new(
                                                    "e.g. xhitara/grunt or xhitara/spitter",
                                                )
                                                .small()
                                                .color(egui::Color32::GRAY),
                                            );

                                            // Remove button
                                            if ui
                                                .add(
                                                    egui::Button::new("Remove")
                                                        .fill(egui::Color32::from_rgb(80, 30, 30)),
                                                )
                                                .clicked()
                                            {
                                                to_remove = Some(key.clone());
                                            }
                                        }
                                    });
                            }
                        }

                        // Handle removal
                        if let Some(key) = to_remove {
                            new_spawners.remove(&key);
                            spawners_modified = true;
                        }

                        // Add spawner button
                        if ui.button("+ Add Mob Spawner").clicked() {
                            let mut new_key = "spawn_1".to_string();
                            let mut counter = 1;
                            while new_spawners.contains_key(&new_key) {
                                counter += 1;
                                new_key = format!("spawn_{}", counter);
                            }

                            let mut new_spawner = toml::value::Table::new();
                            new_spawner.insert("timer".to_string(), toml::Value::Float(5.0));
                            new_spawner.insert(
                                "position".to_string(),
                                toml::Value::Array(vec![
                                    toml::Value::Float(0.0),
                                    toml::Value::Float(0.0),
                                ]),
                            );
                            new_spawner.insert("rotation".to_string(), toml::Value::Float(0.0));
                            new_spawner.insert(
                                "mob_ref".to_string(),
                                toml::Value::String("xhitara/grunt".to_string()),
                            );
                            new_spawners.insert(new_key, toml::Value::Table(new_spawner));
                            spawners_modified = true;
                        }

                        if spawners_modified {
                            let mut new_mob_spawners = toml::value::Table::new();
                            new_mob_spawners
                                .insert("spawners".to_string(), toml::Value::Table(new_spawners));
                            table.insert(
                                "mob_spawners".to_string(),
                                toml::Value::Table(new_mob_spawners),
                            );
                            modified = true;
                        }
                    } else {
                        ui.label("No mob spawners");
                        if ui.button("+ Add Mob Spawners").clicked() {
                            let mut spawners = toml::value::Table::new();
                            let mut inner = toml::value::Table::new();

                            let mut new_spawner = toml::value::Table::new();
                            new_spawner.insert("timer".to_string(), toml::Value::Float(5.0));
                            new_spawner.insert(
                                "position".to_string(),
                                toml::Value::Array(vec![
                                    toml::Value::Float(0.0),
                                    toml::Value::Float(0.0),
                                ]),
                            );
                            new_spawner.insert("rotation".to_string(), toml::Value::Float(0.0));
                            new_spawner.insert(
                                "mob_ref".to_string(),
                                toml::Value::String("xhitara/grunt".to_string()),
                            );

                            inner.insert("spawn_1".to_string(), toml::Value::Table(new_spawner));
                            spawners.insert("spawners".to_string(), toml::Value::Table(inner));
                            table
                                .insert("mob_spawners".to_string(), toml::Value::Table(spawners));
                            modified = true;
                        }
                    }
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

    // Check if modified after all edits
    if modified {
        session.check_modified();
    }
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
