//! Spawner editing functionality for the properties panel.
//!
//! This module handles rendering and editing of projectile spawners and mob spawners.

use bevy_egui::egui;

use crate::data::EditorSession;

use super::fields::{
    render_float_field, render_patch_indicator, render_reset_button, render_string_field,
    render_vec2_field, FieldResult, INHERITED_COLOR, PATCHED_COLOR,
};

/// Valid projectile types for spawners.
const PROJECTILE_TYPES: &[&str] = &["Bullet", "Blast"];

/// Valid faction types.
const FACTIONS: &[&str] = &["Enemy", "Ally"];

/// Check if a specific spawner field is patched.
fn is_spawner_field_patched(
    patch_table: &toml::value::Table,
    spawner_type: &str,
    spawner_name: &str,
    field: &str,
) -> bool {
    patch_table
        .get(spawner_type)
        .and_then(|v| v.as_table())
        .and_then(|t| t.get("spawners"))
        .and_then(|v| v.as_table())
        .and_then(|s| s.get(spawner_name))
        .and_then(|v| v.as_table())
        .map(|t| t.contains_key(field))
        .unwrap_or(false)
}

/// Set a spawner field value in the patch.
fn set_spawner_field(
    session: &mut EditorSession,
    spawner_type: &str,
    spawner_name: &str,
    field: &str,
    value: toml::Value,
) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        // Ensure spawner section exists
        if !mob.contains_key(spawner_type) {
            mob.insert(
                spawner_type.to_string(),
                toml::Value::Table(toml::value::Table::new()),
            );
        }
        let spawners_section = mob.get_mut(spawner_type).unwrap().as_table_mut().unwrap();

        // Ensure spawners exists
        if !spawners_section.contains_key("spawners") {
            spawners_section.insert(
                "spawners".to_string(),
                toml::Value::Table(toml::value::Table::new()),
            );
        }
        let spawners = spawners_section
            .get_mut("spawners")
            .unwrap()
            .as_table_mut()
            .unwrap();

        // Ensure this specific spawner exists
        if !spawners.contains_key(spawner_name) {
            spawners.insert(
                spawner_name.to_string(),
                toml::Value::Table(toml::value::Table::new()),
            );
        }
        let spawner = spawners
            .get_mut(spawner_name)
            .unwrap()
            .as_table_mut()
            .unwrap();

        // Set the field
        spawner.insert(field.to_string(), value);
    }
}

/// Remove a spawner field from the patch.
fn remove_spawner_field(
    session: &mut EditorSession,
    spawner_type: &str,
    spawner_name: &str,
    field: &str,
) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        if let Some(spawners_section) = mob.get_mut(spawner_type).and_then(|v| v.as_table_mut()) {
            if let Some(spawners) = spawners_section
                .get_mut("spawners")
                .and_then(|v| v.as_table_mut())
            {
                if let Some(spawner) = spawners
                    .get_mut(spawner_name)
                    .and_then(|v| v.as_table_mut())
                {
                    spawner.remove(field);

                    // Clean up empty tables
                    if spawner.is_empty() {
                        spawners.remove(spawner_name);
                    }
                }
                if spawners.is_empty() {
                    spawners_section.remove("spawners");
                }
            }
            if spawners_section.is_empty() {
                mob.remove(spawner_type);
            }
        }
    }
}

