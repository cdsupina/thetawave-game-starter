#[cfg(feature = "debug")]
use bevy::ecs::{event::EventWriter, system::Local};
use bevy::{
    ecs::{error::Result, system::ResMut},
    math::Vec2,
};
#[cfg(feature = "debug")]
use bevy_egui::{
    EguiContexts,
    egui::{TopBottomPanel, containers::menu},
};

#[cfg(feature = "debug")]
use thetawave_camera::{Camera2DZoomEvent, Camera3DZoomEvent};
#[cfg(feature = "debug")]
use thetawave_core::Faction;
#[cfg(feature = "debug")]
use thetawave_debug::InspectorDebugSettings;
#[cfg(feature = "debug")]
use thetawave_mobs::{MobDebugSettings, MobType, SpawnMobEvent};
#[cfg(feature = "debug")]
use thetawave_physics::PhysicsDebugSettings;
#[cfg(feature = "debug")]
use thetawave_projectiles::{ProjectileType, SpawnProjectileEvent};

/// System that handles the egui debug menu
#[cfg(feature = "debug")]
pub(in crate::ui) fn game_debug_menu_system(
    mut contexts: EguiContexts,
    mut mob_debug_settings: ResMut<MobDebugSettings>,
    mut physics_debug_settings: ResMut<PhysicsDebugSettings>,
    mut inspector_debug_settings: ResMut<InspectorDebugSettings>,
    mut spawn_mob_event_writer: EventWriter<SpawnMobEvent>,
    mut spawn_projectile_event_writer: EventWriter<SpawnProjectileEvent>,
    mut camera2d_zoom_event_writer: EventWriter<Camera2DZoomEvent>,
    mut camera2d_zoom: Local<i8>,
    mut camera3d_zoom_event_writer: EventWriter<Camera3DZoomEvent>,
    mut camera3d_zoom: Local<i8>,
    mut spawn_location: Local<Vec2>,
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

            ui.menu_button("View", |ui| {
                use bevy_egui::egui::Slider;

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
                    // Xhitara Mobs
                    ui.menu_button("Xhitara", |ui| {
                        if ui.button("Xhitara Grunt").clicked() {
                            spawn_mob_event_writer.write(SpawnMobEvent {
                                mob_type: MobType::XhitaraGrunt,
                                position: *spawn_location,
                                rotation: 0.0,
                            });
                        }

                        if ui.button("Xhitara Spitter").clicked() {
                            spawn_mob_event_writer.write(SpawnMobEvent {
                                mob_type: MobType::XhitaraSpitter,
                                position: *spawn_location,
                                rotation: 0.0,
                            });
                        }

                        if ui.button("Xhitara Gyro").clicked() {
                            spawn_mob_event_writer.write(SpawnMobEvent {
                                mob_type: MobType::XhitaraGyro,
                                position: *spawn_location,
                                rotation: 0.0,
                            });
                        }

                        if ui.button("Xhitara Pacer").clicked() {
                            spawn_mob_event_writer.write(SpawnMobEvent {
                                mob_type: MobType::XhitaraPacer,
                                position: *spawn_location,
                                rotation: 0.0,
                            });
                        }

                        if ui.button("Xhitara Missile").clicked() {
                            spawn_mob_event_writer.write(SpawnMobEvent {
                                mob_type: MobType::XhitaraMissile,
                                position: *spawn_location,
                                rotation: 0.0,
                            });
                        }

                        if ui.button("Xhitara Launcher").clicked() {
                            spawn_mob_event_writer.write(SpawnMobEvent {
                                mob_type: MobType::XhitaraLauncher,
                                position: *spawn_location,
                                rotation: 0.0,
                            });
                        }

                        if ui.button("Xhitara Cyclusk").clicked() {
                            spawn_mob_event_writer.write(SpawnMobEvent {
                                mob_type: MobType::XhitaraCyclusk,
                                position: *spawn_location,
                                rotation: 0.0,
                            });
                        }

                        if ui.button("Ferritharax").clicked() {
                            spawn_mob_event_writer.write(SpawnMobEvent {
                                mob_type: MobType::Ferritharax,
                                position: *spawn_location,
                                rotation: 0.0,
                            });
                        }

                        if ui.button("Trizetheron").clicked() {
                            spawn_mob_event_writer.write(SpawnMobEvent {
                                mob_type: MobType::Trizetheron,
                                position: *spawn_location,
                                rotation: 0.0,
                            });
                        }

                        ui.menu_button("Misc", |ui| {
                            ui.menu_button("Xhitara Tentacle", |ui| {
                                if ui.button("Xhitara Tentacle Short").clicked() {
                                    spawn_mob_event_writer.write(SpawnMobEvent {
                                        mob_type: MobType::XhitaraTentacleShort,
                                        position: *spawn_location,
                                        rotation: 0.0,
                                    });
                                }

                                if ui.button("Xhitara Tentacle Long").clicked() {
                                    spawn_mob_event_writer.write(SpawnMobEvent {
                                        mob_type: MobType::XhitaraTentacleLong,
                                        position: *spawn_location,
                                        rotation: 0.0,
                                    });
                                }

                                if ui.button("Xhitara Tentacle Middle").clicked() {
                                    spawn_mob_event_writer.write(SpawnMobEvent {
                                        mob_type: MobType::XhitaraTentacleMiddle,
                                        position: *spawn_location,
                                        rotation: 0.0,
                                    });
                                }

                                if ui.button("Xhitara Tentacle End").clicked() {
                                    spawn_mob_event_writer.write(SpawnMobEvent {
                                        mob_type: MobType::XhitaraTentacleEnd,
                                        position: *spawn_location,
                                        rotation: 0.0,
                                    });
                                }
                            });
                        });
                    });

                    // Ally Mobs
                    ui.menu_button("Ally", |ui| {
                        if ui.button("Freighter One").clicked() {
                            spawn_mob_event_writer.write(SpawnMobEvent {
                                mob_type: MobType::FreighterOne,
                                position: *spawn_location,
                                rotation: 0.0,
                            });
                        }

                        if ui.button("Freighter Two").clicked() {
                            spawn_mob_event_writer.write(SpawnMobEvent {
                                mob_type: MobType::FreighterTwo,
                                position: *spawn_location,
                                rotation: 0.0,
                            });
                        }

                        ui.menu_button("Misc", |ui| {
                            ui.menu_button("Freighter", |ui| {
                                if ui.button("Freighter Middle").clicked() {
                                    spawn_mob_event_writer.write(SpawnMobEvent {
                                        mob_type: MobType::FreighterMiddle,
                                        position: *spawn_location,
                                        rotation: 0.0,
                                    });
                                }

                                if ui.button("Freighter Back").clicked() {
                                    spawn_mob_event_writer.write(SpawnMobEvent {
                                        mob_type: MobType::FreighterBack,
                                        position: *spawn_location,
                                        rotation: 0.0,
                                    });
                                }
                            });
                        });
                    });
                });

                ui.menu_button("Projectile", |ui| {
                    ui.menu_button("Blast", |ui| {
                        if ui.button("Ally").clicked() {
                            spawn_projectile_event_writer.write(SpawnProjectileEvent {
                                projectile_type: ProjectileType::Blast,
                                faction: Faction::Ally,
                                position: *spawn_location,
                                rotation: 0.0,
                                speed: 0.0,
                                damage: 5,
                                range_seconds: 10.0,
                            });
                        }

                        if ui.button("Enemy").clicked() {
                            spawn_projectile_event_writer.write(SpawnProjectileEvent {
                                projectile_type: ProjectileType::Blast,
                                faction: Faction::Enemy,
                                position: *spawn_location,
                                rotation: 0.0,
                                speed: 0.0,
                                damage: 5,
                                range_seconds: 10.0,
                            });
                        }
                    });
                    ui.menu_button("Bullet", |ui| {
                        if ui.button("Ally").clicked() {
                            spawn_projectile_event_writer.write(SpawnProjectileEvent {
                                projectile_type: ProjectileType::Bullet,
                                faction: Faction::Ally,
                                position: *spawn_location,
                                rotation: 0.0,
                                speed: 0.0,
                                damage: 5,
                                range_seconds: 10.0,
                            });
                        }

                        if ui.button("Enemy").clicked() {
                            spawn_projectile_event_writer.write(SpawnProjectileEvent {
                                projectile_type: ProjectileType::Bullet,
                                faction: Faction::Enemy,
                                position: *spawn_location,
                                rotation: 0.0,
                                speed: 0.0,
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
