use bevy::{
    input::keyboard::NativeKeyCode,
    prelude::{GamepadButton, KeyCode, ResMut},
};
use bevy_alt_ui_navigation_lite::systems::InputMapping;

/// Setup function for input mapping configuration
pub(super) fn setup_input_system(mut input_mapping: ResMut<InputMapping>) {
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
