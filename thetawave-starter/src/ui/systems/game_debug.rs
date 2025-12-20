#[cfg(feature = "debug")]
use bevy::ecs::error::Result;
#[cfg(feature = "debug")]
use bevy::ecs::system::Res;
#[cfg(feature = "debug")]
use bevy::ecs::system::ResMut;
#[cfg(feature = "debug")]
use bevy::ecs::{message::MessageWriter, system::Local};
#[cfg(feature = "debug")]
use bevy::math::Vec2;
#[cfg(feature = "debug")]
use bevy_egui::{
    EguiContexts,
    egui::{TopBottomPanel, containers::menu},
};
#[cfg(feature = "debug")]
use bevy_platform::collections::HashMap;

#[cfg(feature = "debug")]
use thetawave_camera::{Camera2DZoomEvent, Camera3DZoomEvent};
#[cfg(feature = "debug")]
use thetawave_core::{Faction, LoggingSettings};
#[cfg(feature = "debug")]
use thetawave_debug::{InspectorDebugSettings, ToggleMobViewWindowEvent};
#[cfg(feature = "debug")]
use thetawave_mobs::{MobDebugSettings, MobRegistry, SpawnMobEvent};
#[cfg(feature = "debug")]
use thetawave_physics::PhysicsDebugSettings;
#[cfg(feature = "debug")]
use thetawave_projectiles::{ProjectileSpread, ProjectileType, SpawnProjectileEvent};

