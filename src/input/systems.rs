use super::InputType;
use crate::{player::PlayerNum, ui::PlayerJoinEvent};
use bevy::{
    input::keyboard::NativeKeyCode,
    prelude::{
        Commands, Component, Entity, EventReader, GamepadButton, KeyCode, Query, Res, ResMut, With,
    },
};
use bevy_alt_ui_navigation_lite::systems::InputMapping;

#[derive(Component)]
pub(super) struct DummyGamepad;

/// Setup function for input mapping configuration
pub(super) fn setup_input_system(mut input_mapping: ResMut<InputMapping>, mut cmds: Commands) {
    // dummy gamepad for disabling all gamepads
    cmds.spawn(DummyGamepad);

    // Set action keyboard binding to enter
    input_mapping.key_action = KeyCode::Enter;
    // Disable key_free binding
    input_mapping.key_free = KeyCode::Unidentified(NativeKeyCode::Unidentified);
    input_mapping.key_cancel = KeyCode::Unidentified(NativeKeyCode::Unidentified);
    input_mapping.key_next = KeyCode::Unidentified(NativeKeyCode::Unidentified);
    input_mapping.key_next_alt = KeyCode::Unidentified(NativeKeyCode::Unidentified);
    input_mapping.key_previous = KeyCode::Unidentified(NativeKeyCode::Unidentified);

    input_mapping.free_button = GamepadButton::Other(255);
    input_mapping.cancel_button = GamepadButton::Other(255);
    input_mapping.previous_button = GamepadButton::Other(255);
    input_mapping.next_button = GamepadButton::Other(255);

    // Sets focus to follow mouse movement and enables keyboard navigation
    input_mapping.focus_follows_mouse = true;
    input_mapping.keyboard_navigation = true;
}

/// Disable other inputs for menu navigation once a player joins
pub(super) fn disable_additional_players_navigation_system(
    mut input_mapping: ResMut<InputMapping>,
    mut player_join_events: EventReader<PlayerJoinEvent>,
    dummy_gamepad_q: Query<Entity, With<DummyGamepad>>,
) {
    for event in player_join_events.read() {
        if matches!(event.player_num, PlayerNum::One) {
            match event.input {
                InputType::Keyboard => {
                    input_mapping.focus_follows_mouse = true;
                    input_mapping.keyboard_navigation = true;
                    if let Ok(entity) = dummy_gamepad_q.get_single() {
                        input_mapping.gamepads = vec![entity];
                    }
                }
                InputType::Gamepad(entity) => {
                    input_mapping.gamepads.push(entity);
                    input_mapping.focus_follows_mouse = false;
                    input_mapping.keyboard_navigation = false;
                }
            }
        }
    }
}

/// Enable navigation again when entering the Title state
pub(super) fn enable_additional_players_navigation_system(mut input_mapping: ResMut<InputMapping>) {
    input_mapping.focus_follows_mouse = true;
    input_mapping.keyboard_navigation = true;
    input_mapping.gamepads = vec![];
}