/// Render the projectile spawners section.
pub fn render_projectile_spawners_section(
    ui: &mut egui::Ui,
    display_table: &toml::value::Table,
    patch_table: &toml::value::Table,
    session: &mut EditorSession,
    is_patch: bool,
    modified: &mut bool,
) {
    let is_patched = is_patch && patch_table.contains_key("projectile_spawners");

    egui::CollapsingHeader::new("Projectile Spawners")
        .default_open(false)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                render_patch_indicator(ui, is_patched, is_patch);
                if is_patch && !is_patched {
                    ui.label(
                        egui::RichText::new("(inherited from base)")
                            .small()
                            .color(INHERITED_COLOR),
                    );
                } else if is_patch && is_patched {
                    ui.label(
                        egui::RichText::new("(overriding base)")
                            .small()
                            .color(PATCHED_COLOR),
                    );
                }
            });

            let mut delete_spawner: Option<String> = None;
            let mut rename_spawner: Option<(String, String)> = None;

            if let Some(proj_spawners) = display_table
                .get("projectile_spawners")
                .and_then(|v| v.as_table())
            {
                if let Some(spawners) = proj_spawners.get("spawners").and_then(|v| v.as_table()) {
                    let mut keys: Vec<_> = spawners.keys().cloned().collect();
                    keys.sort();

                    for key in keys {
                        if let Some(spawner) = spawners.get(&key).and_then(|v| v.as_table()) {
                            let spawner_key = key.clone();

                            egui::CollapsingHeader::new(format!("Spawner: {}", key))
                                .id_salt(format!("proj_spawner_{}", key))
                                .default_open(false)
                                .show(ui, |ui| {
                                    render_projectile_spawner_fields(
                                        ui,
                                        session,
                                        spawner,
                                        &spawner_key,
                                        patch_table,
                                        is_patch,
                                        modified,
                                        &mut delete_spawner,
                                        &mut rename_spawner,
                                    );
                                });
                        }
                    }
                }
            } else {
                ui.label("No projectile spawners");
            }

            // Handle delete and rename after the loop
            if let Some(name) = delete_spawner {
                delete_spawner_by_name(session, "projectile_spawners", &name);
                *modified = true;
            }
            if let Some((old_name, new_name)) = rename_spawner {
                rename_spawner_by_name(session, "projectile_spawners", &old_name, &new_name);
                *modified = true;
            }

            // Add new spawner button
            ui.separator();
            if ui.button("+ Add Spawner").clicked() {
                add_new_projectile_spawner(session, display_table);
                *modified = true;
            }
        });
}

