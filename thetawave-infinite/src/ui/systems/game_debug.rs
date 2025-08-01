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
    InspectorDebugSettings, PhysicsDebugSettings, SpawnMobEvent,
    camera::{Camera2DZoomEvent, Camera3DZoomEvent},
};

/// System that handles the egui debug menu
#[cfg(feature = "debug")]
pub(in crate::ui) fn game_debug_menu_system(
    mut contexts: EguiContexts,
    mut physics_debug_settings: ResMut<PhysicsDebugSettings>,
    mut inspector_debug_settings: ResMut<InspectorDebugSettings>,
    mut spawn_mob_event_writer: EventWriter<SpawnMobEvent>,
    mut camera2d_zoom_event_writer: EventWriter<Camera2DZoomEvent>,
    mut camera2d_zoom: Local<i8>,
    mut camera3d_zoom_event_writer: EventWriter<Camera3DZoomEvent>,
    mut camera3d_zoom: Local<i8>,
) -> Result {
    use bevy::math::Vec2;
    use strum::IntoEnumIterator;
    use thetawave_starter::MobType;

    let mut camera2d_zoom_new = *camera2d_zoom;
    let mut camera3d_zoom_new = *camera3d_zoom;

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
                ui.menu_button("Mob", |ui| {
                    // Iterate through all MobTypes and create spawn buttons
                    for mob_type in MobType::iter() {
                        if ui.button(format!("{mob_type:?}")).clicked() {
                            spawn_mob_event_writer.write(SpawnMobEvent {
                                mob_type,
                                position: Vec2::new(0.0, 75.0),
                            });
                        }
                    }
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