/// System that handles the egui debug menu
#[cfg(feature = "debug")]
pub(in crate::ui) fn game_debug_menu_system(
    mut contexts: EguiContexts,
    mut mob_debug_settings: ResMut<MobDebugSettings>,
    mut physics_debug_settings: ResMut<PhysicsDebugSettings>,
    mut inspector_debug_settings: ResMut<InspectorDebugSettings>,
    mut logging_settings: ResMut<LoggingSettings>,
    mut spawn_mob_event_writer: MessageWriter<SpawnMobEvent>,
    mut spawn_projectile_event_writer: MessageWriter<SpawnProjectileEvent>,
    mut camera2d_zoom_event_writer: MessageWriter<Camera2DZoomEvent>,
    mut camera2d_zoom: Local<i8>,
    mut camera3d_zoom_event_writer: MessageWriter<Camera3DZoomEvent>,
    mut camera3d_zoom: Local<i8>,
    mut spawn_location: Local<Vec2>,
    mut toggle_mob_view_event_writer: MessageWriter<ToggleMobViewWindowEvent>,
    mob_registry: Res<MobRegistry>,
) -> Result {
    let mut camera2d_zoom_new = *camera2d_zoom;
    let mut camera3d_zoom_new = *camera3d_zoom;

    TopBottomPanel::top("menu_bar").show(contexts.ctx_mut()?, |ui| {
        menu::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("Inspector", |ui| {
                ui.checkbox(
                    &mut inspector_debug_settings.inspector_enabled,
                    "World Inspector",
                );
            });

            ui.menu_button("Logging", |ui| {
                ui.checkbox(&mut logging_settings.combat, "Combat");
                ui.checkbox(&mut logging_settings.abilities, "Abilities");
                ui.checkbox(&mut logging_settings.spawning, "Spawning");
                ui.checkbox(&mut logging_settings.particles, "Particles");
                ui.checkbox(&mut logging_settings.data, "Data Loading");
                ui.checkbox(&mut logging_settings.ui, "UI");
            });

            ui.menu_button("View", |ui| {
                use bevy_egui::egui::Slider;

                if ui.button("Show Mob View").clicked() {
                    toggle_mob_view_event_writer.write(ToggleMobViewWindowEvent);
                }

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("2D Zoom");
                    ui.add(Slider::new(&mut camera2d_zoom_new, -100..=100));
                });

                ui.horizontal(|ui| {
                    ui.label("3D Zoom");
                    ui.add(Slider::new(&mut camera3d_zoom_new, -100..=100));
                });
            });

            ui.menu_button("Physics", |ui| {
                ui.checkbox(&mut physics_debug_settings.gizmos_enabled, "Gizmos");
                ui.checkbox(
                    &mut physics_debug_settings.diagnostics_enabled,
                    "Diagnostics",
                );
            });

            ui.menu_button("Spawn", |ui| {
                ui.menu_button("Settings", |ui| {
                    ui.checkbox(&mut mob_debug_settings.joints_enabled, "Joints Enabled");
                    ui.checkbox(
                        &mut mob_debug_settings.behaviors_enabled,
                        "Behaviors Enabled",
                    );
                });

                ui.horizontal(|ui| {
                    use bevy_egui::egui::DragValue;

                    ui.label("Location");
                    ui.add(DragValue::new(&mut spawn_location.x));
                    ui.add(DragValue::new(&mut spawn_location.y));
                });

                ui.menu_button("Mob", |ui| {
                    // Collect spawnable mobs grouped by directory
                    let mut categories: HashMap<String, Vec<(String, String)>> = HashMap::new();
                    for (key, mob) in mob_registry.spawnable_mobs() {
                        let parts: Vec<&str> = key.split('/').collect();
                        let category = parts[0].to_string();
                        let display_name = mob.name.clone();
                        categories
                            .entry(category)
                            .or_default()
                            .push((key.clone(), display_name));
                    }

                    // Sort categories alphabetically
                    let mut sorted_categories: Vec<_> = categories.into_iter().collect();
                    sorted_categories.sort_by(|a, b| a.0.cmp(&b.0));

                    // Build sub-menus for each category
                    for (category, mobs) in &sorted_categories {
                        // Capitalize the category name for display
                        let category_display = capitalize_first(category);

                        ui.menu_button(&category_display, |ui| {
                            // Sort mobs within category by display name
                            let mut sorted_mobs = mobs.clone();
                            sorted_mobs.sort_by(|a, b| a.1.cmp(&b.1));

                            for (mob_ref, display_name) in sorted_mobs {
                                if ui.button(&display_name).clicked() {
                                    spawn_mob_event_writer.write(SpawnMobEvent::new(
                                        &mob_ref,
                                        *spawn_location,
                                        0.0,
                                    ));
                                }
                            }
                        });
                    }
                });

                ui.menu_button("Projectile", |ui| {
                    ui.menu_button("Blast", |ui| {
                        if ui.button("Ally").clicked() {
                            spawn_projectile_event_writer.write(SpawnProjectileEvent {
                                projectile_type: ProjectileType::Blast,
                                projectile_spread: ProjectileSpread::Arc {
                                    max_spread: 11.5,
                                    projectile_gap: 5.7,
                                    spread_weights: 1.0,
                                },
                                count: 1,
                                faction: Faction::Ally,
                                position: *spawn_location,
                                scale: 1.0,
                                velocity: Vec2::ZERO,
                                damage: 5,
                                range_seconds: 10.0,
                            });
                        }

                        if ui.button("Enemy").clicked() {
                            spawn_projectile_event_writer.write(SpawnProjectileEvent {
                                projectile_type: ProjectileType::Blast,
                                projectile_spread: ProjectileSpread::Arc {
                                    max_spread: 11.5,
                                    projectile_gap: 5.7,
                                    spread_weights: 1.0,
                                },
                                count: 1,
                                faction: Faction::Enemy,
                                position: *spawn_location,
                                scale: 1.0,
                                velocity: Vec2::ZERO,
                                damage: 5,
                                range_seconds: 10.0,
                            });
                        }
                    });
                    ui.menu_button("Bullet", |ui| {
                        if ui.button("Ally").clicked() {
                            spawn_projectile_event_writer.write(SpawnProjectileEvent {
                                projectile_type: ProjectileType::Bullet,
                                projectile_spread: ProjectileSpread::Arc {
                                    max_spread: 11.5,
                                    projectile_gap: 5.7,
                                    spread_weights: 1.0,
                                },
                                count: 1,
                                faction: Faction::Ally,
                                position: *spawn_location,
                                scale: 1.0,
                                velocity: Vec2::ZERO,
                                damage: 5,
                                range_seconds: 10.0,
                            });
                        }

                        if ui.button("Enemy").clicked() {
                            spawn_projectile_event_writer.write(SpawnProjectileEvent {
                                projectile_type: ProjectileType::Bullet,
                                projectile_spread: ProjectileSpread::Arc {
                                    max_spread: 11.5,
                                    projectile_gap: 5.7,
                                    spread_weights: 1.0,
                                },
                                count: 1,
                                faction: Faction::Enemy,
                                position: *spawn_location,
                                scale: 1.0,
                                velocity: Vec2::ZERO,
                                damage: 5,
                                range_seconds: 10.0,
                            });
                        }
                    });
                });
            });
        })
    });

    // Update local variable and send zoom 2d event if it changed
    if camera2d_zoom_new != *camera2d_zoom {
        *camera2d_zoom = camera2d_zoom_new;
        camera2d_zoom_event_writer.write(Camera2DZoomEvent(*camera2d_zoom));
    }

    // Update local variable and send zoom 3d event if it changed
    if camera3d_zoom_new != *camera3d_zoom {
        *camera3d_zoom = camera3d_zoom_new;
        camera3d_zoom_event_writer.write(Camera3DZoomEvent(*camera3d_zoom));
    }

    Ok(())
}

/// Helper function to capitalize the first letter of a string.
#[cfg(feature = "debug")]
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
