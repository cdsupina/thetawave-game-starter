use bevy::prelude::Query;
use bevy_egui::EguiContextSettings;

/// Sets the scale of the egui elements
pub(in crate::ui) fn setup_egui_system(mut egui_settings: Query<&mut EguiContextSettings>) {
    // Increase scale of egui options menu
    egui_settings.single_mut().scale_factor = 2.0;
}
