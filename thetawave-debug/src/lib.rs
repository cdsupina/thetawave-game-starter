use bevy::{
    app::{Plugin, Update},
    ecs::{event::EventWriter, resource::Resource, system::Res},
    input::{keyboard::KeyCode, ButtonInput},
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use thetawave_states::ToggleDebugStateEvent;

pub struct ThetawaveDebugPlugin {
    /// The keycode to toggle debug mode on release
    pub show_debug_keycode: KeyCode,
}

impl Plugin for ThetawaveDebugPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(InspectorDebugSettings::default());
        app.add_plugins(
            WorldInspectorPlugin::new()
                .run_if(|res: Res<InspectorDebugSettings>| res.inspector_enabled),
        );
        app.add_systems(Update, toggle_debug_mode);
        app.insert_resource(DebugKeycode(self.show_debug_keycode));
    }
}

#[derive(Resource)]
struct DebugKeycode(KeyCode);

// Toggle debug mode on keycode release specified in plugin
fn toggle_debug_mode(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut toggle_debug_event_writer: EventWriter<ToggleDebugStateEvent>,
    debug_keycode: Res<DebugKeycode>,
) {
    if keyboard_input.just_released(debug_keycode.0) {
        toggle_debug_event_writer.write(ToggleDebugStateEvent);
    }
}

#[derive(Resource, Default)]
pub struct InspectorDebugSettings {
    pub inspector_enabled: bool,
}