/// Render fields for a single projectile spawner.
fn render_projectile_spawner_fields(
    ui: &mut egui::Ui,
    session: &mut EditorSession,
    spawner: &toml::value::Table,
    spawner_key: &str,
    patch_table: &toml::value::Table,
    is_patch: bool,
    modified: &mut bool,
    delete_spawner: &mut Option<String>,
    rename_spawner: &mut Option<(String, String)>,
) {
    // Name editing and delete button
    ui.horizontal(|ui| {
        ui.label("Name:");
        let mut name = spawner_key.to_string();
        let response = ui.text_edit_singleline(&mut name);
        if response.lost_focus() && name != spawner_key && !name.is_empty() {
            *rename_spawner = Some((spawner_key.to_string(), name));
        }
        if ui
            .add(egui::Button::new("🗑").fill(egui::Color32::from_rgb(120, 60, 60)))
            .on_hover_text("Delete spawner")
            .clicked()
        {
            *delete_spawner = Some(spawner_key.to_string());
        }
    });
    ui.separator();

    // Timer
    let field_patched =
        is_spawner_field_patched(patch_table, "projectile_spawners", spawner_key, "timer");
    let timer = spawner
        .get("timer")
        .and_then(|v| v.as_float())
        .unwrap_or(1.0) as f32;
    match render_float_field(
        ui,
        "Timer:",
        timer,
        0.01..=10.0,
        Some(0.01),
        field_patched,
        is_patch,
    ) {
        FieldResult::Changed(new_val) => {
            set_spawner_field(
                session,
                "projectile_spawners",
                spawner_key,
                "timer",
                toml::Value::Float(new_val as f64),
            );
            *modified = true;
        }
        FieldResult::Reset => {
            remove_spawner_field(session, "projectile_spawners", spawner_key, "timer");
            *modified = true;
        }
        FieldResult::NoChange => {}
    }

    // Position
    let field_patched =
        is_spawner_field_patched(patch_table, "projectile_spawners", spawner_key, "position");
    let pos = spawner.get("position").and_then(|v| v.as_array());
    let pos_x = pos
        .and_then(|a| a.first())
        .and_then(|v| v.as_float())
        .unwrap_or(0.0) as f32;
    let pos_y = pos
        .and_then(|a| a.get(1))
        .and_then(|v| v.as_float())
        .unwrap_or(0.0) as f32;
    match render_vec2_field(
        ui,
        "Position:",
        pos_x,
        pos_y,
        -100.0..=100.0,
        None,
        field_patched,
        is_patch,
    ) {
        FieldResult::Changed((x, y)) => {
            let arr = toml::Value::Array(vec![
                toml::Value::Float(x as f64),
                toml::Value::Float(y as f64),
            ]);
            set_spawner_field(session, "projectile_spawners", spawner_key, "position", arr);
            *modified = true;
        }
        FieldResult::Reset => {
            remove_spawner_field(session, "projectile_spawners", spawner_key, "position");
            *modified = true;
        }
        FieldResult::NoChange => {}
    }

    // Rotation
    let field_patched =
        is_spawner_field_patched(patch_table, "projectile_spawners", spawner_key, "rotation");
    let rot = spawner
        .get("rotation")
        .and_then(|v| v.as_float())
        .unwrap_or(0.0) as f32;
    match render_float_field(
        ui,
        "Rotation:",
        rot,
        -180.0..=180.0,
        None,
        field_patched,
        is_patch,
    ) {
        FieldResult::Changed(new_val) => {
            set_spawner_field(
                session,
                "projectile_spawners",
                spawner_key,
                "rotation",
                toml::Value::Float(new_val as f64),
            );
            *modified = true;
        }
        FieldResult::Reset => {
            remove_spawner_field(session, "projectile_spawners", spawner_key, "rotation");
            *modified = true;
        }
        FieldResult::NoChange => {}
    }

    // Projectile Type (dropdown)
    let field_patched = is_spawner_field_patched(
        patch_table,
        "projectile_spawners",
        spawner_key,
        "projectile_type",
    );
    let proj_type = spawner
        .get("projectile_type")
        .and_then(|v| v.as_str())
        .unwrap_or("Bullet");
    ui.horizontal(|ui| {
        render_patch_indicator(ui, field_patched, is_patch);
        let text_color = if is_patch && !field_patched {
            INHERITED_COLOR
        } else {
            ui.style().visuals.text_color()
        };
        ui.label(egui::RichText::new("Type:").color(text_color));
        let mut selected = proj_type.to_string();
        egui::ComboBox::from_id_salt(format!("proj_type_{}", spawner_key))
            .selected_text(&selected)
            .show_ui(ui, |ui| {
                for &ptype in PROJECTILE_TYPES {
                    if ui.selectable_label(selected == ptype, ptype).clicked() {
                        selected = ptype.to_string();
                    }
                }
            });
        if selected != proj_type {
            set_spawner_field(
                session,
                "projectile_spawners",
                spawner_key,
                "projectile_type",
                toml::Value::String(selected),
            );
            *modified = true;
        }
        if render_reset_button(ui, field_patched, is_patch) {
            remove_spawner_field(
                session,
                "projectile_spawners",
                spawner_key,
                "projectile_type",
            );
            *modified = true;
        }
    });

    // Faction (dropdown)
    let field_patched =
        is_spawner_field_patched(patch_table, "projectile_spawners", spawner_key, "faction");
    let faction = spawner
        .get("faction")
        .and_then(|v| v.as_str())
        .unwrap_or("Enemy");
    ui.horizontal(|ui| {
        render_patch_indicator(ui, field_patched, is_patch);
        let text_color = if is_patch && !field_patched {
            INHERITED_COLOR
        } else {
            ui.style().visuals.text_color()
        };
        ui.label(egui::RichText::new("Faction:").color(text_color));
        let mut selected = faction.to_string();
        egui::ComboBox::from_id_salt(format!("proj_faction_{}", spawner_key))
            .selected_text(&selected)
            .show_ui(ui, |ui| {
                for &f in FACTIONS {
                    if ui.selectable_label(selected == f, f).clicked() {
                        selected = f.to_string();
                    }
                }
            });
        if selected != faction {
            set_spawner_field(
                session,
                "projectile_spawners",
                spawner_key,
                "faction",
                toml::Value::String(selected),
            );
            *modified = true;
        }
        if render_reset_button(ui, field_patched, is_patch) {
            remove_spawner_field(session, "projectile_spawners", spawner_key, "faction");
            *modified = true;
        }
    });
}

