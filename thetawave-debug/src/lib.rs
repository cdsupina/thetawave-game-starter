use bevy::{
    app::{Plugin, Update},
    ecs::{
        event::{Event, EventWriter},
        schedule::IntoScheduleConfigs,
        system::{Local, Res},
    },
    input::{keyboard::KeyCode, ButtonInput},
    state::condition::in_state,
};
use thetawave_states::AppState;

pub struct ThetawaveDebugPlugin;

impl Plugin for ThetawaveDebugPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, toggle_debug_mode.run_if(in_state(AppState::Game)));
        app.add_event::<ToggleDebugModeEvent>();
    }
}

#[derive(Event)]
pub struct ToggleDebugModeEvent(pub bool);

fn toggle_debug_mode(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut is_active: Local<bool>,
    mut toggle_debug_event_writer: EventWriter<ToggleDebugModeEvent>,
) {
    if keyboard_input.just_released(KeyCode::Backquote) {
        *is_active = !*is_active;
    }

    toggle_debug_event_writer.write(ToggleDebugModeEvent(*is_active));
}
