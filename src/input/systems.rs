use bevy::{
    input::keyboard::NativeKeyCode,
    prelude::{KeyCode, ResMut},
};
use bevy_alt_ui_navigation_lite::systems::InputMapping;

/// Setup function for input mapping configuration
pub(super) fn setup_input_system(mut input_mapping: ResMut<InputMapping>) {
    // Set action keyboard binding to enter
    input_mapping.key_action = KeyCode::Enter;
    // Disable key_free binding
    input_mapping.key_free = KeyCode::Unidentified(NativeKeyCode::Unidentified);

    // Sets focus to follow mouse movement and enables keyboard navigation
    input_mapping.focus_follows_mouse = true;
    input_mapping.keyboard_navigation = true;
}