/// Render the mob spawners section.
pub fn render_mob_spawners_section(
    ui: &mut egui::Ui,
    display_table: &toml::value::Table,
    patch_table: &toml::value::Table,
    session: &mut EditorSession,
    is_patch: bool,
    modified: &mut bool,
) {
    let is_patched = is_patch && patch_table.contains_key("mob_spawners");

    egui::CollapsingHeader::new("Mob Spawners")
        .default_open(false)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                render_patch_indicator(ui, is_patched, is_patch);
                if is_patch && !is_patched {
                    ui.label(
                        egui::RichText::new("(inherited from base)")
                            .small()
                            .color(INHERITED_COLOR),
                    );
                } else if is_patch && is_patched {
                    ui.label(
                        egui::RichText::new("(overriding base)")
                            .small()
                            .color(PATCHED_COLOR),
                    );
                }
            });

            let mut delete_spawner: Option<String> = None;
            let mut rename_spawner: Option<(String, String)> = None;

            if let Some(mob_spawners) = display_table.get("mob_spawners").and_then(|v| v.as_table())
            {
                if let Some(spawners) = mob_spawners.get("spawners").and_then(|v| v.as_table()) {
                    let mut keys: Vec<_> = spawners.keys().cloned().collect();
                    keys.sort();

                    for key in keys {
                        if let Some(spawner) = spawners.get(&key).and_then(|v| v.as_table()) {
                            let spawner_key = key.clone();

                            egui::CollapsingHeader::new(format!("Spawner: {}", key))
                                .id_salt(format!("mob_spawner_{}", key))
                                .default_open(false)
                                .show(ui, |ui| {
                                    render_mob_spawner_fields(
                                        ui,
                                        session,
                                        spawner,
                                        &spawner_key,
                                        patch_table,
                                        is_patch,
                                        modified,
                                        &mut delete_spawner,
                                        &mut rename_spawner,
                                    );
                                });
                        }
                    }
                }
            } else {
                ui.label("No mob spawners");
            }

            // Handle delete and rename after the loop
            if let Some(name) = delete_spawner {
                delete_spawner_by_name(session, "mob_spawners", &name);
                *modified = true;
            }
            if let Some((old_name, new_name)) = rename_spawner {
                rename_spawner_by_name(session, "mob_spawners", &old_name, &new_name);
                *modified = true;
            }

            // Add new mob spawner button
            ui.separator();
            if ui.button("+ Add Mob Spawner").clicked() {
                add_new_mob_spawner(session, display_table);
                *modified = true;
            }
        });
}

