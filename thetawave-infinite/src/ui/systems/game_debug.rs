use bevy::ecs::{error::Result, system::ResMut};
#[cfg(feature = "debug")]
use bevy::ecs::{event::EventWriter, system::Local};
#[cfg(feature = "debug")]
use bevy_egui::{
    EguiContexts,
    egui::{TopBottomPanel, menu},
};

#[cfg(feature = "debug")]
use thetawave_starter::{
    InspectorDebugSettings, PhysicsDebugSettings, SpawnMobEvent, camera::CameraZoomEvent,
};

#[cfg(feature = "debug")]
/// System that handles the egui debug menu
pub(in crate::ui) fn game_debug_menu_system(
    mut contexts: EguiContexts,
    mut physics_debug_settings: ResMut<PhysicsDebugSettings>,
    mut inspector_debug_settings: ResMut<InspectorDebugSettings>,
    mut spawn_mob_event_writer: EventWriter<SpawnMobEvent>,
    mut camera_zoom_event_writer: EventWriter<CameraZoomEvent>,
    mut camera_zoom: Local<i8>,
) -> Result {
    use bevy::math::Vec2;
    use thetawave_starter::MobType;

    let mut camera_zoom_new = *camera_zoom;

    TopBottomPanel::top("menu_bar").show(contexts.ctx_mut()?, |ui| {
        menu::bar(ui, |ui| {
            ui.menu_button("Inspector", |ui| {
                ui.checkbox(
                    &mut inspector_debug_settings.inspector_enabled,
                    "World Inspector",
                );
            });

            ui.menu_button("View", |ui| {
                use bevy_egui::egui::Slider;

                ui.horizontal(|ui| {
                    ui.label("Zoom");
                    ui.add(Slider::new(&mut camera_zoom_new, -100..=100));
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
                ui.menu_button("Mob", |ui| {
                    if ui.button("Grunt").clicked() {
                        spawn_mob_event_writer.write(SpawnMobEvent {
                            mob_type: MobType::Grunt,
                            position: Vec2::new(0.0, 50.0),
                        });
                    }

                    if ui.button("Shooter").clicked() {
                        spawn_mob_event_writer.write(SpawnMobEvent {
                            mob_type: MobType::Shooter,
                            position: Vec2::new(0.0, 50.0),
                        });
                    }
                });
            });
        })
    });

    // Update local variable and send zoom event if it changed
    if camera_zoom_new != *camera_zoom {
        *camera_zoom = camera_zoom_new;
        camera_zoom_event_writer.write(CameraZoomEvent(*camera_zoom));
    }

    Ok(())
}
