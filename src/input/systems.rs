use bevy::prelude::ResMut;
use bevy_alt_ui_navigation_lite::systems::InputMapping;

/// Setup function for input mapping configuration
/// Sets focus to follow mouse movement and enables keyboard navigation
pub(super) fn setup(mut input_mapping: ResMut<InputMapping>) {
    input_mapping.focus_follows_mouse = true;
    input_mapping.keyboard_navigation = true;
}