/// Render fields for a single mob spawner.
fn render_mob_spawner_fields(
    ui: &mut egui::Ui,
    session: &mut EditorSession,
    spawner: &toml::value::Table,
    spawner_key: &str,
    patch_table: &toml::value::Table,
    is_patch: bool,
    modified: &mut bool,
    delete_spawner: &mut Option<String>,
    rename_spawner: &mut Option<(String, String)>,
) {
    // Name editing and delete button
    ui.horizontal(|ui| {
        ui.label("Name:");
        let mut name = spawner_key.to_string();
        let response = ui.text_edit_singleline(&mut name);
        if response.lost_focus() && name != spawner_key && !name.is_empty() {
            *rename_spawner = Some((spawner_key.to_string(), name));
        }
        if ui
            .add(egui::Button::new("🗑").fill(egui::Color32::from_rgb(120, 60, 60)))
            .on_hover_text("Delete spawner")
            .clicked()
        {
            *delete_spawner = Some(spawner_key.to_string());
        }
    });
    ui.separator();

    // Timer
    let field_patched =
        is_spawner_field_patched(patch_table, "mob_spawners", spawner_key, "timer");
    let timer = spawner
        .get("timer")
        .and_then(|v| v.as_float())
        .unwrap_or(1.0) as f32;
    match render_float_field(
        ui,
        "Timer:",
        timer,
        0.01..=60.0,
        Some(0.1),
        field_patched,
        is_patch,
    ) {
        FieldResult::Changed(new_val) => {
            set_spawner_field(
                session,
                "mob_spawners",
                spawner_key,
                "timer",
                toml::Value::Float(new_val as f64),
            );
            *modified = true;
        }
        FieldResult::Reset => {
            remove_spawner_field(session, "mob_spawners", spawner_key, "timer");
            *modified = true;
        }
        FieldResult::NoChange => {}
    }

    // Position
    let field_patched =
        is_spawner_field_patched(patch_table, "mob_spawners", spawner_key, "position");
    let pos = spawner.get("position").and_then(|v| v.as_array());
    let pos_x = pos
        .and_then(|a| a.first())
        .and_then(|v| v.as_float())
        .unwrap_or(0.0) as f32;
    let pos_y = pos
        .and_then(|a| a.get(1))
        .and_then(|v| v.as_float())
        .unwrap_or(0.0) as f32;
    match render_vec2_field(
        ui,
        "Position:",
        pos_x,
        pos_y,
        -100.0..=100.0,
        None,
        field_patched,
        is_patch,
    ) {
        FieldResult::Changed((x, y)) => {
            let arr = toml::Value::Array(vec![
                toml::Value::Float(x as f64),
                toml::Value::Float(y as f64),
            ]);
            set_spawner_field(session, "mob_spawners", spawner_key, "position", arr);
            *modified = true;
        }
        FieldResult::Reset => {
            remove_spawner_field(session, "mob_spawners", spawner_key, "position");
            *modified = true;
        }
        FieldResult::NoChange => {}
    }

    // Rotation
    let field_patched =
        is_spawner_field_patched(patch_table, "mob_spawners", spawner_key, "rotation");
    let rot = spawner
        .get("rotation")
        .and_then(|v| v.as_float())
        .unwrap_or(0.0) as f32;
    match render_float_field(
        ui,
        "Rotation:",
        rot,
        -180.0..=180.0,
        None,
        field_patched,
        is_patch,
    ) {
        FieldResult::Changed(new_val) => {
            set_spawner_field(
                session,
                "mob_spawners",
                spawner_key,
                "rotation",
                toml::Value::Float(new_val as f64),
            );
            *modified = true;
        }
        FieldResult::Reset => {
            remove_spawner_field(session, "mob_spawners", spawner_key, "rotation");
            *modified = true;
        }
        FieldResult::NoChange => {}
    }

    // Mob Ref
    let field_patched =
        is_spawner_field_patched(patch_table, "mob_spawners", spawner_key, "mob_ref");
    let mob_ref = spawner
        .get("mob_ref")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    match render_string_field(ui, "Mob Ref:", mob_ref, field_patched, is_patch) {
        FieldResult::Changed(new_val) => {
            set_spawner_field(
                session,
                "mob_spawners",
                spawner_key,
                "mob_ref",
                toml::Value::String(new_val),
            );
            *modified = true;
        }
        FieldResult::Reset => {
            remove_spawner_field(session, "mob_spawners", spawner_key, "mob_ref");
            *modified = true;
        }
        FieldResult::NoChange => {}
    }
}

/// Generate a unique spawner name.
fn generate_unique_spawner_name(
    existing_spawners: Option<&toml::value::Table>,
    prefix: &str,
) -> String {
    let directions = ["north", "south", "east", "west", "center"];
    for dir in directions {
        let name = format!("{}_{}", prefix, dir);
        if existing_spawners
            .map(|s| !s.contains_key(&name))
            .unwrap_or(true)
        {
            return name;
        }
    }
    // Fallback to numbered names
    for i in 1..100 {
        let name = format!("{}_{}", prefix, i);
        if existing_spawners
            .map(|s| !s.contains_key(&name))
            .unwrap_or(true)
        {
            return name;
        }
    }
    format!("{}_new", prefix)
}

/// Add a new projectile spawner.
fn add_new_projectile_spawner(session: &mut EditorSession, display_table: &toml::value::Table) {
    let existing = display_table
        .get("projectile_spawners")
        .and_then(|v| v.as_table())
        .and_then(|t| t.get("spawners"))
        .and_then(|v| v.as_table());

    let name = generate_unique_spawner_name(existing, "spawner");

    // Create default spawner
    let mut spawner = toml::value::Table::new();
    spawner.insert("timer".to_string(), toml::Value::Float(1.0));
    spawner.insert(
        "position".to_string(),
        toml::Value::Array(vec![toml::Value::Float(0.0), toml::Value::Float(0.0)]),
    );
    spawner.insert("rotation".to_string(), toml::Value::Float(0.0));
    spawner.insert(
        "projectile_type".to_string(),
        toml::Value::String("Bullet".to_string()),
    );
    spawner.insert(
        "faction".to_string(),
        toml::Value::String("Enemy".to_string()),
    );

    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        // Ensure projectile_spawners.spawners exists
        if !mob.contains_key("projectile_spawners") {
            mob.insert(
                "projectile_spawners".to_string(),
                toml::Value::Table(toml::value::Table::new()),
            );
        }
        let ps = mob
            .get_mut("projectile_spawners")
            .unwrap()
            .as_table_mut()
            .unwrap();
        if !ps.contains_key("spawners") {
            ps.insert(
                "spawners".to_string(),
                toml::Value::Table(toml::value::Table::new()),
            );
        }
        let spawners = ps.get_mut("spawners").unwrap().as_table_mut().unwrap();
        spawners.insert(name, toml::Value::Table(spawner));
    }
}

/// Add a new mob spawner.
fn add_new_mob_spawner(session: &mut EditorSession, display_table: &toml::value::Table) {
    let existing = display_table
        .get("mob_spawners")
        .and_then(|v| v.as_table())
        .and_then(|t| t.get("spawners"))
        .and_then(|v| v.as_table());

    let name = generate_unique_spawner_name(existing, "mob");

    // Create default mob spawner
    let mut spawner = toml::value::Table::new();
    spawner.insert("timer".to_string(), toml::Value::Float(5.0));
    spawner.insert(
        "position".to_string(),
        toml::Value::Array(vec![toml::Value::Float(0.0), toml::Value::Float(0.0)]),
    );
    spawner.insert("rotation".to_string(), toml::Value::Float(0.0));
    spawner.insert("mob_ref".to_string(), toml::Value::String(String::new()));

    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        // Ensure mob_spawners.spawners exists
        if !mob.contains_key("mob_spawners") {
            mob.insert(
                "mob_spawners".to_string(),
                toml::Value::Table(toml::value::Table::new()),
            );
        }
        let ms = mob
            .get_mut("mob_spawners")
            .unwrap()
            .as_table_mut()
            .unwrap();
        if !ms.contains_key("spawners") {
            ms.insert(
                "spawners".to_string(),
                toml::Value::Table(toml::value::Table::new()),
            );
        }
        let spawners = ms.get_mut("spawners").unwrap().as_table_mut().unwrap();
        spawners.insert(name, toml::Value::Table(spawner));
    }
}

/// Delete a spawner by name.
fn delete_spawner_by_name(session: &mut EditorSession, spawner_type: &str, name: &str) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        if let Some(spawners_section) = mob.get_mut(spawner_type).and_then(|v| v.as_table_mut()) {
            if let Some(spawners) = spawners_section
                .get_mut("spawners")
                .and_then(|v| v.as_table_mut())
            {
                spawners.remove(name);

                // Clean up empty tables
                if spawners.is_empty() {
                    spawners_section.remove("spawners");
                }
            }
            if spawners_section.is_empty() {
                mob.remove(spawner_type);
            }
        }
    }
}

/// Rename a spawner.
fn rename_spawner_by_name(
    session: &mut EditorSession,
    spawner_type: &str,
    old_name: &str,
    new_name: &str,
) {
    if let Some(mob) = session.current_mob.as_mut().and_then(|v| v.as_table_mut()) {
        if let Some(spawners_section) = mob.get_mut(spawner_type).and_then(|v| v.as_table_mut()) {
            if let Some(spawners) = spawners_section
                .get_mut("spawners")
                .and_then(|v| v.as_table_mut())
            {
                // Check if new name already exists
                if spawners.contains_key(new_name) {
                    return; // Don't overwrite existing spawner
                }
                // Move the spawner data to the new key
                if let Some(data) = spawners.remove(old_name) {
                    spawners.insert(new_name.to_string(), data);
                }
            }
        }
    }
}
